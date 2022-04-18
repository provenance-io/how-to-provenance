package io.provenance.example.util

fun <T: Any> T?.elvis(default: () -> T): T = this ?: default()

fun String.isAlphaOnly(): Boolean = this.all { it.isLetter() }
