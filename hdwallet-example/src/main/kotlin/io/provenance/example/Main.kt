package io.provenance.example

import io.provenance.example.examples.ExampleSuite
import io.provenance.example.examples.GenerateMnemonic
import io.provenance.example.examples.GenerateAddress
import io.provenance.example.examples.Signing
import io.provenance.example.util.InputUtil.inputEnum

/**
 * Main entrypoint for the application.  Executes one ExampleSuite's start() function and then exits.
 * To start this code, simply run `./gradlew run` from the root directory.
 *
 * For the best possible user experience, avoiding unnecessary gradle messages, add the `-q` flag:
 * `./gradlew run -q`
 */
fun main() {
    println("""
        Provenance HDWallet Examples
        --------------------------------------------------------------------------------------------------------------
        GENERATE_ADDRESS:  Example of using the Wallet class to derive a bech32 address.
        GENERATE_MNEMONIC: Example of using the MnemonicWords class to generate a mnemonic.
        SIGNING:           Example of using the BCECSigner class to sign a message payload.
        
        Type "quit" at any time into any prompt to exit the program early.
        --------------------------------------------------------------------------------------------------------------
        
    """.trimIndent())
    // Use the InputUtil's inputEnum helper function to get one of the Examples enum values and execute the suite it
    // contains.  inputEnum will continue to prompt for input until "quit" is typed, or one of the enum values declared
    // in the Examples enum class is specified
    inputEnum<Examples>("Which example would you like to run?").suite.start()
}

/**
 * Each example is established as an object under io.provenance.example.examples, and each enum is assigned to one
 * of the files.  Each suite inherits from the ExampleSuite interface, which provides a start function.
 */

private enum class Examples(val suite: ExampleSuite) {
    GENERATE_ADDRESS(suite = GenerateAddress),
    GENERATE_MNEMONIC(suite = GenerateMnemonic),
    SIGNING(suite = Signing),
}
