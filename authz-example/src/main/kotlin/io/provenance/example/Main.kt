package io.provenance.example

import io.provenance.example.util.PbClientUtil
import io.provenance.name.v1.QueryResolveRequest

fun main() {
    val pbClient = PbClientUtil.newClient()
    println(pbClient.nameClient.resolve(QueryResolveRequest.newBuilder().setName("testassets.pb").build()).address)
}
