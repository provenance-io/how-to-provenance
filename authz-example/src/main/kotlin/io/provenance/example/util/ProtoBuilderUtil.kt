package io.provenance.example.util

import cosmos.base.v1beta1.CoinOuterClass.Coin

object ProtoBuilderUtil {
    fun coin(amount: Long, denom: String): Coin = coin(amount.toString(), denom)

    fun coin(amount: String, denom: String): Coin = Coin.newBuilder().setAmount(amount).setDenom(denom).build()
}
