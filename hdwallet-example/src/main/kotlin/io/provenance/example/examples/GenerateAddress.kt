package io.provenance.example.examples

import io.provenance.example.util.DefaultParam
import io.provenance.example.util.InputParams
import io.provenance.example.util.InputUtil.input
import io.provenance.example.util.InputUtil.inputEnum
import io.provenance.example.util.InputUtil.inputString
import io.provenance.example.util.toDeterministicSeed
import io.provenance.example.util.toHumanReadableString
import io.provenance.hdwallet.bip39.DeterministicSeed
import io.provenance.hdwallet.bip39.MnemonicWords
import io.provenance.hdwallet.hrp.Hrp
import io.provenance.hdwallet.wallet.Wallet
import java.util.UUID

/**
 * Provenance's addresses are in bech32 format.  This example showcases the use of the Wallet class to take a seed value
 * and derive a bech32 address that might be used in a Provenance blockchain environment.
 */
object GenerateAddress : ExampleSuite {
    override fun start() {
        // Networks generally establish different prefixes for main and test networks.  For instance, provenance uses
        // tp for testnet, and pb for mainnet.
        val networkType = inputEnum(
            messagePrefix = "[Choose a network type]",
            params = InputParams(default = DefaultParam(value = NetworkType.MAINNET)),
        )
        println("Using network type [${networkType.displayName}] with hrp [${networkType.hrp}]")
        when (inputEnum<AddressGenerationType>(messagePrefix = "[Choose a generation type]")) {
            AddressGenerationType.MNEMONIC -> generateFromMnemonic(networkType)
            AddressGenerationType.SEED -> generateFromSeed(networkType)
        }
    }

    /**
     * A Wallet that allows access to keys via an HD path can easily be created using the Mnemonic words class.
     * This example showcases that.
     */
    private fun generateFromMnemonic(networkType: NetworkType) {
        val mnemonicWords = when (inputEnum<MnemonicGenerationType>("How would you like to generate a mnemonic?")) {
            MnemonicGenerationType.AUTOMATIC -> GenerateMnemonic.promptMnemonic()
            MnemonicGenerationType.MANUAL -> input(
                message = "Please enter a mnemonic (separated by spaces)",
                converter = { input -> MnemonicWords.of(input) },
            )
        }
        println("Using mnemonic [${mnemonicWords.toHumanReadableString()}] to generate address")
        // As discussed in the GenerateMnemonic comments, a mnemonic can be used alone, or with an additional passphrase
        // to further increase security in its derivation
        val passphrase = inputString(
            message = "Choose a wallet passphrase",
            params = InputParams(default = DefaultParam(value = "", description = "NONE")),
        )
        // The Wallet class allows for easy instantiation using its MnemonicWords class.  Check out GenerateMnemonic
        // to see some examples of how to easily use that class
        val wallet = Wallet.fromMnemonic(
            hrp = networkType.hrp,
            passphrase = passphrase,
            mnemonicWords = mnemonicWords,
            testnet = networkType == NetworkType.TESTNET,
        )
        // The Wallet class overrides the get operator function to allow an HDPath to be provided for account derivation.
        // The Account class contains a simple access for the Bech32 address in its value property.
        val address = wallet[networkType.hdPath].address.value
        println("Established [${networkType.displayName}] address [$address] from mnemonic using HD path [${networkType.hdPath}]")
    }

    /**
     * A wallet can also easily be generated using a ByteArray of values.  To demonstrate, this path generates a
     * random UUID and uses it as the seed for a wallet.
     */
    private fun generateFromSeed(networkType: NetworkType) {
        val seedUuid = UUID.randomUUID()
        println("Using uuid [$seedUuid] as bytes for wallet seed")
        val wallet = Wallet.fromSeed(hrp = networkType.hrp, seed = seedUuid.toDeterministicSeed())
        val address = wallet[networkType.hdPath].address.value
        println("Established [${networkType.displayName}] address [$address] from seed using HD path [${networkType.hdPath}]")
    }
}

/**
 * A shortcut to the Hrp enum's ProvenanceBlockchain values.  The Hrp enum also provides cosmos and crpyto values,
 * but those address types are not necessary for interacting with the Provenance ecoysystem.
 *
 * @param hrp Human Readable Prefix.  This value is appended to the beginning of the account's bech32 address
 * @param hdPath The hierarchical deterministic path to a Provenance address.
 */
private enum class NetworkType(val displayName: String, val hrp: String, val hdPath: String) {
    MAINNET(displayName = "mainnet", hrp = Hrp.ProvenanceBlockchain.mainnet, hdPath = "m/44'/505'/0'/0/0"),
    TESTNET(displayName = "testnet", hrp = Hrp.ProvenanceBlockchain.testnet, hdPath = "m/44'/1'/0'/0/0'"),
}

/**
 * Simple switch enum to separate each address generation pathway.
 */
private enum class AddressGenerationType {
    MNEMONIC,
    SEED,
}

/**
 * Simple switch for generating a MnemonicWords class from user input, or by random generation.
 */
private enum class MnemonicGenerationType {
    MANUAL,
    AUTOMATIC,
}
