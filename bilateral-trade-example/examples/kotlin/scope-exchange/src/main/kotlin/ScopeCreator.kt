import cosmos.tx.v1beta1.ServiceOuterClass
import cosmos.tx.v1beta1.TxOuterClass
import io.provenance.client.grpc.BaseReqSigner
import io.provenance.client.grpc.PbClient
import io.provenance.client.protobuf.extensions.getBaseAccount
import io.provenance.metadata.v1.MsgWriteScopeRequest
import io.provenance.metadata.v1.Party
import io.provenance.metadata.v1.PartyType
import io.provenance.metadata.v1.ScopeRequest
import io.provenance.metadata.v1.ScopeResponse

class ScopeCreator(
    private val pbClient: PbClient,
) {
    fun createScope(scopeUuid: String, scopeSpecUuid: String, owner: WalletSigner): ScopeResponse {
        println("Creating scope")
        val createScopeMsg = MsgWriteScopeRequest.newBuilder()
            .setScopeUuid(scopeUuid)
            .setSpecUuid(scopeSpecUuid)
            .addSigners(owner.address())
            .apply {
                scopeBuilder.setValueOwnerAddress(owner.address())
                    .addOwners(
                        Party.newBuilder()
                        .setRole(PartyType.PARTY_TYPE_OWNER)
                        .setAddress(owner.address())
                    )
            }
            .build().toAny()

        val ownerAccount = pbClient.authClient.getBaseAccount(owner.address())

        pbClient.estimateAndBroadcastTx(
            TxOuterClass.TxBody.newBuilder().addMessages(createScopeMsg).build(), listOf(
            BaseReqSigner(owner, 0, ownerAccount)
        ), ServiceOuterClass.BroadcastMode.BROADCAST_MODE_BLOCK, gasAdjustment = 1.5).also {
            if (it.txResponse.code != 0) {
                throw Exception("Error creating scope, code: ${it.txResponse.code}, message: ${it.txResponse.rawLog}")
            }

            println("Scope created successfully: ${it.txResponse.txhash}")
        }

        return pbClient.metadataClient.scope(ScopeRequest.newBuilder().setScopeId(scopeUuid).build())
    }
}
