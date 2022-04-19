package io.provenance.example.examples

import io.provenance.example.util.DefaultParam
import io.provenance.example.util.InputParams
import io.provenance.example.util.InputUtil.input
import io.provenance.example.util.InputValidation
import io.provenance.example.util.toHumanReadableString
import io.provenance.hdwallet.bip39.MnemonicWords

/**
 * A mnemonic phrase is a secret seed that can be used to derive an infinite number of accounts on a blockchain.
 * These phrases are the building blocks of a Provenance account's address.  Provenance uses a standard pathing to
 * derive its accounts from a generated mnemonic.
 * A good readme on how the mnemonic words list is intended to be used and its purpose: https://github.com/bitcoin/bips/blob/master/bip-0039.mediawiki
 */
object GenerateMnemonic : ExampleSuite {
    override fun start() {
        println(this.promptMnemonic().toHumanReadableString())
    }

    // Use Provenance's MnemonicWords helper class to generate a new mnemonic using the user's strength input
    fun promptMnemonic(): MnemonicWords = MnemonicWords.generate(strength = promptMnemonicStrength())

    private fun promptMnemonicStrength(): Int = input(
        message = "Enter a mnemonic strength (multiple of 32)",
        params = InputParams(
            // The provenanced tool generates mnemonics using a strength of 256, so using this default will
            // emulate that output
            default = DefaultParam(value = 256),
            validation = InputValidation(
                validate = { value -> value % 32 == 0 && value >= 32 && value <= 256 },
                validationRuleText = listOf(
                    // The strength value is used to create a byte array, which mandates a compatible size
                    // The underlying MnemonicWords.generate() function does a check to ensure the input is divisible by
                    // 32 when it runs
                    "Input must be a multiple of 32",
                    // The MnemonicWords.generate() function will accept a value of 0, but it will generate no words
                    "Input must be greater than or equal to 32",
                    // The MnemonicWords.generate() function throws an exception for values greater than 256
                    "Input must be less than or equal to 256"
                ),
            )
        ),
        converter = { value -> value.toInt() },
    )
}
