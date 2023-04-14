package io.provenance.example

import kotlinx.coroutines.flow.collect
import tech.figure.eventstream.decodeBase64
import tech.figure.eventstream.decoder.moshiDecoderAdapter
import tech.figure.eventstream.net.okHttpNetAdapter
import tech.figure.eventstream.stream.flows.blockDataFlow
import tech.figure.eventstream.stream.flows.pollingBlockDataFlow
import tech.figure.eventstream.stream.models.dateTime
import tech.figure.eventstream.stream.models.txData
import tech.figure.eventstream.stream.models.txEvents

val EVENTS = listOf("wasm")

/**
 * Listen to the Provenance Blockchain event stream for transaction events of interest
 *
 * Provide the following environment variables to supply connection information
 * NODE_URI: the uri of a Provenance node (i.e. http://localhost:26657 if running locally)
 * START_HEIGHT: a specific block height to start listening for events at (otherwise, the latest height will be used)
 */
suspend fun main() {
    val nodeUri = System.getenv("NODE_URI") ?: Defaults.NODE_URI
    val netAdapter = okHttpNetAdapter(nodeUri)
    val decoderAdapter = moshiDecoderAdapter()

    val startingBlockHeight = System.getenv("START_HEIGHT")?.toLong()
    println("Listening for events $EVENTS from height $startingBlockHeight")

    // initialize the event stream flow
    when (startingBlockHeight) {
        null -> pollingBlockDataFlow(netAdapter) // no starting height, listen for live blocks via polling strategy
        else -> blockDataFlow(netAdapter, decoderAdapter, from = startingBlockHeight) // starting height provided, use combined historical/live flow
    }.collect { blockData ->
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
