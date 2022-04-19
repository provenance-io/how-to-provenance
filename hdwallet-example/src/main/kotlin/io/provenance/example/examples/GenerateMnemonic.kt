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
 * derive its accounts from a generated mnemonic.  As such,
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
                validate = { value -> value % 32 == 0 },
                validationRuleText = listOf("Input must be a multiple of 32"),
            )
        ),
        converter = { value -> value.toInt() },
    )
}
