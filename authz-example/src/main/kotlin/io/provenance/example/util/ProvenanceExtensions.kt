package io.provenance.example.util

import com.google.protobuf.Message
import cosmos.bank.v1beta1.QueryOuterClass.QueryBalanceRequest
import cosmos.tx.v1beta1.ServiceOuterClass.BroadcastMode
import cosmos.tx.v1beta1.ServiceOuterClass.BroadcastTxResponse
import cosmos.tx.v1beta1.TxOuterClass.TxBody
import io.provenance.client.grpc.BaseReqSigner
import io.provenance.client.grpc.PbClient
import io.provenance.client.protobuf.extensions.toAny
import io.provenance.client.wallet.WalletSigner
import io.provenance.name.v1.QueryResolveRequest
import java.math.BigDecimal

private val DEFAULT_BROADCAST_MODE = BroadcastMode.BROADCAST_MODE_BLOCK
private const val DEFAULT_GAS_ADJUSTMENT: Double = 2.0

/**
 * A helper function to execute a single transaction against the blockchain.
 *
 * @param signer The WalletSigner instance that indicates the account executing the transaction.
 * @param transaction Any protobuf Message compatible with Provenance and/or the Cosmos SDK.
 * @param broadcastMode The mode in which to broadcast each transaction. See the enum itself for descriptions. The
 *                      examples in this project are not guaranteed to work correctly when using other broadcast modes,
 *                      due to the need to wait for the transaction to successfully execute before proceeding.
 * @param feeGranter    The address of an account authorized to pay fees for the transaction, other than the account
 *                      contained in the provided signer param.  This is an authz feature that allows gas and message
 *                      fees to be paid by proxy when the transaction is emitted.
 */
fun PbClient.executeTx(
    signer: WalletSigner,
    transaction: Message,
    broadcastMode: BroadcastMode = DEFAULT_BROADCAST_MODE,
    gasAdjustment: Double = DEFAULT_GAS_ADJUSTMENT,
    feeGranter: String? = null,
): BroadcastTxResponse = this.estimateAndBroadcastTx(
    txBody = TxBody.newBuilder().addMessages(transaction.toAny()).build(),
    signers = BaseReqSigner(signer = signer).let(::listOf),
    mode = broadcastMode,
    gasAdjustment = gasAdjustment,
    feeGranter = feeGranter,
).also { response ->
    // A non-zero response code indicates a failure of some sort.  The rawLog contains hints and information about the
    // failure in this scenario. Emit the message as an exception to reveal the issue
    if (response.txResponse.code != 0) {
        throw IllegalStateException("Failed to execute transaction(s) and got response: ${response.txResponse.rawLog}")
    }
}

/**
 * A helper function to take a Provenance Name module name and resolve it to its bound address.
 */
fun PbClient.resolveName(name: String): String = nameClient.resolve(
    QueryResolveRequest.newBuilder().setName(name).build()
).address

/**
 * A helper function to determine how much of a specific denomination of coin a specified bech32 address currently owns.
 * Converts to a BigDecimal to provide a numeric representation of the balance amount.  64bit numeric types like Long
 * can potentially be too small to hold the entire balance of an account.
 */
fun PbClient.queryBalance(address: String, denom: String): BigDecimal = bankClient.balance(
    QueryBalanceRequest.newBuilder().setAddress(address).setDenom(denom).build()
).balance.amount.toBigDecimal()
