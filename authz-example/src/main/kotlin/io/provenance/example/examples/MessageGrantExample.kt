package io.provenance.example.examples

import cosmos.authz.v1beta1.Authz.GenericAuthorization
import cosmos.authz.v1beta1.Authz.Grant
import cosmos.authz.v1beta1.Tx.MsgGrant
import cosmos.authz.v1beta1.Tx.MsgRevoke
import io.provenance.client.grpc.PbClient
import io.provenance.client.protobuf.extensions.toAny
import io.provenance.client.wallet.WalletSigner
import io.provenance.example.util.PbClientUtil
import io.provenance.example.util.WalletSignerUtil
import io.provenance.example.util.executeTx
import io.provenance.metadata.v1.Description
import io.provenance.metadata.v1.MsgWriteScopeSpecificationRequest
import io.provenance.metadata.v1.PartyType
import io.provenance.metadata.v1.ScopeSpecification
import io.provenance.metadata.v1.ScopeSpecificationRequest
import io.provenance.scope.util.toProtoTimestamp
import java.time.OffsetDateTime
import java.util.UUID

/**
 * This example showcases how to use the authz module to grant privileges to modify ownership of a metadata module
 * object from the owner account (helper account) to another account (main account).  It does the following:
 * - Creates a scope specification owned by the helper account.
 * - Grants the main account authz privileges to execute MsgWriteScopeSpecificationRequest transactions on behalf of the helper account.
 * - Alters the ownership of the scope specification to the main account using only the main account's signature.
 * - Revokes the authz message grant from the main account as a cleanup step after all authz actions are completed.
 */
object MessageGrantExample : ExampleSuite {
    override fun start() {
        val pbClient = PbClientUtil.newClient()
        val mainAccount = WalletSignerUtil.newSigner("[Main Account]")
        println("Main account established with address [${mainAccount.address()}]")
        val helperAccount = WalletSignerUtil.newSigner("[Helper Account]")
        println("Helper account established with address [${helperAccount.address()}]")
        // First, create a scope specification owned by the helper account
        val scopeSpecUuid = createScopeSpecification(pbClient, helperAccount)
        // Second, grant the main account authz privileges to run MsgWriteScopeSpecificationRequest messages on behalf
        // of the helper account
        grantMessageExecuteToMainAccount(pbClient, mainAccount, helperAccount)
        // Third, have the main account take ownership of the scope specification by re-writing it with itself as the
        // owner, and signing without any intervention from the helper account
        writeScopeSpecOwnerToMainAccount(scopeSpecUuid, pbClient, mainAccount, helperAccount)
        // Finally, revoke MsgWriteScopeSpecificationRequest authz privileges from the main account via the helper
        // account as a cleanup step
        revokeGrantToMainAccount(pbClient, mainAccount, helperAccount)
    }

    /**
     * A simple demonstration of the creation of a scope specification.  This, alone, does relatively nothing in the
     * provenance ecosystem.  However, it does create a metadata object owned solely by the helper account.  This
     * ownership cannot be changed without either including the helper account as a signer, or by using authz to
     * delegate those permissions to another account.
     */
    private fun createScopeSpecification(
        pbClient: PbClient,
        helperAccount: WalletSigner,
    ): UUID = UUID.randomUUID().also { scopeSpecUuid ->
        println("Writing new scope spec with uuid [${helperAccount.address()}] to be owned by helper account [${helperAccount.address()}]")
        try {
            pbClient.executeTx(
                signer = helperAccount,
                transaction = MsgWriteScopeSpecificationRequest
                    .newBuilder()
                    .addSigners(helperAccount.address())
                    .setSpecUuid(scopeSpecUuid.toString())
                    .setSpecification(
                        ScopeSpecification
                            .newBuilder()
                            .setDescription(
                                Description.newBuilder()
                                    .setName("fake-scope-spec")
                                    .setDescription("a fake scope specification")
                            )
                            .addOwnerAddresses(helperAccount.address())
                            .addPartiesInvolved(PartyType.PARTY_TYPE_OWNER)
                    )
                    .build(),
            )
            println("Successfully created scope specification with uuid [$scopeSpecUuid] owned by [${helperAccount.address()}]")
        } catch (e: Exception) {
            println("Failed to create scope spec for helper account [${helperAccount.address()}]")
            e.printStackTrace()
        }
    }

    /**
     * This function uses authz to authorize the main account to execute MsgWriteScopeSpecificationRequests on behalf
     * of the helper account.
     */
    private fun grantMessageExecuteToMainAccount(
        pbClient: PbClient,
        mainAccount: WalletSigner,
        helperAccount: WalletSigner,
    ) {
        println("Granting MsgWriteScopeSpecificationRequest authority to [${mainAccount.address()}] from [${helperAccount.address()}]")
        try {
            pbClient.executeTx(
                signer = helperAccount,
                transaction = MsgGrant.newBuilder()
                    .setGrant(
                        Grant
                            .newBuilder()
                            // Provenance consumes GenericAuthorization here: https://github.com/provenance-io/provenance/blob/main/third_party/proto/cosmos/authz/v1beta1/authz.proto
                            // A generic authorization can wrap any message type and dynamically grant access, if authz
                            // is enabled for that particular message type.  Provenance leverages authz for message-type
                            // grants in order to allow metadata changes
                            .setAuthorization(
                                GenericAuthorization
                                    .newBuilder()
                                    .setMsg("/${MsgWriteScopeSpecificationRequest.getDescriptor().fullName}")
                                    .build()
                                    .toAny()
                            )
                            // For message-based authz grants, an expiration window is required.  To ensure that no
                            // issues exist when running this example, creating a one-day window of time is more than
                            // sufficient.
                            .setExpiration(OffsetDateTime.now().plusDays(1).toProtoTimestamp())
                            .build()
                    )
                    // Grant this MsgWriteScopeSpecificationRequest access to the main account
                    .setGrantee(mainAccount.address())
                    // Grant MsgWriteScopeSpecificationRequest on behalf of the helper account
                    .setGranter(helperAccount.address())
                    .build()
            )
            println("Successfully granted MsgWriteScopeSpecificationRequest authority to [${mainAccount.address()}]")
        } catch (e: Exception) {
            println("Failed to grant MsgWriteScopeSpecificationRequest authority to [${mainAccount.address()}] from [${helperAccount.address()}]")
            e.printStackTrace()
        }
    }

    /**
     * This function uses the main account's new authz permissions to change the owner of the previously-created scope
     * specification from the helper account to the main account.  By way of authz grant, the main account will be able
     * to sign a message alone that achieves this.  Without the authz grant, this function would fail.  To demonstrate
     * this, comment out the usage of grantMessageExecuteToMainAccount() in the start() function.
     */
    private fun writeScopeSpecOwnerToMainAccount(
        scopeSpecUuid: UUID,
        pbClient: PbClient,
        mainAccount: WalletSigner,
        helperAccount: WalletSigner,
    ) {
        try {
            println("Re-writing scope specification with uuid [$scopeSpecUuid] to swap owner from helper account [${helperAccount.address()}] to main account [${mainAccount.address()}]")
            pbClient.executeTx(
                signer = mainAccount,
                transaction = MsgWriteScopeSpecificationRequest
                    .newBuilder()
                    // Although the helperAccount owns the scope specification, the authz grant will allow the
                    // mainAccount to take ownership without a signature from the scope spec owner.  This would be
                    // rejected without the grant because the helperAccount would be required as a signer.
                    .addSigners(mainAccount.address())
                    .setSpecUuid(scopeSpecUuid.toString())
                    .setSpecification(
                        ScopeSpecification
                            .newBuilder()
                            .setDescription(
                                Description.newBuilder()
                                    .setName("fake-scope-spec")
                                    .setDescription("a fake scope specification")
                            )
                            .addOwnerAddresses(mainAccount.address())
                            .addPartiesInvolved(PartyType.PARTY_TYPE_OWNER)
                    )
                    .build(),
            )
            println("Successfully executed transaction to set the main account [${mainAccount.address()}] as the owner of the scope spec [$scopeSpecUuid]")
            // Query up the scope specification to fully verify that the changes have been recorded on the blockchain.
            val scopeSpec = pbClient.metadataClient.scopeSpecification(
                ScopeSpecificationRequest.newBuilder().setSpecificationId(scopeSpecUuid.toString()).build()
            ).scopeSpecification.specification
            check(scopeSpec.ownerAddressesList.size == 1 && mainAccount.address() in scopeSpec.ownerAddressesList) {
                "Expected only one owner to be listed on scope [$scopeSpecUuid] and for the owner to be the main account [${mainAccount.address()}]. " +
                        "Instead, found owner addresses: ${scopeSpec.ownerAddressesList}"
            }
            println("Success! Scope specification [$scopeSpecUuid] is now solely owned by the main account [${mainAccount.address()}]")
        } catch (e: Exception) {
            println("Failed to set main account [${mainAccount.address()}] as the owner of scope spec [$scopeSpecUuid]")
            e.printStackTrace()
        }
    }

    /**
     * A simple cleanup step that showcases how to revoke an authz grant for a message type.  Revokes the
     * MsgWriteScopeSpecificationRequest authorization granted during the execution of the
     * grantMessageExecuteToMainAccount() function.
     */
    private fun revokeGrantToMainAccount(
        pbClient: PbClient,
        mainAccount: WalletSigner,
        helperAccount: WalletSigner,
    ) {
        println("Revoking MsgWriteScopeSpecificationRequest grant from [${helperAccount.address()}] to [${mainAccount.address()}]")
        try {
            pbClient.executeTx(
                signer = helperAccount,
                transaction = MsgRevoke.newBuilder()
                    .setGrantee(mainAccount.address())
                    .setGranter(helperAccount.address())
                    .setMsgTypeUrl("/${MsgWriteScopeSpecificationRequest.getDescriptor().fullName}")
                    .build(),
            )
            println("Successfully revoked MsgWriteScopeSpecificationRequest authorization from [${helperAccount.address()}] to [${mainAccount.address()}]")
        } catch (e: Exception) {
            println("Failed to revoke MsgWriteScopeSpecificationRequest authorization!")
            e.printStackTrace()
        }
    }
}
