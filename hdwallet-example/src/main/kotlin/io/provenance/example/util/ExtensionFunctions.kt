package io.provenance.example.util

import io.provenance.hdwallet.bip39.MnemonicWords

/**
 * Each "word" in the MnemonicWords class is stored as a CharArray.  This function simply joins them using Kotlin's
 * CharArray.concatToString() helper function, which is the same as instantiating a String using the CharArray as
 * a constructor parameter.
 */
fun MnemonicWords.toHumanReadableString(): String = this
    .words
    .joinToString(separator = " ") { it.concatToString() }
