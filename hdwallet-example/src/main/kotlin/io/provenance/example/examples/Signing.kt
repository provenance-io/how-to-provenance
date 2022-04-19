package io.provenance.example.examples

import io.provenance.example.util.DefaultParam
import io.provenance.example.util.InputParams
import io.provenance.example.util.InputUtil.inputString
import io.provenance.example.util.toDeterministicSeed
import io.provenance.hdwallet.common.hashing.sha256
import io.provenance.hdwallet.signer.BCECSigner
import io.provenance.hdwallet.wallet.Wallet
import java.util.UUID

/**
 * This is an example that is similar to the signing instructions provided in the HDWallet library's readme.
 * It generates a Wallet from a seed, and uses that seed to sign a byte array payload after hashing it as sha256.
 */
object Signing : ExampleSuite {
    private const val TESTNET_HRP: String = "tp"
    private const val TESTNET_HD_PATH: String = "m/44'/1'/0'/0/0'"

    override fun start() {
        val messageToSign = inputString("Please enter a message to sign")
        val seed = inputString(
            "Please enter a seed for your testnet wallet (any text)",
            params = InputParams(
                default = DefaultParam(value = UUID.randomUUID().toString())
            )
        )
        println("Using uuid seed [$seed] for wallet derivation")
        // Generate a Wallet from the randomly-generated UUID
        val wallet = Wallet.fromSeed(hrp = TESTNET_HRP, seed = seed.toDeterministicSeed())
        val account = wallet[TESTNET_HD_PATH]
        println("Signing payload [$messageToSign] with testnet account address [${account.address.value}]")
        // Construct a new instance of BCECSigner.  This class can be re-used, as it contains no instance variables
        val signer = BCECSigner()
        // Convert the input to a ByteArray encoded as UTF_8, and then hash it with the sha256 algorithm.
        // HDWallet provides this simple extension function for ByteArray types for ease of use
        val payload = messageToSign.toByteArray().sha256()
        // Provide the private key from the derived Wallet's account, alongside the hashed payload to get a signature
        // from the BCECSigner
        val signature = signer.sign(
            privateKey = account.keyPair.privateKey,
            payload = payload,
        )
        val signedBtcPayload = signature.encodeAsBTC().toByteArray()
        println("[SIGNED] Generated signature for payload [$messageToSign] and derived BTC bytes [${signedBtcPayload.joinToString(separator = " ") { it.toString() }}]")
        // Provide the public key from the derived Wallet's account, alongside the hash payload and the fetched
        // ECDSASignature value to ensure that the signature is valid.
        // ECDSA == Elliptic Curve Digital Signature Algorithm
        val verified = signer.verify(
            publicKey = account.keyPair.publicKey,
            data = payload,
            signature = signature,
        )
        if (verified) {
            println("[SUCCESS] Signed payload [$messageToSign]")
        } else {
            println("[ERROR] Failed to verify signature for payload [$messageToSign]")
        }
    }
}
