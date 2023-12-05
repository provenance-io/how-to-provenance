package io.p8e.demo

import com.google.protobuf.Any
import cosmos.base.abci.v1beta1.Abci
import cosmos.tx.v1beta1.ServiceOuterClass
import cosmos.tx.v1beta1.TxOuterClass
import io.p8e.demo.proto.LoanData
import io.p8e.demo.proto.loan
import io.p8e.demo.proto.servicer
import io.provenance.client.grpc.BaseReqSigner
import io.provenance.client.grpc.GasEstimationMethod
import io.provenance.client.grpc.PbClient
import io.provenance.client.wallet.NetworkType
import io.provenance.client.wallet.fromMnemonic
import io.provenance.metadata.v1.ScopeResponse
import io.provenance.metadata.v1.scopeRequest
import io.provenance.scope.contract.annotations.Record
import io.provenance.scope.contract.proto.Specifications
import io.provenance.scope.encryption.ecies.ECUtils
import io.provenance.scope.encryption.model.DirectKeyRef
import io.provenance.scope.encryption.util.toJavaPrivateKey
import io.provenance.scope.sdk.Affiliate
import io.provenance.scope.sdk.Client
import io.provenance.scope.sdk.ClientConfig
import io.provenance.scope.sdk.Session
import io.provenance.scope.sdk.SharedClient
import io.provenance.scope.sdk.SignedResult
import java.net.URI
import java.util.*
import java.util.concurrent.TimeUnit

/**
 * Data class for hydrating.
 */
data class Loan(
    @Record("loan") val loan: LoanData.Loan,
    @Record("servicer") val servicer: LoanData.Servicer
)

/**
 * A GRPC client instance that connects to a local Provennace Blockchain instance.
 */
val pbcClient = PbClient("chain-local", URI("grpc://localhost:9090"), GasEstimationMethod.MSG_FEE_CALCULATION)

// sdk client for contract execution and object store interactions
/**
 * Builds a WalletSigner instance from a specified NetworkType, using the correct hrp (human-readable-prefix) of tp
 * for Provenance Blockchain testnet accounts, and the correct HD (hierarchical deterministic) path.
 * Uses a default mnemonic value to ensure that the address used across all executions remains constant.
 */
private val signer = fromMnemonic(
    networkType = NetworkType(prefix = "tp", path = "m/44'/1'/0'/0/0"),
    mnemonic = "stable payment cliff fault abuse clinic bus belt film then forward world goose bring picnic rich special brush basic lamp window coral worry change"
)
private val encryptionPrivateKey = "0A2100EF4A9391903BFE252CB240DA6695BC5F680A74A8E16BEBA003833DFE9B18C147".toJavaPrivateKey()
private val signingPrivateKey = "0A2100EF4A9391903BFE252CB240DA6695BC5F680A74A8E16BEBA003833DFE9B18C147".toJavaPrivateKey()
private val affiliate = Affiliate(
    encryptionKeyRef = DirectKeyRef(ECUtils.toPublicKey(encryptionPrivateKey)!!, encryptionPrivateKey),
    signingKeyRef = DirectKeyRef(ECUtils.toPublicKey(signingPrivateKey)!!, signingPrivateKey),
    partyType = Specifications.PartyType.ORIGINATOR,
)
private val config = ClientConfig(
    cacheJarSizeInBytes = 0L,
    cacheSpecSizeInBytes = 0L,
    cacheRecordSizeInBytes = 0L,
    osGrpcUrl = URI("grpc://localhost:8090"),
    mainNet = false,
)
val sdkClient = Client(SharedClient(config = config), affiliate)

/**
 * Execute the contract and send to PBC.
 */
fun executeContractAndSendAsTx(session: Session): ServiceOuterClass.BroadcastTxResponse =
    (sdkClient.execute(session) as SignedResult).let { executionResult ->
        val messages = executionResult.messages.map { Any.pack(it, "") }
        TxOuterClass.TxBody.newBuilder().addAllMessages(messages).build()
    }.let { txBody ->
        pbcClient.estimateAndBroadcastTx(
            txBody,
            signers = listOf(BaseReqSigner(signer)),
            mode = ServiceOuterClass.BroadcastMode.BROADCAST_MODE_BLOCK
        )
    }

/**
 * Parses an error message from a TxResponse, if a tx has failed.
 */
fun Abci.TxResponse.getError(): String =
    logsList.filter { it.log.isNotBlank() }.takeIf { it.isNotEmpty() }?.joinToString("; ") { it.log }
        ?: rawLog

/**
 * Fetches the scope from PBC.
 */
fun loadScope(scopeUuid: UUID): ScopeResponse =
    scopeRequest {
        scopeId = scopeUuid.toString()
        includeSessions = true
        includeRecords = true
        includeRequest = true
    }.let { request ->
        pbcClient.metadataClient.withDeadlineAfter(10, TimeUnit.SECONDS).scope(request)
    }

/**
 * Hydrates data from a scope to object store.
 */
fun hydrateLoan(scopeUuid: UUID): Loan =
    loadScope(scopeUuid).let { scope -> sdkClient.hydrate(Loan::class.java, scope) }

fun executeCreateLoanScopeContract(
    scopeUuid: UUID,
    loan: LoanData.Loan,
    servicer: LoanData.Servicer
): ServiceOuterClass.BroadcastTxResponse {

    // a session is the representation in a scope of a contract execution
    val session = sdkClient
        .newSession(CreateLoanScopeContract::class.java, DemoLoanScopeSpecification::class.java)
        .setScopeUuid(scopeUuid)
        .addProposedRecord("loan", loan)
        .addProposedRecord("servicer", servicer)
        .build()

    // the contract and send the execution results to provenance blockchain
    return executeContractAndSendAsTx(session)
}

fun executeUpdateLoanScopeServicerContract(
    scopeUuid: UUID,
    servicer: LoanData.Servicer
): ServiceOuterClass.BroadcastTxResponse {

    // fetch the scope from PBC
    val scope = loadScope(scopeUuid)

    // a session is the representation in a scope of a contract execution
    val session = sdkClient
        .newSession(UpdateLoanScopeServicerContract::class.java, scope)
        .addProposedRecord("servicer", servicer)
        .build()

    // the contract and send the execution results to provenance blockchain
    return executeContractAndSendAsTx(session)
}

fun main() {
    // this will be the uuid of our example scope
    val scopeUuid = UUID.randomUUID()

    // the data we want to save
    val loanRecord = loan {
        id = scopeUuid.toString()
        type = "SOME_LOAN_TYPE"
        originator = "IM_A_BANK"
    }
    val servicerRecord = servicer {
        id = UUID.randomUUID().toString()
        name = "IM_A_LOAN_SERVICER"
    }

    println("Creating scope record for loan $scopeUuid...")
    val createResponse = executeCreateLoanScopeContract(scopeUuid, loanRecord, servicerRecord)
    if (createResponse.txResponse.code == 0) {
        println("Scope: $scopeUuid")
        println("Tx Hash: ${createResponse.txResponse.txhash}")
    } else { //error
        throw IllegalStateException(createResponse.txResponse.getError())
    }

    println("Checking the data stored on chain for loan $scopeUuid...")
    hydrateLoan(scopeUuid).let { data ->
        println("Loan Id: ${data.loan.id}")
        println("Loan Type: ${data.loan.type}")
        println("Loan Originator: ${data.loan.originator}")
        println("Loan Servicer Id: ${data.servicer.id}")
        println("Loan Servicer Name: ${data.servicer.name}")
    }

    // data for the new servicer
    val newServicer = servicer {
        id = UUID.randomUUID().toString()
        name = "IM_A_DIFFERENT_LOAN_SERVICER"
    }

    println("Updating the servicer record for loan $scopeUuid...")
    val updateResponse = executeUpdateLoanScopeServicerContract(scopeUuid, newServicer)
    if (updateResponse.txResponse.code == 0) {
        println("Scope: $scopeUuid")
        println("Tx Hash: ${updateResponse.txResponse.txhash}")
    } else { //error
        throw IllegalStateException(updateResponse.txResponse.getError())
    }

    println("Checking the data stored on chain for loan $scopeUuid...")
    hydrateLoan(scopeUuid).let { data ->
        println("Loan Id: ${data.loan.id}")
        println("Loan Type: ${data.loan.type}")
        println("Loan Originator: ${data.loan.originator}")
        println("Loan Servicer Id: ${data.servicer.id}")
        println("Loan Servicer Name: ${data.servicer.name}")
    }
}
