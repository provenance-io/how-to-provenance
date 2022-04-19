package io.provenance.example.util

import io.provenance.hdwallet.bip39.DeterministicSeed
import io.provenance.hdwallet.bip39.MnemonicWords
import java.util.UUID

/**
 * Each "word" in the MnemonicWords class is stored as a CharArray.  This function simply joins them using Kotlin's
 * CharArray.concatToString() helper function, which is the same as instantiating a String using the CharArray as
 * a constructor parameter.
 */
fun MnemonicWords.toHumanReadableString(): String = this
    .words
    .joinToString(separator = " ") { it.concatToString() }

/**
 * Helper function to convert a UUID to a DeterministicSeed by convert it to a UTF-8 encoded ByteArray.
 * A DeterministicSeed can also be created using a java SecretKey, but this is much simpler to
 * demonstrate.
 */
fun UUID.toDeterministicSeed(): DeterministicSeed = this.toString().toDeterministicSeed()

/**
 * Helper function to convert any string to a deterministic seed by using fromBytes and passing in a UTF_8 encoded
 * ByteArray.
 */
fun String.toDeterministicSeed(): DeterministicSeed = DeterministicSeed.fromBytes(
    // Kotlin's String.toByteArray extension function automatically defaults to UTF_8 encoding
    this.toByteArray()
)
