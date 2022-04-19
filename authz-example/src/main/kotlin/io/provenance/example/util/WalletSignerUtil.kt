package io.provenance.example.util

import io.provenance.client.wallet.NetworkType
import io.provenance.client.wallet.WalletSigner
import io.provenance.client.wallet.fromMnemonic
import io.provenance.example.util.InputUtil.inputString

/**
 * Uses Provenance's fromMnemonic function to acquire a WalletSigner for use in PbClient transactions.  For the purposes
 * of these examples, only testnet addresses are accepted.
 */
object WalletSignerUtil {
    fun newSigner(messagePrefix: String? = null, mnemonic: String? = null): WalletSigner = fromMnemonic(
        networkType = NetworkType.TESTNET,
        mnemonic = mnemonic ?: inputString("$messagePrefix Please enter a valid mnemonic (testnet address)"),
    )
}
