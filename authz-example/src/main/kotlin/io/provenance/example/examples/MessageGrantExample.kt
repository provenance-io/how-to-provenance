package io.provenance.example.examples

import cosmos.authz.v1beta1.Authz.GenericAuthorization
import cosmos.authz.v1beta1.Authz.Grant
import cosmos.authz.v1beta1.Tx.MsgExec
import cosmos.authz.v1beta1.Tx.MsgGrant
import cosmos.authz.v1beta1.Tx.MsgRevoke
import cosmos.bank.v1beta1.Tx.MsgSend
import io.provenance.client.grpc.PbClient
import io.provenance.client.protobuf.extensions.toAny
import io.provenance.client.wallet.WalletSigner
import io.provenance.example.util.DefaultParam
import io.provenance.example.util.InputParams
import io.provenance.example.util.InputUtil.input
import io.provenance.example.util.PbClientType
import io.provenance.example.util.PbClientUtil
import io.provenance.example.util.ProtoBuilderUtil.coin
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
        revokeMsgSendGrantToMainAccount(pbClient, mainAccount, helperAccount)
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
                transaction = MsgGrant.newBuilder()
                    .setGrant(
                        Grant
                            .newBuilder()
                            // All Provenance authorization types are found at: https://github.com/provenance-io/provenance/blob/main/third_party/proto/cosmos/authz/v1beta1/authz.proto
                            // A generic authorization can wrap any message type and dynamically grant access, if authz
                            // is enabled for that particular message type.  This authorization could also be achieved
                            // by directly using the proper authorization message. For MsgSend, the message would be a
                            // SendAuthorization: https://github.com/cosmos/cosmos-sdk/blob/master/proto/cosmos/bank/v1beta1/authz.proto
                            // The upside to adding authorizations via direct specification is that a spend limit could
                            // also be appended if using that approach.
                            .setAuthorization(
                                GenericAuthorization
                                    .newBuilder()
                                    .setMsg("/${MsgSend.getDescriptor().fullName}")
                                    .build()
                                    .toAny()
                            )
                            // For message-based authz grants, an expiration window is required.  To ensure that no
                            // issues exist when running this example, creating a one-day window of time is more than
                            // sufficient.
                            .setExpiration(OffsetDateTime.now().plusDays(1).toProtoTimestamp())
                            .build()
                    )
                    // Grant this MsgSend access to the main account
                    .setGrantee(mainAccount.address())
                    // Grant MsgSends on behalf of the helper account
                    .setGranter(helperAccount.address())
                    .build()
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
        val mainBalanceBeforeTransfer = pbClient.queryBalance(mainAccount.address(), "nhash")
        val helperBalanceBeforeTransfer = pbClient.queryBalance(helperAccount.address(), "nhash")
        println("Main Account Balance: ${mainBalanceBeforeTransfer}nhash | Helper Account Balance: ${helperBalanceBeforeTransfer}nhash")
        // Prompt how much nhash to take from helperAccount to send to mainAccount. Default to 100nhash for a fairly
        // low-impact transfer
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
                // Important: In order to do actions that would normally not be possible, like a MsgSend that transfers
                // funds from the helperAccount to the mainAccount while only having the mainAccount sign, the transaction
                // message must be wrapped in a MsgExec, which lets the server know that the request will be using
                // authz permissions to perform the action.
                transaction = MsgExec
                    .newBuilder()
                    // Establish the grantee as the mainAccount, denoting that it will perform actions authorized by
                    // the authz module
                    .setGrantee(mainAccount.address())
                    .addMsgs(
                        MsgSend.newBuilder()
                            // Transfer the specified amount of nhash from input
                            .addAmount(coin(amountToSend, "nhash"))
                            // Set the helperAccount as the sender.  This would normally require that the helperAccount
                            // is specified as the signer, but authz is helping this not be necessary because the
                            // helperAccount granted the mainAccount authorization to do this
                            .setFromAddress(helperAccount.address())
                            // Set the mainAccount as the receiving address for the transfer
                            .setToAddress(mainAccount.address())
                            .build()
                            .toAny()
                    )
                    .build(),
            )
            println("Successfully executed transaction to send ${amountToSend}nhash from [${helperAccount.address()}] to [${mainAccount.address()}]")
            val mainBalanceAfterTransfer = pbClient.queryBalance(mainAccount.address(), "nhash")
            val helperBalanceAfterTransfer = pbClient.queryBalance(helperAccount.address(), "nhash")
            println("Main Account Balance: ${mainBalanceAfterTransfer}nhash | Helper Account Balance: ${helperBalanceAfterTransfer}nhash")
            // The helper did not spend any gas fees for the transaction because the main account was the signer.
            // This allows this check to determine if the helper's balance was directly reduced by the sent amount
            check(helperBalanceBeforeTransfer - helperBalanceAfterTransfer == amountToSend) {
                "Expected ${amountToSend}nhash to be sent from the helper account [${helperAccount.address()}] to the main account [${mainAccount.address()}]"
            }
            println("Successfully sent ${amountToSend}nhash from [${helperAccount.address()}] to [${mainAccount.address()}]")
        } catch (e: Exception) {
            println("Failed to send ${amountToSend}nhash from [${helperAccount.address()}] to [${mainAccount.address()}]")
            e.printStackTrace()
        }
    }

    private fun revokeMsgSendGrantToMainAccount(
        pbClient: PbClient,
        mainAccount: WalletSigner,
        helperAccount: WalletSigner,
    ) {
        println("Revoking MsgSend grant from [${helperAccount.address()}] to [${mainAccount.address()}]")
        try {
            pbClient.executeTx(
                signer = helperAccount,
                transaction = MsgRevoke.newBuilder()
                    .setGrantee(mainAccount.address())
                    .setGranter(helperAccount.address())
                    .setMsgTypeUrl("/${MsgSend.getDescriptor().fullName}")
                    .build(),
            )
            println("Successfully revoked MsgSend authorization from [${helperAccount.address()}] to [${mainAccount.address()}]")
        } catch (e: Exception) {
            println("Failed to revoke MsgSend authorization!")
            e.printStackTrace()
        }
    }
}
