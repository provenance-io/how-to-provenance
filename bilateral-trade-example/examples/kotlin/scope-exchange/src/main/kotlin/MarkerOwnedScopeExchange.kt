import com.google.protobuf.ByteString
import cosmos.base.v1beta1.CoinOuterClass
import cosmos.tx.v1beta1.ServiceOuterClass
import cosmos.tx.v1beta1.TxOuterClass
import cosmwasm.wasm.v1.Tx
import io.provenance.client.grpc.BaseReqSigner
import io.provenance.client.grpc.GasEstimationMethod
import io.provenance.client.grpc.PbClient
import io.provenance.client.protobuf.extensions.getBaseAccount
import java.net.URI
import java.util.UUID

fun main() {
    val SCOPE_SPEC_UUID = "fefebe5a-e85d-4b75-857d-56bba1ec142d"
    val CONTRACT_ADDRESS = "tp1fft5c9nll2wkwmulmzzf90rv4ayn990j0qzp2d9cxkhu59yphrgs49secc"
    val SCOPE_UUID = UUID.randomUUID().toString()

    val client = PbClient("pio-testnet-1", URI("grpcs://grpc.test.provenance.io:443"), GasEstimationMethod.MSG_FEE_CALCULATION)

    // create a scope owned by 'seller'
    println("Please enter the seller's mnemonic:") // this is the initial owner of the scope
    val mnemonic1 = readLine()!!

    val initialOwnerSigner = WalletSigner(NetworkType.TESTNET_HARDENED, mnemonic1)
    println("initial owner address: ${initialOwnerSigner.address()}")

    // fetch scope to get address and base of update scope message
    val scopeResponse = ScopeCreator(client).createScope(SCOPE_UUID, SCOPE_SPEC_UUID, initialOwnerSigner)
    val scopeAddress = scopeResponse.scope.scopeIdInfo.scopeAddr
    println("Scope address: $scopeAddress")

    // at this point, a regular account owns the scope. We are going to go ahead and create a marker, transfer ownership to that marker
    // and then demonstrate that we can exchange shares in that marker as a proxy for exchanging control of the scope

    println("Enter the marker's denom")
    val markerDenom = readLine()!!
    val markerShares = 100

    val marker = MarkerCreator(client).createMarker(markerShares, markerDenom, initialOwnerSigner)

    val setScopeOwnerMsg = scopeResponse.getChangeOwnerMessage(marker.baseAccount.address)

    var ownerAccount = client.authClient.getBaseAccount(initialOwnerSigner.address())

    client.estimateAndBroadcastTx(
        TxOuterClass.TxBody.newBuilder().addMessages(setScopeOwnerMsg).build(), listOf(
            BaseReqSigner(initialOwnerSigner, 0, ownerAccount)
        ), ServiceOuterClass.BroadcastMode.BROADCAST_MODE_BLOCK, gasAdjustment = 1.5).also {
        if (it.txResponse.code != 0) {
            throw Exception("Error transferring ownership of scope to marker, code: ${it.txResponse.code}, message: ${it.txResponse.rawLog}")
        }

        println("Marker successfully set as scope owner: ${it.txResponse.txhash}")
    }

    println("Transferring scope ownership to contract and placing ask for hash")
    val ASK_UUID = UUID.randomUUID()
    println("ask uuid: $ASK_UUID")
    val createAskMsg = Tx.MsgExecuteContract.newBuilder()
        .setContract(CONTRACT_ADDRESS)
        .setMsg(ByteString.copyFromUtf8("""{"create_ask": {"id": "$ASK_UUID", "quote": [{"denom": "nhash", "amount": "1"}]}}""")) // note no scope address set in message
        .addFunds(CoinOuterClass.Coin.newBuilder().setAmount(markerShares.toString()).setDenom(markerDenom)) // sending marker shares in as proxy for scope ownership
        .setSender(initialOwnerSigner.address())
        .build().toAny()

    ownerAccount = client.authClient.getBaseAccount(initialOwnerSigner.address())
    // this shows how to send multiple messages in one transaction. If one message fails, they all fail and no action is taken (though some gas fees may still be incurred)
    client.estimateAndBroadcastTx(TxOuterClass.TxBody.newBuilder().addAllMessages(listOf(createAskMsg)).build(), listOf(
        BaseReqSigner(initialOwnerSigner, 0, ownerAccount)
    ), ServiceOuterClass.BroadcastMode.BROADCAST_MODE_BLOCK).also {
        if (it.txResponse.code != 0) {
            throw Exception("Error transferring/creating ask for marker denom, code: ${it.txResponse.code}, message: ${it.txResponse.rawLog}")
        }

        println("Marker denom ask created successfully: ${it.txResponse.txhash}")
    }

    // place a bid for the scope as the 'buyer' with the same funds as the ask requested
    println("Please enter the bidder's mnemonic:")
    val mnemonic2 = readLine()!!

    val buyerSigner = WalletSigner(NetworkType.TESTNET_HARDENED, mnemonic2)
    val buyerAccount = client.authClient.getBaseAccount(buyerSigner.address())
    println("buyer address: ${buyerSigner.address()}")

    println("Creating bid for marker denom")
    val BID_UUID = UUID.randomUUID()
    println("bid uuid: $BID_UUID")
    val createBidMsg = Tx.MsgExecuteContract.newBuilder()
        .setContract(CONTRACT_ADDRESS)
        // note the base is the same scope address as was created above
        .setMsg(ByteString.copyFromUtf8("""{"create_bid": {"id": "$BID_UUID", "base": {"coin": {"coins": [{"amount": "$markerShares", "denom": "$markerDenom"}]}}}}"""))
        .addFunds(CoinOuterClass.Coin.newBuilder().setAmount("1").setDenom("nhash"))
        .setSender(buyerSigner.address())
        .build().toAny()

    client.estimateAndBroadcastTx(TxOuterClass.TxBody.newBuilder().addMessages(createBidMsg).build(), listOf(
        BaseReqSigner(buyerSigner, 0, buyerAccount)
    ), ServiceOuterClass.BroadcastMode.BROADCAST_MODE_BLOCK).also {
        if (it.txResponse.code != 0) {
            throw Exception("Error creating bid for scope, code: ${it.txResponse.code}, message: ${it.txResponse.rawLog}")
        }

        println("Scope bid created successfully: ${it.txResponse.txhash}")
    }

    // execute the match between the supplied bid/ask as the contract admin. Note that only the admin has the privileges
    // to execute the match, though all they are doing is supplying two ids and the validation of the match
    // will still happen within the contract
    println("Please enter the contract admin mnemonic:")
    val adminMnemonic = readLine()!!

    val adminSigner = WalletSigner(NetworkType.TESTNET, adminMnemonic)
    val adminAccount = client.authClient.getBaseAccount(adminSigner.address())

    val executeMatchMsg = Tx.MsgExecuteContract.newBuilder()
        .setContract(CONTRACT_ADDRESS)
        .setMsg(ByteString.copyFromUtf8("""{"execute_match": {"ask_id": "$ASK_UUID", "bid_id": "$BID_UUID"}}"""))
        .setSender(adminSigner.address())
        .build().toAny()

    client.estimateAndBroadcastTx(TxOuterClass.TxBody.newBuilder().addMessages(executeMatchMsg).build(), listOf(
        BaseReqSigner(adminSigner, 0, adminAccount)
    ), ServiceOuterClass.BroadcastMode.BROADCAST_MODE_BLOCK).also {
        if (it.txResponse.code != 0) {
            throw Exception("Error matching marker denom ask/bid, code: ${it.txResponse.code}, message: ${it.txResponse.rawLog}")
        }

        println("marker denom matched successfully, transfer complete: ${it.txResponse.txhash}")
    }

    // at the end, the 'buyer' holds all of the marker denom, thereby holding the value of the scope
    // and the 'seller' received the quote value in exchange (just 1 hash in this case, could be any combination of coins
    // the scope itself is still owned by the marker
}
