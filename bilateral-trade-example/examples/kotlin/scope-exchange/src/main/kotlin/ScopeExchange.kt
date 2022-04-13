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
    println("Please enter the seller's mnemonic:")
    val mnemonic1 = readLine()!!

    val sellerSigner = WalletSigner(NetworkType.TESTNET_HARDENED, mnemonic1)
    val sellerAccount = client.authClient.getBaseAccount(sellerSigner.address())
    var sellerOffset = 0
    println("seller address: ${sellerSigner.address()}")

    println("Creating scope")
    val createScopeMsg = MsgWriteScopeRequest.newBuilder()
        .setScopeUuid(SCOPE_UUID)
        .setSpecUuid(SCOPE_SPEC_UUID)
        .addSigners(sellerSigner.address())
        .apply {
            scopeBuilder.setValueOwnerAddress(sellerSigner.address())
                .addOwners(Party.newBuilder()
                    .setRole(PartyType.PARTY_TYPE_OWNER)
                    .setAddress(sellerSigner.address())
                )
        }
        .build().toAny()

    client.estimateAndBroadcastTx(TxOuterClass.TxBody.newBuilder().addMessages(createScopeMsg).build(), listOf(
        BaseReqSigner(sellerSigner, sellerOffset, sellerAccount)
    ), ServiceOuterClass.BroadcastMode.BROADCAST_MODE_BLOCK, gasAdjustment = 1.5).also {
        if (it.txResponse.code != 0) {
            throw Exception("Error creating scope, code: ${it.txResponse.code}, message: ${it.txResponse.rawLog}")
        }

        println("Scope created successfully: ${it.txResponse.txhash}")
        sellerOffset++
    }

    // fetch scope to get address and base of update scope message
    val scopeResponse = client.metadataClient.scope(ScopeRequest.newBuilder().setScopeId(SCOPE_UUID).build())
    val scope = scopeResponse.scope.scope
    val scopeAddress = scopeResponse.scope.scopeIdInfo.scopeAddr
    println("Scope address: $scopeAddress")

    // create messages in single transaction to transfer ownership to the contract and place an ask
    println("Transferring scope ownership to contract and placing ask for hash")
    val ASK_UUID = UUID.randomUUID()
    println("ask uuid: $ASK_UUID")
    val transferScopeMsg = MsgWriteScopeRequest.newBuilder()
        .setScopeUuid(SCOPE_UUID)
        .setSpecUuid(SCOPE_SPEC_UUID)
        .addSigners(sellerSigner.address())
        .setScope(scope.toBuilder()
            .setValueOwnerAddress(CONTRACT_ADDRESS)
            .clearOwners()
            .addAllOwners(scope.ownersList.filter { it.role != PartyType.PARTY_TYPE_OWNER }.plus(Party.newBuilder()
                .setAddress(CONTRACT_ADDRESS)
                .setRole(PartyType.PARTY_TYPE_OWNER)
                .build())
            )
        )
        .build().toAny()
    val createAskMsg = Tx.MsgExecuteContract.newBuilder()
        .setContract(CONTRACT_ADDRESS)
        .setMsg(ByteString.copyFromUtf8("""{"create_ask": {"id": "$ASK_UUID", "quote": [{"denom": "nhash", "amount": "1"}], "scope_address": "$scopeAddress"}}"""))
        .setSender(sellerSigner.address())
        .build().toAny()

    client.estimateAndBroadcastTx(TxOuterClass.TxBody.newBuilder().addAllMessages(listOf(transferScopeMsg, createAskMsg)).build(), listOf(
        BaseReqSigner(sellerSigner, sellerOffset, sellerAccount)
    ), ServiceOuterClass.BroadcastMode.BROADCAST_MODE_BLOCK).also {
        if (it.txResponse.code != 0) {
            throw Exception("Error transferring/creating ask for scope, code: ${it.txResponse.code}, message: ${it.txResponse.rawLog}")
        }

        println("Scope ask created successfully: ${it.txResponse.txhash}")
        sellerOffset++
    }

    // place a bid for the scope as the 'buyer' with the same funds as the ask requested
    println("Please enter the bidder's mnemonic:")
    val mnemonic2 = readLine()!!

    val buyerSigner = WalletSigner(NetworkType.TESTNET_HARDENED, mnemonic2)
    val buyerAccount = client.authClient.getBaseAccount(buyerSigner.address())
    var buyerOffset = 0
    println("buyer address: ${buyerSigner.address()}")

    println("Creating bid for scope")
    val BID_UUID = UUID.randomUUID()
    println("bid uuid: $BID_UUID")
    val createBidMsg = Tx.MsgExecuteContract.newBuilder()
        .setContract(CONTRACT_ADDRESS)
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

    // execute the match between the supplied bid/ask as the contract admin
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
