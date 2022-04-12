package io.provenance.example

import io.provenance.eventstream.decoder.moshiDecoderAdapter
import io.provenance.eventstream.extensions.decodeBase64
import io.provenance.eventstream.net.okHttpNetAdapter
import io.provenance.eventstream.stream.flows.blockFlow
import io.provenance.eventstream.stream.models.extensions.blockEvents
import io.provenance.eventstream.stream.models.extensions.dateTime
import io.provenance.eventstream.stream.models.extensions.txData
import io.provenance.eventstream.stream.models.extensions.txEvents
import kotlinx.coroutines.flow.collect
import kotlinx.coroutines.flow.filter
import kotlinx.coroutines.runBlocking

val EVENTS = listOf("wasm")

/**
 * Listen to the Provenance event stream for transaction events of interest
 *
 * Provide the following environment variables to supply connection information
 * NODE_URI: the uri of a Provenance node (i.e. http://localhost:26657 if running locally)
 * START_HEIGHT: a specific block height to start listening for events at (otherwise, the latest height will be used)
 */
suspend fun main() {
    val netAdapter = okHttpNetAdapter(System.getenv("NODE_URI"))
    val decoderAdapter = moshiDecoderAdapter()

    val startingBlockHeight = System.getenv("START_HEIGHT")?.toLong() ?: netAdapter.rpcAdapter.getCurrentHeight()
    println("Listening for events $EVENTS from height $startingBlockHeight")

    // initialize the event stream flow
    blockFlow(netAdapter, decoderAdapter, from = startingBlockHeight)
        .collect { blockData ->
            blockData.blockResult.txEvents(blockData.block.dateTime()) { index -> blockData.block.txData(index) }
                .filter { txEvent -> txEvent.eventType in EVENTS } // filter out events you are not looking for
                .takeIf { it.isNotEmpty() }
                ?.also {
                    println("Received block with desired events at height ${blockData.height}")
                }
                ?.joinToString("\n") { blockEvent ->
                    // event attributes are key/value pairs nested under the event that are base64-encoded
                    "${blockEvent.eventType}: " +  blockEvent.attributes.joinToString("\n\t", prefix = "[\n\t", postfix = "\n]") { attribute -> "${attribute.key?.decodeBase64()}: ${attribute.value?.decodeBase64()}" }
                }?.let(::println)
        }
}
