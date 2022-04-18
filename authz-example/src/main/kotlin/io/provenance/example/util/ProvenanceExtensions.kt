package io.provenance.example.util

import com.google.protobuf.Message
import cosmos.bank.v1beta1.QueryOuterClass.QueryBalanceRequest
import cosmos.tx.v1beta1.ServiceOuterClass.BroadcastMode
import cosmos.tx.v1beta1.ServiceOuterClass.BroadcastTxResponse
import cosmos.tx.v1beta1.TxOuterClass
import io.provenance.client.grpc.BaseReqSigner
import io.provenance.client.grpc.PbClient
import io.provenance.client.protobuf.extensions.getBaseAccount
import io.provenance.client.protobuf.extensions.toAny
import io.provenance.client.wallet.WalletSigner
import io.provenance.name.v1.QueryResolveRequest

private val DEFAULT_BROADCAST_MODE = BroadcastMode.BROADCAST_MODE_BLOCK
private const val DEFAULT_GAS_ADJUSTMENT: Double = 2.0

fun PbClient.executeTx(
    signer: WalletSigner,
    transactions: Collection<Message>,
    broadcastMode: BroadcastMode = DEFAULT_BROADCAST_MODE,
    gasAdjustment: Double = DEFAULT_GAS_ADJUSTMENT,
    feeGranter: WalletSigner? = null,
): BroadcastTxResponse = this.estimateAndBroadcastTx(
    txBody = TxOuterClass.TxBody.newBuilder().addAllMessages(transactions.map { it.toAny() }).build(),
    signers = BaseReqSigner(
        signer = signer,
        account = this.authClient.getBaseAccount(signer.address()),
    ).let(::listOf),
    mode = broadcastMode,
    gasAdjustment = gasAdjustment,
    feeGranter = feeGranter?.address(),
).also { response ->
    // A non-zero response code indicates a failure of some sort.  The rawLog contains hints and information about the
    // failure in this scenario. Emit the message as an exception to reveal the issue
    if (response.txResponse.code != 0) {
        throw IllegalStateException("Failed to execute transaction(s) and got response: ${response.txResponse.rawLog}")
    }
}

fun PbClient.resolveName(name: String): String = nameClient.resolve(
    QueryResolveRequest.newBuilder().setName(name).build()
).address

fun PbClient.queryBalance(address: String, denom: String): Long = bankClient.balance(
    QueryBalanceRequest.newBuilder().setAddress(address).setDenom(denom).build()
).balance.amount.toLong()
