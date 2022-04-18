import cosmos.tx.v1beta1.ServiceOuterClass
import cosmos.tx.v1beta1.TxOuterClass
import io.provenance.client.grpc.BaseReqSigner
import io.provenance.client.grpc.GasEstimationMethod
import io.provenance.client.grpc.PbClient
import io.provenance.client.protobuf.extensions.getBaseAccount
import io.provenance.client.protobuf.extensions.toAny
import io.provenance.metadata.v1.ContractSpecification
import io.provenance.metadata.v1.DefinitionType
import io.provenance.metadata.v1.Description
import io.provenance.metadata.v1.InputSpecification
import io.provenance.metadata.v1.MsgAddContractSpecToScopeSpecRequest
import io.provenance.metadata.v1.MsgWriteContractSpecificationRequest
import io.provenance.metadata.v1.MsgWriteRecordRequest
import io.provenance.metadata.v1.MsgWriteRecordSpecificationRequest
import io.provenance.metadata.v1.MsgWriteScopeRequest
import io.provenance.metadata.v1.MsgWriteScopeSpecificationRequest
import io.provenance.metadata.v1.MsgWriteSessionRequest
import io.provenance.metadata.v1.Party
import io.provenance.metadata.v1.PartyType
import io.provenance.metadata.v1.Process
import io.provenance.metadata.v1.Record
import io.provenance.metadata.v1.RecordInput
import io.provenance.metadata.v1.RecordInputStatus
import io.provenance.metadata.v1.RecordOutput
import io.provenance.metadata.v1.RecordSpecification
import io.provenance.metadata.v1.ResultStatus
import io.provenance.metadata.v1.Scope
import io.provenance.metadata.v1.ScopeRequest
import io.provenance.metadata.v1.ScopeSpecification
import io.provenance.metadata.v1.Session
import io.provenance.scope.util.MetadataAddress
import io.provenance.scope.util.base64String
import io.provenance.scope.util.sha256
import io.provenance.scope.util.toByteString
import java.net.URI
import java.util.UUID

/**
 * Create a scope (NFT/Non-Fungible Token) on Provenance Blockchain
 */
fun main() {
    // configuration
    val chainId = System.getenv("CHAIN_ID") ?: "chain-local"
    val nodeUri = System.getenv("NODE_URI") ?: "grpc://localhost:9090"
    val pbClient = PbClient(chainId, URI(nodeUri), GasEstimationMethod.MSG_FEE_CALCULATION)

    // setting up somewhere to store my off-chain data. In reality, this would be a database, Provenance object-store (github.com/provenance-io/object-store)
    // or something else. This just allows me to put/get data by the sha256 of the data
    val fakeStorage = hashMapOf<String, ByteArray>()

    // first we need to have some setup to define the type of asset this NFT represents.
    // This takes the form of a scope specification, contract specification and record specifications.
    // If an appropriate asset class is pre-existing for the type of scope you are creating, then this doesn't need
    // to be set up again. Typically, this is more of an ahead-of-time configuration task that may happen via a different
    // service or as a one-time configuration.

    println("Please enter your mnemonic")
    val mnemonic = readLine()!!
    val signer = WalletSigner(NetworkType.TESTNET, mnemonic)

    // Create a scope specification. This defines which contract specifications are allowed to be used to modify the
    // data within a scope of this type.
    val SCOPE_SPEC_UUID = UUID.randomUUID()
    val createScopeSpec = MsgWriteScopeSpecificationRequest.newBuilder()
        .setSpecUuid(SCOPE_SPEC_UUID.toString())
        .setSpecification(ScopeSpecification.newBuilder()
            .setDescription(Description.newBuilder()
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
        .setSpecification(ContractSpecification.newBuilder()
            .setDescription(Description.newBuilder()
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

    val scopeSpecId = MetadataAddress.forScopeSpecification(SCOPE_SPEC_UUID)
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
        .setSpecification(RecordSpecification.newBuilder()
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

    val account = pbClient.authClient.getBaseAccount(signer.address())
    var sequenceOffset = 0
    pbClient.estimateAndBroadcastTx(TxOuterClass.TxBody.newBuilder().addAllMessages(specMessages).build(), listOf(
        BaseReqSigner(signer, sequenceOffset, account)
    ), mode = ServiceOuterClass.BroadcastMode.BROADCAST_MODE_BLOCK).let { result ->
        if (result.txResponse.code != 0) {
            throw Exception("Error broadcasting specification messages")
        }

        println("successfully created specifications: ${result.txResponse.txhash}")
        sequenceOffset++
    }

    val SCOPE_UUID = UUID.randomUUID()
    println("Scope UUID is: $SCOPE_UUID")
    val writeScope = MsgWriteScopeRequest.newBuilder()
        .setScopeUuid(SCOPE_UUID.toString())
        .setSpecUuid(SCOPE_SPEC_UUID.toString())
        .setScope(Scope.newBuilder()
            .addOwners(Party.newBuilder()
                .setAddress(signer.address())
                .setRole(PartyType.PARTY_TYPE_OWNER)
            )
            .setValueOwnerAddress(signer.address())
        )
        .addSigners(signer.address())
        .build()
        .toAny()

    val SESSION_UUID = UUID.randomUUID()
    val sessionId = MetadataAddress.forSession(SCOPE_UUID, SESSION_UUID)
    val writeSession = MsgWriteSessionRequest.newBuilder()
        .setSpecUuid(CONTRACT_SPEC_UUID.toString())
        .setSession(Session.newBuilder()
            .setSessionId(sessionId.bytes.toByteString())
            .setName("Initial session")
            .addParties(Party.newBuilder()
                .setAddress(signer.address())
                .setRole(PartyType.PARTY_TYPE_OWNER)
            )
        )
        .addSigners(signer.address())
        .build()
        .toAny()

    println("Please enter a value for the record")
    val recordInputString = readLine()!!
    val recordInputHash = com.google.protobuf.StringValue.newBuilder()
        .setValue(recordInputString)
        .build().toByteArray().let {
            val hash = it.sha256()
            fakeStorage.put(hash.base64String(), it)
            hash.base64String()
        }

    val recordOutputHash = com.google.protobuf.StringValue.newBuilder()
        .setValue("${recordInputString}-modified")
        .build().toByteArray().let {
            val hash = it.sha256()
            fakeStorage.put(hash.base64String(), it)
            hash.base64String()
        }

    val writeRecord = MsgWriteRecordRequest.newBuilder()
        .setContractSpecUuid(CONTRACT_SPEC_UUID.toString())
        .setRecord(Record.newBuilder()
            .setName(RECORD_NAME)
            .addInputs(RecordInput.newBuilder()
                .setHash(recordInputHash)
                .setTypeName(com.google.protobuf.StringValue::class.java.name)
                .setName(RECORD_INPUT_NAME)
                .setStatusValue(RecordInputStatus.RECORD_INPUT_STATUS_PROPOSED_VALUE) // proposed as new value, not pulled in from existing scope
            )
            .addOutputs(RecordOutput.newBuilder()
                .setHash(recordOutputHash)
                .setStatus(ResultStatus.RESULT_STATUS_PASS)
            )
            .setProcess(Process.newBuilder()
                .setName(com.google.protobuf.StringValue::class.java.name)
                .setHash("fakeProtoHash123")
                .setMethod("modifyString") // reference to some method off-chain
            )
            .setSessionId(sessionId.bytes.toByteString())
        )
        .addParties(Party.newBuilder()
            .setAddress(signer.address())
            .setRole(PartyType.PARTY_TYPE_OWNER)
        )
        .addSigners(signer.address())
        .build()
        .toAny()

    val writeScopeMessages = listOf(writeScope, writeSession, writeRecord)

    pbClient.estimateAndBroadcastTx(TxOuterClass.TxBody.newBuilder().addAllMessages(writeScopeMessages).build(), listOf(
        BaseReqSigner(signer, sequenceOffset, account)
    ), mode = ServiceOuterClass.BroadcastMode.BROADCAST_MODE_BLOCK).let { result ->
        if (result.txResponse.code != 0) {
            throw Exception("Error broadcasting scope messages")
        }

        println("successfully created scope: ${result.txResponse.txhash}")
        sequenceOffset++
    }

    val scopeResponse = pbClient.metadataClient.scope(ScopeRequest.newBuilder().setScopeId(SCOPE_UUID.toString()).setIncludeRecords(true).build())

    val record = scopeResponse.recordsList.first().record
    val queriedRecordInputHash = record.inputsList.first().hash
    val queriedRecordOutputHash = record.outputsList.first().hash

    val fetchedRecordInput = fakeStorage.get(queriedRecordInputHash)
    val fetchedRecordOutput = fakeStorage.get(queriedRecordOutputHash)

    com.google.protobuf.StringValue.parseFrom(fetchedRecordInput).value.let { parsedInputValue ->
        assert(parsedInputValue == recordInputString) { "The parsed input value did not match what was provided" }
        println("Parsed input value as stored on chain: '$parsedInputValue'")
    }
    com.google.protobuf.StringValue.parseFrom(fetchedRecordOutput).value.let { parsedOutputValue ->
        assert(parsedOutputValue == "${recordInputString}-modified") { "The parsed output value did not match what was provided" }
        println("Parsed output value as stored on chain: '$parsedOutputValue'")
    }
}
