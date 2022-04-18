import cosmos.tx.v1beta1.ServiceOuterClass
import cosmos.tx.v1beta1.TxOuterClass
import io.provenance.client.grpc.BaseReqSigner
import io.provenance.client.grpc.PbClient
import io.provenance.client.protobuf.extensions.getBaseAccount
import io.provenance.client.protobuf.extensions.toAny
import io.provenance.metadata.v1.ContractSpecification
import io.provenance.metadata.v1.ContractSpecificationRequest
import io.provenance.metadata.v1.DefinitionType
import io.provenance.metadata.v1.Description
import io.provenance.metadata.v1.InputSpecification
import io.provenance.metadata.v1.MsgAddContractSpecToScopeSpecRequest
import io.provenance.metadata.v1.MsgWriteContractSpecificationRequest
import io.provenance.metadata.v1.MsgWriteRecordSpecificationRequest
import io.provenance.metadata.v1.MsgWriteScopeSpecificationRequest
import io.provenance.metadata.v1.PartyType
import io.provenance.metadata.v1.RecordSpecification
import io.provenance.metadata.v1.ScopeSpecification
import io.provenance.metadata.v1.ScopeSpecificationRequest
import io.provenance.scope.util.MetadataAddress
import io.provenance.scope.util.toByteString
import java.util.UUID

data class ScopeSpecificationInfo(
    val scopeSpecUuid: UUID,
    val contractSpecUuid: UUID,
    val recordInputName: String,
    val recordName: String
)

class ScopeSpecificationCreator(
    private val pbClient: PbClient
) {
    fun createSpecAndRecordsIfNotExists(scopeSpecUuid: UUID, signer: WalletSigner): ScopeSpecificationInfo {
        val existingScopeSpec = pbClient.metadataClient.scopeSpecification(ScopeSpecificationRequest.newBuilder()
            .setSpecificationId(scopeSpecUuid.toString())
            .build())

        if (existingScopeSpec.scopeSpecification.hasSpecification()) {
            // scope spec already exists, return info
            println("Found ScopeSpec with id $scopeSpecUuid, skipping creation")

            val contractSpecUuid = existingScopeSpec.scopeSpecification.specification.contractSpecIdsList.first().let { MetadataAddress.fromBytes(it.toByteArray()).getPrimaryUuid() }
            val contractSpec = pbClient.metadataClient.contractSpecification(ContractSpecificationRequest.newBuilder()
                .setSpecificationId(contractSpecUuid.toString())
                .setIncludeRecordSpecs(true)
                .build()
            )

            return ScopeSpecificationInfo(
                scopeSpecUuid,
                contractSpecUuid,
                contractSpec.recordSpecificationsList.first().specification.inputsList.first().name,
                contractSpec.recordSpecificationsList.first().specification.name,
            )
        }

        println("Creating ScopeSpec with id $scopeSpecUuid")

        // Create a scope specification. This defines which contract specifications are allowed to be used to modify the
        // data within a scope of this type.
        val createScopeSpec = MsgWriteScopeSpecificationRequest.newBuilder()
            .setSpecUuid(scopeSpecUuid.toString())
            .setSpecification(
                ScopeSpecification.newBuilder()
                .setDescription(
                    Description.newBuilder()
                    .setName("testScopeSpec")
                    .setDescription("A scope specification for testing")
                ).addPartiesInvolved(PartyType.PARTY_TYPE_OWNER)
                .addOwnerAddresses(signer.address())
            )
            .addSigners(signer.address())
            .build()
            .toAny()

        // Create a contract specification. This defines an off-chain process that can be used to generate/modify
        // records for a scope. Resulting records will be contained in a session linked to this specification
        val CONTRACT_SPEC_UUID = UUID.randomUUID()
        val createContractSpec = MsgWriteContractSpecificationRequest.newBuilder()
            .setSpecUuid(CONTRACT_SPEC_UUID.toString())
            .setSpecification(
                ContractSpecification.newBuilder()
                .setDescription(
                    Description.newBuilder()
                    .setName("myCoolContract")
                    .setDescription("A contract that does things for testScopeSpec")
                )
                .addPartiesInvolved(PartyType.PARTY_TYPE_OWNER)
                .setClassName("io.provenance.example.testContract")
                .setHash("fakeHash123") // this should be the hash of the code (i.e. Java jar, etc) that this contract spec represents
                .addOwnerAddresses(signer.address())
            )
            .addSigners(signer.address())
            .build()
            .toAny()

        val scopeSpecId = MetadataAddress.forScopeSpecification(scopeSpecUuid)
        val contractSpecId = MetadataAddress.forContractSpecification(CONTRACT_SPEC_UUID)
        val addContractSpecToScopeSpec = MsgAddContractSpecToScopeSpecRequest.newBuilder()
            .setScopeSpecificationId(scopeSpecId.bytes.toByteString())
            .setContractSpecificationId(contractSpecId.bytes.toByteString())
            .addSigners(signer.address())
            .build()
            .toAny()

        val RECORD_NAME = "record1"
        val RECORD_INPUT_NAME = "${RECORD_NAME}Input1"
        val createRecordSpec = MsgWriteRecordSpecificationRequest.newBuilder()
            .setContractSpecUuid(CONTRACT_SPEC_UUID.toString())
            .setSpecification(
                RecordSpecification.newBuilder()
                .setName(RECORD_NAME)
                .setTypeName(com.google.protobuf.StringValue::class.java.name)
                .addResponsibleParties(PartyType.PARTY_TYPE_OWNER)
                .addAllInputs(listOf(
                    InputSpecification.newBuilder()
                        .setName(RECORD_INPUT_NAME)
                        .setTypeName(com.google.protobuf.StringValue::class.java.name) // just using a string for illustration purposes
                        .setHash("fakeProtoHash123") // this should be the hash of the code that defines this type (i.e. java jar containing compiled proto definitions that the data could be deserialized into)
                        .build()
                ))
                .setResultType(DefinitionType.DEFINITION_TYPE_RECORD)
            )
            .addSigners(signer.address())
            .build()
            .toAny()

        val specMessages = listOf(createScopeSpec, createContractSpec, addContractSpecToScopeSpec, createRecordSpec)

        pbClient.estimateAndBroadcastTx(
            TxOuterClass.TxBody.newBuilder().addAllMessages(specMessages).build(), listOf(
            BaseReqSigner(signer)
        ), mode = ServiceOuterClass.BroadcastMode.BROADCAST_MODE_BLOCK).let { result ->
            if (result.txResponse.code != 0) {
                throw Exception("Error broadcasting specification messages")
            }

            println("successfully created specifications: ${result.txResponse.txhash}")
        }

        return ScopeSpecificationInfo(
            scopeSpecUuid,
            CONTRACT_SPEC_UUID,
            RECORD_INPUT_NAME,
            RECORD_NAME
        )
    }
}
