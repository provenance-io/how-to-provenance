package io.provenance.example.util

fun <T: Any> T?.elvis(default: () -> T): T = this ?: default()
