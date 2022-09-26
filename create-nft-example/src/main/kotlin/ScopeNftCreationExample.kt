import cosmos.tx.v1beta1.ServiceOuterClass
import cosmos.tx.v1beta1.TxOuterClass
import io.provenance.client.grpc.BaseReqSigner
import io.provenance.client.grpc.GasEstimationMethod
import io.provenance.client.grpc.PbClient
import io.provenance.client.protobuf.extensions.toAny
import io.provenance.client.wallet.NetworkType
import io.provenance.client.wallet.fromMnemonic
import io.provenance.metadata.v1.MsgWriteRecordRequest
import io.provenance.metadata.v1.MsgWriteScopeRequest
import io.provenance.metadata.v1.MsgWriteSessionRequest
import io.provenance.metadata.v1.Party
import io.provenance.metadata.v1.PartyType
import io.provenance.metadata.v1.Process
import io.provenance.metadata.v1.Record
import io.provenance.metadata.v1.RecordInput
import io.provenance.metadata.v1.RecordInputStatus
import io.provenance.metadata.v1.RecordOutput
import io.provenance.metadata.v1.ResultStatus
import io.provenance.metadata.v1.Scope
import io.provenance.metadata.v1.ScopeRequest
import io.provenance.metadata.v1.Session
import io.provenance.scope.util.MetadataAddress
import io.provenance.scope.util.toByteString
import io.provenance.scope.util.toUuid
import java.net.URI
import java.util.UUID

/**
 * Create a scope (NFT/Non-Fungible Token) on Provenance Blockchain
 */
class ScopeNftCreationExample {
    // configuration
    val chainId = System.getenv("CHAIN_ID") ?: "chain-local"
    val nodeUri = System.getenv("NODE_URI") ?: "grpc://localhost:9090"
    val pbClient = PbClient(chainId, URI(nodeUri), GasEstimationMethod.MSG_FEE_CALCULATION)

    val scopeSpecUuid = System.getenv("SCOPE_SPEC_UUID")?.toUuid() ?: UUID.randomUUID()

    // setting up somewhere to store my off-chain data. In reality, this would be a database, Provenance object-store (github.com/provenance-io/object-store)
    // or something else. This just allows me to put/get data by the sha256 of the data
    val fakeStorage = FakeDataStore()

    init {
        // this is the account to create the scope/specifications and be set as the owner of the scope
        println("Please enter your mnemonic")
    }
    val mnemonic = readLine()!!
    val signer = fromMnemonic(NetworkType(prefix = "tp", path = "m/44'/1'/0'/0/0"), mnemonic)

    init {
        println("Please enter a value for the record")
    }
    val recordInputString = readLine()!!

    fun run() {
        // first we need to have some setup to define the type of asset this NFT represents.
        // This takes the form of a scope specification, contract specification and record specifications.
        // If an appropriate asset class is pre-existing for the type of scope you are creating, then this doesn't need
        // to be set up again. Typically, this is more of an ahead-of-time configuration task that may happen via a different
        // service or as a one-time configuration.
        val scopeSpecInfo = ScopeSpecificationCreator(pbClient).createSpecAndRecordsIfNotExists(scopeSpecUuid, signer)


        // Now we are going to actually create a scope, that will contain a session reflecting some off-chain process
        // that will contain a record that is referenced by hash on-chain and can be fetched as such off-chain
        // note that some of the hashes are faked here, and in reality would refer to some well-known code off-chain that could
        // be used to verify that an output hash could be produced from the input hashes
        val scopeUuid = createScope(scopeSpecInfo)

        // Now we will fetch the scope from the chain, inspect the contents and retrieve the underlying data based
        // on those contents
        verifyScope(scopeSpecInfo, scopeUuid)
    }

    /**
     * A function to simulate some off-chain process handling inputs and forming output
     */
    fun inputModifier(input: String): String = "${input}-modified"

    /**
     * Create a scope with one record conforming to the previously-created scope specification
     */
    fun createScope(scopeSpecInfo: ScopeSpecificationInfo): UUID {
        val scopeUuid = UUID.randomUUID()
        println("Scope UUID is: $scopeUuid")
        val writeScope = MsgWriteScopeRequest.newBuilder()
            .setScopeUuid(scopeUuid.toString())
            .setSpecUuid(scopeSpecInfo.scopeSpecUuid.toString())
            .setScope(
                Scope.newBuilder()
                .addOwners(
                    Party.newBuilder()
                    .setAddress(signer.address())
                    .setRole(PartyType.PARTY_TYPE_OWNER)
                )
                .setValueOwnerAddress(signer.address())
            )
            .addSigners(signer.address())
            .build()
            .toAny()

        val sessionUuid = UUID.randomUUID()
        val sessionId = MetadataAddress.forSession(scopeUuid, sessionUuid)
        val writeSession = MsgWriteSessionRequest.newBuilder()
            .setSpecUuid(scopeSpecInfo.contractSpecUuid.toString())
            .setSession(
                Session.newBuilder()
                .setSessionId(sessionId.bytes.toByteString())
                .setName("Initial session")
                .addParties(
                    Party.newBuilder()
                    .setAddress(signer.address())
                    .setRole(PartyType.PARTY_TYPE_OWNER)
                )
            )
            .addSigners(signer.address())
            .build()
            .toAny()

        val recordInputHash = com.google.protobuf.StringValue.newBuilder()
            .setValue(recordInputString)
            .build().toByteArray().let {
                fakeStorage.put(it)
            }

        val recordOutputHash = com.google.protobuf.StringValue.newBuilder()
            .setValue(inputModifier(recordInputString))
            .build().toByteArray().let {
                fakeStorage.put(it)
            }

        // The record is where we supply hashes that represent inputs/outputs to some function (input could be the same as
        // the output), where the hashes also allow us to later fetch this data off-chain, and verify its validity
        val writeRecord = MsgWriteRecordRequest.newBuilder()
            .setContractSpecUuid(scopeSpecInfo.contractSpecUuid.toString())
            .setRecord(
                Record.newBuilder()
                .setName(scopeSpecInfo.recordName)
                .addInputs(
                    RecordInput.newBuilder()
                    .setHash(recordInputHash)
                    .setTypeName(com.google.protobuf.StringValue::class.java.name)
                    .setName(scopeSpecInfo.recordInputName)
                    .setStatusValue(RecordInputStatus.RECORD_INPUT_STATUS_PROPOSED_VALUE) // proposed as new value, as opposed to being pulled in from existing scope (as opposed to RECORD_INPUT_STATUS_RECORD_VALUE)
                )
                .addOutputs(
                    RecordOutput.newBuilder()
                    .setHash(recordOutputHash)
                    .setStatus(ResultStatus.RESULT_STATUS_PASS)
                )
                .setProcess(
                    Process.newBuilder()
                    .setName(com.google.protobuf.StringValue::class.java.name)
                    .setHash("fakeProtoHash123") // this should be the hash of the code that defines this type (i.e. java jar containing compiled proto definitions that the data could be deserialized into)
                    .setMethod("inputModifier") // reference to some method off-chain, in our case the 'inputModifier' function
                )
                .setSessionId(sessionId.bytes.toByteString())
            )
            .addParties(
                Party.newBuilder()
                .setAddress(signer.address())
                .setRole(PartyType.PARTY_TYPE_OWNER)
            )
            .addSigners(signer.address())
            .build()
            .toAny()

        // send the messages to the chain in a transaction
        val writeScopeMessages = listOf(writeScope, writeSession, writeRecord)
        pbClient.estimateAndBroadcastTx(
            TxOuterClass.TxBody.newBuilder().addAllMessages(writeScopeMessages).build(), listOf(
            BaseReqSigner(signer)
        ), mode = ServiceOuterClass.BroadcastMode.BROADCAST_MODE_BLOCK).let { result ->
            if (result.txResponse.code != 0) {
                throw Exception("Error broadcasting scope messages")
            }

            println("successfully created scope: ${result.txResponse.txhash}")
        }

        return scopeUuid
    }

    /**
     * Asserting that what is stored in the scope on-chain matches what is expected
     */
    fun verifyScope(scopeSpecInfo: ScopeSpecificationInfo, scopeUuid: UUID) {
        val scopeResponse = pbClient.metadataClient.scope(ScopeRequest.newBuilder().setScopeId(scopeUuid.toString()).setIncludeRecords(true).build())

        // our example
        val record = scopeResponse.recordsList.find { it.record.name == scopeSpecInfo.recordName }?.record ?: throw IllegalStateException("The specified record could not be found in scope")
        val queriedRecordInputHash = record.inputsList.first().hash
        val queriedRecordOutputHash = record.outputsList.first().hash

        // we can fetch off-chain data by hash
        val fetchedRecordInput = fakeStorage.get(queriedRecordInputHash)
        val fetchedRecordOutput = fakeStorage.get(queriedRecordOutputHash)

        // now we can hydrate the data back into the data type it represents and verify its validity
        val parsedInputValue = com.google.protobuf.StringValue.parseFrom(fetchedRecordInput).value.also { parsedInputValue ->
            assert(parsedInputValue == recordInputString) { "The parsed input value did not match what was provided" }
            println("Parsed input value as stored on chain: '$parsedInputValue'")
        }
        com.google.protobuf.StringValue.parseFrom(fetchedRecordOutput).value.let { parsedOutputValue ->
            assert(parsedOutputValue == inputModifier(parsedInputValue)) { "The parsed output value did not match what was provided when run through the specified function" }
            println("Parsed output value as stored on chain: '$parsedOutputValue'")
        }
        println("Successfully verified scope data ✅")
    }
}
