package io.provenance.example.examples

import io.provenance.example.util.InputParams
import io.provenance.example.util.InputUtil.inputEnum
import io.provenance.example.util.InputUtil.inputString
import io.provenance.example.util.InputValidation
import io.provenance.hdwallet.bip39.MnemonicWords
import io.provenance.hdwallet.wallet.Wallet
import kotlin.math.pow

/**
 * The Provenance Blockchain uses the concept hierarchical deterministic wallets and pathing as defined in the BIP32
 * proposal.  This example showcases how many different values can be used with an HD-path to build different addresses
 * from a singular mnemonic.
 */
object ExploreHDPath : ExampleSuite {
    /**
     * This example always uses the same mnemonic to illustrate how HD pathing can result in many different addresses
     */
    private const val MNEMONIC: String = "congress buddy level powder joke corn area price tell stem virtual above before camera cruise produce arena clarify sentence shield target brown gloom drift"

    /**
     * A number in an HD path can be at most (2^31) - 1
     */
    private val MAX_PATH_VALUE: Long = (2.0.pow(31) - 1).toLong()

    /**
     * Ensures that the given value for the HD path is valid against the following terms:
     * - Must contain at MOST one apostrophe.
     * - If an apostrophe exists in the string, it must be the final character in the string.
     * - The string must contain a non-negative integer.
     * - Non-negative integer must be less than the maximum HD path value of (2^31) - 1
     */
    private val hdPathValidation: InputValidation<String> = InputValidation(
        validate = { input ->
            input.trim().let { trimmed ->
                trimmed.count { it == '\'' }.let { apostropheCount ->
                    // The value must only contain, at most, a single apostrophe
                    apostropheCount <= 1 &&
                            // The input must either have no apostrophes, or must end with one
                            (apostropheCount == 0 || trimmed.endsWith('\'')) &&
                            // The input before the apostrophe must be zero, or within the upper bound for HD paths
                            trimmed.split("'")
                                .firstOrNull()
                                ?.toLongOrNull()
                                ?.let { numericPart -> numericPart in 0..MAX_PATH_VALUE } == true
                }
            }
        },
        validationRuleText = listOf(
            "The value must only contain at most a single apostrophe",
            "The input must either have no apostrophes or must end with one",
            "The input before the apostrophe must be zero or within the upper bound for HD paths",
        )
    )

    override fun start() {
        // This example text was paraphrased from: https://docs.provenance.io/blockchain/basics/accounts
        println("""
            
            -------------------------------------------------------------------------------------------------------------------------------
            This example illustrates how a single mnemonic can become many addresses by modifying the HD (hierarchical deterministic) path.

            Path structure (this path is for creating a Provenance Blockchain mainnet account):
            m / 44' / 505' /  0' /  0  /  0
            -------------------------------
            1   2     3       4     5     6
            
            1. Root:      The extended key pair seed value that is restored from a mnemonic phrase.
            2. Purpose:   Set to 44' (hardened) to follow the BIP43 recommendation. Indicates that the sub-tree follows the 
                          BIP43 specification.
            3. Coin Type: Specifies the type of coin native to the blockchain.  Provenance Blockchain uses 505, as it is 
                          registered: https://github.com/satoshilabs/slips/blob/master/slip-0044.md
            4. Account:   A numbered index beginning at 0. This number can be incremented to move to a different account.
            5. Change:    0 for internal and 1 for external. Internal is used for transactions within a wallet, whereas 
                          external are intended to be outwardly extended, ex: receiving payments.
            6. Index:     This number is an increasing index for use in BIP32 derivation. This is commonly-hardened.
            
            *Hardened key: A hardened key is a value in the HD path that ends with an apostrophe.  This ensures that a 
                           child public key cannot be logically proven to be linked to a parent private key.  There's no
                           way to go "back up" the path.
            
            *Provenance Testnet Path: m/44'/1'/0'/0/0'
                                      Notice that the index is hardened?  Provenance Blockchain uses a hardened index in
                                      its testnet addresses as a standard.  Above, you can see the mainnet address path
                                      does not have a hardened index.
                                      
            Well, what are we waiting for?  Let's make a Provenance Blockchain path!
            -------------------------------------------------------------------------------------------------------------------------------
            
            Using mnemonic: [$MNEMONIC]
              
        """.trimIndent())

        // This is a Provenance Blockchain address. Always use "m" for the root
        val root = "m"
        println("[${getPath(root)}]. Used root [$root]")
        // This is a Provenance Blockchain address. Always use "44'" for the purpose to conform to BIP43.
        val purpose = "44'"
        println("[${getPath(root, purpose)}]. Used purpose [$purpose]")
        val coinType = inputEnum<CoinType>("[Enter a coin type]")
        println("[${getPath(root, purpose, coinType.value)}]. Used coin type [${coinType.value}] (${coinType.displayName()})")
        val account = inputString(
            message = "Enter an account value (0 - $MAX_PATH_VALUE)",
            params = InputParams(validation = hdPathValidation),
        )
        println("[${getPath(root, purpose, coinType.value, account)}]. Used account [$account]")
        val changeType = inputEnum<ChangeType>("[Enter a change type]")
        println("[${getPath(root, purpose, coinType.value, account, changeType.value)}]. Used change type [${changeType.value}] (${changeType.name.lowercase()})")
        val index = inputString(
            message = "Enter an index value (0 - $MAX_PATH_VALUE)",
            params = InputParams(validation = hdPathValidation),
        )
        val hdPath = getPath(root, purpose, coinType.value, account, changeType.value, index)
        println("[$hdPath]. Used index [$index]")
        println("Used prefix [${coinType.hrp}] to derive [${coinType.displayName()}] address with HD path [$hdPath]: ${generateAddress(hdPath, coinType)}")
    }

    /**
     * A builder for an HD path with the option to omit various inputs, ensuring that partially-complete values can be
     * omitted for logging.
     */
    private fun getPath(
        root: String? = null,
        purpose: String? = null,
        coinType: String? = null,
        account: String? = null,
        change: String? = null,
        index: String? = null,
    ): String = listOfNotNull(root, purpose, coinType, account, change, index).joinToString("/")

    /**
     * Takes the generated HD path from the example, alongside the chosen CoinType enum to build a wallet with the
     * default mnemonic.  The wallet is then used to derive an account's address with the hd path.
     */
    private fun generateAddress(hdPath: String, coinType: CoinType): String = Wallet.fromMnemonic(
        hrp = coinType.hrp,
        // Keep things simple across re-runs by avoiding passphrase input.  This is yet another way to create different
        // account addresses using the same mnemonic and hd path.
        passphrase = "",
        mnemonicWords = MnemonicWords.of(MNEMONIC),
        testnet = coinType == CoinType.TESTNET,
    )[hdPath].address.value

}

/**
 * A simple switch between Provenance Blockchain HD path coin types.
 */
private enum class CoinType(val value: String, val hrp: String) {
    /**
     * Provenance uses 505' as its coin type for mainnet.
     */
    MAINNET(value = "505'", hrp = "pb"),

    /**
     * Provenance uses 1' as a testnet coin type.  It's the standard for all testnets: https://github.com/satoshilabs/slips/blob/master/slip-0044.md
     */
    TESTNET(value = "1'", hrp = "tp");

    fun displayName(): String = this.name.lowercase()
}

/**
 * A simple switch for change types that allows users to visibly experience the difference between internal and external.
 */
private enum class ChangeType(val value: String) {
    INTERNAL(value = "0"),
    EXTERNAL(value = "1"),
}
