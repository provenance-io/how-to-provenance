package io.provenance.example.util



import cosmos.base.v1beta1.CoinOuterClass
import cosmos.tx.v1beta1.ServiceOuterClass
import cosmos.tx.v1beta1.TxOuterClass
import io.provenance.client.grpc.BaseReqSigner
import io.provenance.client.grpc.PbClient
import io.provenance.client.protobuf.extensions.getBaseAccount
import io.provenance.client.protobuf.extensions.toAny
import io.provenance.client.wallet.WalletSigner
import io.provenance.marker.v1.Access
import io.provenance.marker.v1.AccessGrant
import io.provenance.marker.v1.MarkerAccount
import io.provenance.marker.v1.MarkerStatus
import io.provenance.marker.v1.MarkerType
import io.provenance.marker.v1.MsgActivateRequest
import io.provenance.marker.v1.MsgAddMarkerRequest
import io.provenance.marker.v1.MsgWithdrawRequest
import io.provenance.marker.v1.QueryMarkerRequest

class MarkerCreator(
    private val pbClient: PbClient
) {
    fun createMarker(shares: Int, denom: String, owner: WalletSigner): MarkerAccount {
        val createMarker = MsgAddMarkerRequest.newBuilder()
            .setMarkerType(MarkerType.MARKER_TYPE_COIN)
            .setAmount(CoinOuterClass.Coin.newBuilder().setAmount(shares.toString()).setDenom(denom))
            .setFromAddress(owner.address())
            .setAllowGovernanceControl(false)
            .setManager(owner.address())
            .setSupplyFixed(true)
            .addAccessList(AccessGrant.newBuilder()
                .setAddress(owner.address())
                .addAllPermissions(listOf(
                    Access.ACCESS_ADMIN,
                    Access.ACCESS_WITHDRAW,
                    Access.ACCESS_DEPOSIT,
                ))
            )
            .setStatus(MarkerStatus.MARKER_STATUS_FINALIZED)
            .build().toAny()

        val activateMarker = MsgActivateRequest.newBuilder()
            .setDenom(denom)
            .setAdministrator(owner.address())
            .build().toAny()

        val transferCoin = MsgWithdrawRequest.newBuilder()
            .setDenom(denom)
            .addAmount(CoinOuterClass.Coin.newBuilder()
                .setAmount(shares.toString())
                .setDenom(denom)
            )
            .setToAddress(owner.address())
            .setAdministrator(owner.address())
            .build().toAny()

        val ownerAccount = pbClient.authClient.getBaseAccount(owner.address())

        pbClient.estimateAndBroadcastTx(
            TxOuterClass.TxBody.newBuilder().addAllMessages(listOf(createMarker, activateMarker, transferCoin)).build(), listOf(
                BaseReqSigner(owner, 0, ownerAccount)
            ), ServiceOuterClass.BroadcastMode.BROADCAST_MODE_BLOCK, gasAdjustment = 1.5).also {
            if (it.txResponse.code != 0) {
                throw Exception("Error creating marker, code: ${it.txResponse.code}, message: ${it.txResponse.rawLog}")
            }

            println("Marker created successfully: ${it.txResponse.txhash}")
        }

        return pbClient.markerClient.marker(QueryMarkerRequest.newBuilder().setId(denom).build()).marker.unpack(MarkerAccount::class.java)
    }
}
