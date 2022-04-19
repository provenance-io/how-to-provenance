import com.google.protobuf.Any
import com.google.protobuf.ByteString
import com.google.protobuf.Message
import cosmos.base.v1beta1.CoinOuterClass
import cosmos.crypto.secp256k1.Keys
import cosmos.tx.v1beta1.ServiceOuterClass
import cosmos.tx.v1beta1.TxOuterClass
import cosmwasm.wasm.v1.Tx
import io.provenance.client.grpc.BaseReqSigner
import io.provenance.client.grpc.GasEstimationMethod
import io.provenance.client.grpc.PbClient
import io.provenance.client.grpc.Signer
import io.provenance.client.protobuf.extensions.getBaseAccount
import io.provenance.hdwallet.bip39.MnemonicWords
import io.provenance.hdwallet.common.hashing.sha256
import io.provenance.hdwallet.signer.BCECSigner
import io.provenance.hdwallet.wallet.Account
import io.provenance.hdwallet.wallet.Wallet
import io.provenance.metadata.v1.MsgWriteScopeRequest
import io.provenance.metadata.v1.Party
import io.provenance.metadata.v1.PartyType
import io.provenance.metadata.v1.ScopeRequest
import java.net.URI
import java.util.UUID

enum class NetworkType(
    /**
     * The hrp (Human Readable Prefix) of the network address
     */
    val prefix: String,
    /**
     * The HD wallet path
     */
    val path: String
) {
    TESTNET("tp", "m/44'/1'/0'/0/0"),
    TESTNET_HARDENED("tp", "m/44'/1'/0'/0/0'"),
    MAINNET("pb", "m/44'/505'/0'/0/0")
}

class WalletSigner(networkType: NetworkType, mnemonic: String, passphrase: String = "") : Signer {

    val wallet = Wallet.fromMnemonic(networkType.prefix, passphrase.toCharArray(), MnemonicWords.of(mnemonic))

    val account: Account = wallet[networkType.path]

    override fun address(): String = account.address.value

    override fun pubKey(): Keys.PubKey =
        Keys.PubKey.newBuilder().setKey(ByteString.copyFrom(account.keyPair.publicKey.compressed())).build()

    override fun sign(data: ByteArray): ByteArray = BCECSigner()
        .sign(account.keyPair.privateKey, data.sha256())
        .encodeAsBTC().toByteArray()
}

fun Message.toAny() = Any.pack(this, "")

fun main() {
    // This is just an existing scope spec in testnet that I am using for ease of use, any scope of any specification can be used
    val SCOPE_SPEC_UUID = "fefebe5a-e85d-4b75-857d-56bba1ec142d"
    val CONTRACT_ADDRESS = "tp1fft5c9nll2wkwmulmzzf90rv4ayn990j0qzp2d9cxkhu59yphrgs49secc"
    val SCOPE_UUID = UUID.randomUUID().toString()

    val client = PbClient("pio-testnet-1", URI("grpcs://grpc.test.provenance.io:443"), GasEstimationMethod.MSG_FEE_CALCULATION)

    // create a scope owned by 'seller'
    println("Please enter the seller's mnemonic:") // this is the initial owner of the scope
    val mnemonic1 = readLine()!!

    val sellerSigner = WalletSigner(NetworkType.TESTNET, mnemonic1)
    println("seller address: ${sellerSigner.address()}")

    // fetch scope to get address and base of update scope message
    val scopeResponse = ScopeCreator(client).createScope(SCOPE_UUID, SCOPE_SPEC_UUID, sellerSigner)
    val scopeAddress = scopeResponse.scope.scopeIdInfo.scopeAddr
    println("Scope address: $scopeAddress")

    // create messages in single transaction to transfer ownership to the contract and place an ask.
    // it is important that these are in the same transaction, so the scope doesn't end up owned by the contract
    // with no record for an ask and who the original owner was
    println("Transferring scope ownership to contract and placing ask for hash")
    val ASK_UUID = UUID.randomUUID()
    println("ask uuid: $ASK_UUID")
    val transferScopeMsg = scopeResponse.getChangeOwnerMessage(CONTRACT_ADDRESS) // helper message to take existing scope and generate set ownership message
    val createAskMsg = Tx.MsgExecuteContract.newBuilder()
        .setContract(CONTRACT_ADDRESS)
        .setMsg(ByteString.copyFromUtf8("""{"create_ask": {"id": "$ASK_UUID", "quote": [{"denom": "nhash", "amount": "1"}], "scope_address": "$scopeAddress"}}"""))
        .setSender(sellerSigner.address())
        .build().toAny()

    val sellerAccount = client.authClient.getBaseAccount(sellerSigner.address())
    // this shows how to send multiple messages in one transaction. If one message fails, they all fail and no action is taken (though some gas fees may still be incurred)
    client.estimateAndBroadcastTx(TxOuterClass.TxBody.newBuilder().addAllMessages(listOf(transferScopeMsg, createAskMsg)).build(), listOf(
        BaseReqSigner(sellerSigner, 0, sellerAccount)
    ), ServiceOuterClass.BroadcastMode.BROADCAST_MODE_BLOCK).also {
        if (it.txResponse.code != 0) {
            throw Exception("Error transferring/creating ask for scope, code: ${it.txResponse.code}, message: ${it.txResponse.rawLog}")
        }

        println("Scope ask created successfully: ${it.txResponse.txhash}")
    }

    // place a bid for the scope as the 'buyer' with the same funds as the ask requested
    println("Please enter the bidder's mnemonic:")
    val mnemonic2 = readLine()!!

    val buyerSigner = WalletSigner(NetworkType.TESTNET, mnemonic2)
    val buyerAccount = client.authClient.getBaseAccount(buyerSigner.address())
    var buyerOffset = 0
    println("buyer address: ${buyerSigner.address()}")

    println("Creating bid for scope")
    val BID_UUID = UUID.randomUUID()
    println("bid uuid: $BID_UUID")
    val createBidMsg = Tx.MsgExecuteContract.newBuilder()
        .setContract(CONTRACT_ADDRESS)
        // note the base is the same scope address as was created above
        .setMsg(ByteString.copyFromUtf8("""{"create_bid": {"id": "$BID_UUID", "base": {"scope": {"scope_address": "$scopeAddress"}}}}"""))
        .addFunds(CoinOuterClass.Coin.newBuilder().setAmount("1").setDenom("nhash"))
        .setSender(buyerSigner.address())
        .build().toAny()

    client.estimateAndBroadcastTx(TxOuterClass.TxBody.newBuilder().addMessages(createBidMsg).build(), listOf(
        BaseReqSigner(buyerSigner, buyerOffset, buyerAccount)
    ), ServiceOuterClass.BroadcastMode.BROADCAST_MODE_BLOCK).also {
        if (it.txResponse.code != 0) {
            throw Exception("Error creating bid for scope, code: ${it.txResponse.code}, message: ${it.txResponse.rawLog}")
        }

        println("Scope bid created successfully: ${it.txResponse.txhash}")
        buyerOffset++
    }

    // execute the match between the supplied bid/ask as the contract admin. Note that only the admin has the privileges
    // to execute the match, though all they are doing is supplying two ids and the validation of the match
    // will still happen within the contract
    println("Please enter the contract admin mnemonic:")
    val adminMnemonic = readLine()!!

    val adminSigner = WalletSigner(NetworkType.TESTNET, adminMnemonic)
    val adminAccount = client.authClient.getBaseAccount(adminSigner.address())
    var adminOffset = 0

    val executeMatchMsg = Tx.MsgExecuteContract.newBuilder()
        .setContract(CONTRACT_ADDRESS)
        .setMsg(ByteString.copyFromUtf8("""{"execute_match": {"ask_id": "$ASK_UUID", "bid_id": "$BID_UUID"}}"""))
        .setSender(adminSigner.address())
        .build().toAny()

    client.estimateAndBroadcastTx(TxOuterClass.TxBody.newBuilder().addMessages(executeMatchMsg).build(), listOf(
        BaseReqSigner(adminSigner, adminOffset, adminAccount)
    ), ServiceOuterClass.BroadcastMode.BROADCAST_MODE_BLOCK).also {
        if (it.txResponse.code != 0) {
            throw Exception("Error matching scope ask/bid, code: ${it.txResponse.code}, message: ${it.txResponse.rawLog}")
        }

        println("Scope matched successfully, transfer complete: ${it.txResponse.txhash}")
        adminOffset++
    }
}
