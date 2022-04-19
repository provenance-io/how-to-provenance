package io.provenance.example.examples

import cosmos.base.v1beta1.CoinOuterClass.Coin
import cosmos.feegrant.v1beta1.Feegrant.BasicAllowance
import cosmos.feegrant.v1beta1.QueryOuterClass.QueryAllowanceRequest
import cosmos.feegrant.v1beta1.Tx.MsgGrantAllowance
import cosmos.feegrant.v1beta1.Tx.MsgRevokeAllowance
import io.provenance.client.grpc.PbClient
import io.provenance.client.protobuf.extensions.toAny
import io.provenance.client.wallet.WalletSigner
import io.provenance.example.util.DefaultParam
import io.provenance.example.util.InputParams
import io.provenance.example.util.InputUtil.input
import io.provenance.example.util.InputUtil.inputString
import io.provenance.example.util.InputValidation
import io.provenance.example.util.PbClientUtil
import io.provenance.example.util.WalletSignerUtil
import io.provenance.example.util.executeTx
import io.provenance.example.util.queryBalance
import io.provenance.example.util.resolveName
import io.provenance.name.v1.MsgBindNameRequest
import io.provenance.name.v1.NameRecord

/**
 * This example showcases how to use the authz module to use a secondary account ("helper") to pay fees for a different
 * account ("main").  It does the following:
 * - Grants an allowance of 10 billion nhash from the "helper" account to the "main" account for executing messages.
 * - Binds a new name to the "main" account and uses the "helper" account to pay all fees.
 * - Revokes the fee grant from the "helper" account to the "main" as cleanup after all authz actions are completed.
 */
object FeeGrantExample : ExampleSuite {
    override fun start() {
        val pbClient = PbClientUtil.newClient()
        val mainAccount = WalletSignerUtil.newSigner("[Main Account]")
        println("Main account established with address [${mainAccount.address()}]")
        val helperAccount = WalletSignerUtil.newSigner("[Helper Account]")
        println("Helper account established with address [${helperAccount.address()}]")
        // First, enable the main account to pay its fees using the helper account
        grantFeeToMainAccount(pbClient, mainAccount, helperAccount)
        // Second, bind a name to the main account, using the helper account to pay fees after the grant was made
        bindNameWithFeeGrant(pbClient, mainAccount, helperAccount)
        // Finally, cleanup the fee grant by revoking it
        revokeFeeGrantToMainAccount(pbClient, mainAccount, helperAccount)
    }

    /**
     * Uses Authz to grant the mainAccount access to use the helperAccount to pay fees on its behalf by emitting a
     * MsgGrantAllowance request in a transaction.
     */
    private fun grantFeeToMainAccount(
        pbClient: PbClient,
        mainAccount: WalletSigner,
        helperAccount: WalletSigner,
    ) {
        val feeGrantAmount = input(
            message = "Enter an amount of nhash to grant from ${helperAccount.address()}",
            params = InputParams(
                // Default to 10 billion. This is much more than what is needed, but shows that very large amounts
                // can be specified without issue
                default = DefaultParam(10_000_000_000L)
            ),
            converter = { it.toLongOrNull() }
        )
        println("Establishing a grant for ${feeGrantAmount}nhash from [${helperAccount.address()}] to [${mainAccount.address()}]")
        try {
            pbClient.executeTx(
                signer = helperAccount,
                transaction = MsgGrantAllowance.newBuilder()
                    // Each fee grant allowance type is specified in this file: https://github.com/provenance-io/provenance/blob/main/third_party/proto/cosmos/feegrant/v1beta1/feegrant.proto
                    .setAllowance(
                        BasicAllowance
                            .newBuilder()
                            .addSpendLimit(Coin.newBuilder().setAmount(feeGrantAmount.toString()).setDenom("nhash"))
                            .build()
                            .toAny()
                    )
                    // Granter = account that is granting access and can be used as a proxy for fees
                    .setGranter(helperAccount.address())
                    // Grantee = account that is authorized to use the granter for fee payments
                    .setGrantee(mainAccount.address())
                    .build(),
            )
            // After successfully establishing the grant, it can be queried with the feegrantClient.
            // This sanity check will ensure that the grant was successful
            val grant = pbClient.feegrantClient.allowance(
                QueryAllowanceRequest.newBuilder()
                    .setGrantee(mainAccount.address())
                    .setGranter(helperAccount.address())
                    .build()
            )
            // Multiple different grant types exist, so the allowance itself is serialized as an Any.  However, it was just
            // created in the preceding lines, so it can be cast to a BasicAllowance without fear of cast exceptions
            val basicAllowance = grant.allowance.allowance.unpack(BasicAllowance::class.java)
            println("Grant to account [${mainAccount.address()}] from account [${helperAccount.address()}] was successfully made as basic allowance: $basicAllowance")
        } catch (e: Exception) {
            println("Failed to add fee grant!")
            e.printStackTrace()
        }
    }

    /**
     * Binds a name to the mainAccount's address, using the helperAccount as the feePayer.  This shows that the authz
     * grant from grantFeeToMainAccount has successfully enabled the helperAccount to pay on behalf of the mainAccount.
     */
    private fun bindNameWithFeeGrant(
        pbClient: PbClient,
        mainAccount: WalletSigner,
        helperAccount: WalletSigner,
    ) {
        println("Binding a new name on Provenance for account [${mainAccount.address()}] using [${helperAccount.address()}] to pay fees")
        val inputPrefix = "[Name Registration]"
        val childName = inputString(
            message = "$inputPrefix Enter a name to bind (alpha characters only)",
            params = InputParams(
                validation = InputValidation(
                    validate = { namePrefix -> namePrefix.all { it.isLetter() } },
                    validationRuleText = listOf("Input must be alphabetical characters only")
                ),
                default = DefaultParam("testname"),
            ),
        )
        val parentName = inputString(
            message = "$inputPrefix Enter the existing parent name to bind to (unrestricted)",
            params = InputParams(
                validation = InputValidation(
                    validate = { namePrefix -> namePrefix.all { it.isLetter() } },
                    validationRuleText = listOf("Input must be alphabetical characters only")
                ),
                // The name "pio" is created unrestricted on local instances.  If using testnet or another instance,
                // be sure to use an unrestricted name, or execution will fail when attempting to find the parent address
                // owner of the restricted name as a signer
                default = DefaultParam("pio"),
            )
        )
        val restrict = input(
            message = "$inputPrefix Restrict name? (true/false)",
            params = InputParams(default = DefaultParam(true)),
            converter = { it.toBooleanStrictOrNull() },
        )
        val fullName = "${childName}.${parentName}"
        println("Binding name [$fullName] to [${mainAccount.address()}] with [restrict = $restrict]")
        try {
            // Print balances prior to executing the transaction in order to showcase that the mainAccount does not
            // actually send any funds when using the helperAccount as its fee payer
            val mainBalanceBeforeTx = pbClient.queryBalance(address = mainAccount.address(), denom = "nhash")
            val helperBalanceBeforeTx = pbClient.queryBalance(address = helperAccount.address(), denom = "nhash")
            println("[Before Name Bind] Main account [${mainAccount.address()}] nhash = $mainBalanceBeforeTx | Helper account [${helperAccount.address()}] nhash = $helperBalanceBeforeTx")
            pbClient.executeTx(
                signer = mainAccount,
                transaction = MsgBindNameRequest.newBuilder()
                    // When the parent name is unrestricted, the address of the account being used to bind the child
                    // name should be used as the address for the parent.  However, if the parent name is restricted,
                    // the owner of the parent name must be set in the address in the parent NameRecord, and the parent must
                    // be included as a signer.
                    .setParent(NameRecord.newBuilder().setAddress(mainAccount.address()).setName(parentName).build())
                    .setRecord(
                        NameRecord.newBuilder()
                            .setAddress(mainAccount.address())
                            .setName(childName)
                            .setRestricted(restrict)
                            .build()
                    )
                    .build(),
                // This parameter allows the helperAccount to cover gas and message fees for the sender (mainAccount).
                // If the fee grant in grantFeeToMainAccount() had not been made, including the helper account's address
                // in the transaction as a fee granter would fail because the main account has not been enabled for this
                // via the authz module
                feeGranter = helperAccount.address(),
            )
            val nameAddress = pbClient.resolveName(fullName)
            if (nameAddress != mainAccount.address()) {
                throw IllegalStateException("Name [$fullName] was bound to address [$nameAddress], not to the expected address of [${mainAccount.address()}]")
            }
            println("Successfully bound name [$fullName] to address [${mainAccount.address()}]")
            val mainBalanceAfterTx = pbClient.queryBalance(address = mainAccount.address(), denom = "nhash")
            val helperBalanceAfterTx = pbClient.queryBalance(address = helperAccount.address(), denom = "nhash")
            // The mainAccount should show 0nhash spent, and the helperAccount should show the entire cost of binding
            // a name: gas fees + message fees
            println("[After Name Bind] Main account [${mainAccount.address()}] nhash = $mainBalanceAfterTx (Spent = ${mainBalanceBeforeTx - mainBalanceAfterTx}) | Helper account [${helperAccount.address()}] nhash = $helperBalanceAfterTx (Spent = ${helperBalanceBeforeTx - helperBalanceAfterTx})")
        } catch (e: Exception) {
            println("Failed to bind name [$fullName] to address [${mainAccount.address()}]")
            e.printStackTrace()
        }
    }

    /**
     * A simple cleanup step that showcases how to revoke access granted via authz when it is no longer desired or
     * needed.  Revokes the authz grant created in grantFeeToMainAccount.
     */
    private fun revokeFeeGrantToMainAccount(
        pbClient: PbClient,
        mainAccount: WalletSigner,
        helperAccount: WalletSigner,
    ) {
        println("Revoking fee grant from [${helperAccount.address()}] to [${mainAccount.address()}]")
        try {
            pbClient.executeTx(
                signer = helperAccount,
                transaction = MsgRevokeAllowance.newBuilder()
                    .setGrantee(mainAccount.address())
                    .setGranter(helperAccount.address())
                    .build(),
            )
            println("Successfully revoked fee grant from [${helperAccount.address()}] to [${mainAccount.address()}]")
        } catch (e: Exception) {
            println("Failed to revoke fee grant!")
            e.printStackTrace()
        }
    }
}
