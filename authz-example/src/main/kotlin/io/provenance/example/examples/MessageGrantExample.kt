package io.provenance.example.examples

import cosmos.authz.v1beta1.Authz.GenericAuthorization
import cosmos.authz.v1beta1.Authz.Grant
import cosmos.authz.v1beta1.Tx.MsgGrant
import cosmos.bank.v1beta1.Tx.MsgSend
import cosmos.base.v1beta1.CoinOuterClass
import ibc.applications.transfer.v1.Tx.MsgTransfer
import io.provenance.client.grpc.PbClient
import io.provenance.client.protobuf.extensions.toAny
import io.provenance.client.wallet.WalletSigner
import io.provenance.example.util.DefaultParam
import io.provenance.example.util.InputParams
import io.provenance.example.util.InputUtil.input
import io.provenance.example.util.PbClientType
import io.provenance.example.util.PbClientUtil
import io.provenance.example.util.WalletSignerUtil
import io.provenance.example.util.executeTx
import io.provenance.example.util.queryBalance
import io.provenance.scope.util.toProtoTimestamp
import java.time.OffsetDateTime

object MessageGrantExample : ExampleSuite {
    override fun start() {
        val pbClient = PbClientUtil.newClient(clientType = PbClientType.LOCAL)
        val mainAccount = WalletSignerUtil.newSigner("[Main Account]", mnemonic = "develop cycle wedding text knock approve arm flame grace razor armor buyer fringe idle knock spell check hockey stamp vivid sail food ordinary maid")
        println("Main account established with address [${mainAccount.address()}]")
        val helperAccount = WalletSignerUtil.newSigner("[Helper Account]", mnemonic = "earn attend jar milk open poverty inject park flock face bonus check climb illness test emerge giraffe weather castle shoot robust try creek pause")
        println("Helper account established with address [${helperAccount.address()}]")
        grantMessageExecuteToMainAccount(pbClient, mainAccount, helperAccount)
        sendFundsFromHelperToMain(pbClient, mainAccount, helperAccount)
    }

    private fun grantMessageExecuteToMainAccount(
        pbClient: PbClient,
        mainAccount: WalletSigner,
        helperAccount: WalletSigner,
    ) {
        println("Granting MsgSend authority to [${mainAccount.address()}] from [${helperAccount.address()}]")
        try {
            pbClient.executeTx(
                signer = helperAccount,
                transactions = MsgGrant.newBuilder()
                    .setGrant(
                        Grant
                            .newBuilder()
                                // All authorization types are found at: https://github.com/provenance-io/provenance/blob/main/third_party/proto/cosmos/authz/v1beta1/authz.proto
                            .setAuthorization(
                                GenericAuthorization
                                    .newBuilder()
                                    // The Msg qualifier can be reliably derived using its descriptor.  Another option
                                    // is to use the toAny() extension function and just get the typeUrl, but this
                                    // approach should be less impacting to app performance
                                    .setMsg("/${MsgSend.getDescriptor().fullName}")
                                    .build()
                                    .toAny()
                            )
                            .setExpiration(OffsetDateTime.now().plusDays(1).toProtoTimestamp())
                            .build()
                    )
                    .setGrantee(mainAccount.address())
                    .setGranter(helperAccount.address())
                    .build()
                    .let(::listOf)
            )
            println("Successfully granted MsgSend authority to [${mainAccount.address()}]")
        } catch (e: Exception) {
            println("Failed to grant MsgSend authority to [${mainAccount.address()}] from [${helperAccount.address()}]")
            e.printStackTrace()
        }
    }

    private fun sendFundsFromHelperToMain(
        pbClient: PbClient,
        mainAccount: WalletSigner,
        helperAccount: WalletSigner,
    ) {
        // Prompt how much nhash to take from helperAccount to send to mainAccount. Default to 100nhash for a fairly
        // low-impact transfer
        val mainBalanceBeforeTransfer = pbClient.queryBalance(mainAccount.address(), "nhash")
        val helperBalanceBeforeTransfer = pbClient.queryBalance(helperAccount.address(), "nhash")
        println("Main Account Balance: ${mainBalanceBeforeTransfer}nhash | Helper Account Balance: ${helperBalanceBeforeTransfer}nhash")
        val amountToSend = input(
            message = "Amount of nhash to send [${mainAccount.address()}] from [${helperAccount.address()}] (Max: ${helperBalanceBeforeTransfer}nhash)",
            params = InputParams(
                // Don't let the default ever breach the amount of nhash the helper account has
                default = DefaultParam(100L.coerceAtMost(helperBalanceBeforeTransfer)),
                // Don't accept input if it attempts to send more than the amount of nhash the helper account has
                validation = { amountToSend -> amountToSend <= helperBalanceBeforeTransfer },
            ),
            converter = { it.toLongOrNull() },
        )
        try {
            println("Sending ${amountToSend}nhash from [${helperAccount.address()}] to [${mainAccount.address()}]")
            pbClient.executeTx(
                signer = mainAccount,
                transactions = MsgSend.newBuilder()
                    .addAmount(CoinOuterClass.Coin.newBuilder().setAmount(amountToSend.toString()).setDenom("nhash"))
                    .setFromAddress(helperAccount.address())
                    .setToAddress(mainAccount.address())
                    .build()
                    .let(::listOf),
            )
            println("Successfully executed transaction to send ${amountToSend}nhash from [${helperAccount.address()}] to [${mainAccount.address()}]")
            val mainBalanceAfterTransfer = pbClient.queryBalance(mainAccount.address(), "nhash")
            val helperBalanceAfterTransfer = pbClient.queryBalance(helperAccount.address(), "nhash")
            println("Main Account Balance: ${mainBalanceAfterTransfer}nhash | Helper Account Balance: ${helperBalanceAfterTransfer}nhash")
            check(helperBalanceBeforeTransfer - helperBalanceAfterTransfer == amountToSend) {
                "Expected ${amountToSend}nhash to be sent from the helper account [${helperAccount.address()}] to the main account [${mainAccount.address()}]"
            }
            println("Successfully sent ${amountToSend}nhash from [${helperAccount.address()}] to [${mainAccount.address()}]")
        } catch (e: Exception) {
            println("Failed to send ${amountToSend}nhash from [${helperAccount.address()}] to [${mainAccount.address()}]")
            e.printStackTrace()
        }
    }
}
