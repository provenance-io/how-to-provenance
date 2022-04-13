package io.provenance.example

import io.provenance.eventstream.decoder.moshiDecoderAdapter
import io.provenance.eventstream.extensions.decodeBase64
import io.provenance.eventstream.net.okHttpNetAdapter
import io.provenance.eventstream.stream.flows.blockFlow
import io.provenance.eventstream.stream.flows.liveBlockFlow
import io.provenance.eventstream.stream.kafkaBlockSink
import io.provenance.eventstream.stream.kafkaBlockSource
import io.provenance.eventstream.stream.models.StreamBlockImpl
import io.provenance.eventstream.stream.models.extensions.blockEvents
import io.provenance.eventstream.stream.models.extensions.dateTime
import io.provenance.eventstream.stream.models.extensions.txData
import io.provenance.eventstream.stream.models.extensions.txErroredEvents
import io.provenance.eventstream.stream.models.extensions.txEvents
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.catch
import kotlinx.coroutines.flow.collect
import kotlinx.coroutines.flow.onEach
import kotlinx.coroutines.joinAll
import kotlinx.coroutines.launch
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.withContext
import org.apache.kafka.clients.CommonClientConfigs
import org.apache.kafka.clients.consumer.ConsumerConfig
import org.apache.kafka.clients.producer.KafkaProducer
import org.apache.kafka.clients.producer.ProducerConfig

suspend fun main() {
    // the topic to publish to/read from, can override if needed. The default will work with the docker-compose setup
    val topicName = System.getenv("KAFKA_TOPIC") ?: Defaults.KAFKA_TOPIC
    // the comma-separated kafka servers list
    val kafkaServers = System.getenv("KAFKA_BOOTSTRAP_SERVERS_CONFIG") ?: Defaults.KAFKA_SERVERS

    // for the sake of example, we are going to launch a producer/consumer in the same process
    // in reality, these would be entirely different services
    withContext(Dispatchers.IO) {
        try {
            listOf(
                launch { startProducer(kafkaServers, topicName) },
                launch { startConsumer(kafkaServers, topicName) }
            ).joinAll()
        } catch (e: Exception) {
            println("Error running producer/consumer setup, ${e.message}")
        }
    }
}

/**
 * A Kafka producer process that reads from the event stream and then publishes to the topic via the KafkaBlockSink
 */
suspend fun startProducer(kafkaServers: String, topicName: String) {
    val nodeUri = System.getenv("NODE_URI") ?: Defaults.NODE_URI
    val netAdapter = okHttpNetAdapter(nodeUri)
    val decoderAdapter = moshiDecoderAdapter()

    // the block height to start listening from
    // in practice, this should be tracked via some persistent storage system so that you can pick up where you last
    // left off on the event stream. If this value is unset, this will start listening for live blocks only
    val startingBlockHeight = System.getenv("START_HEIGHT")?.toLong()
    println("Listening for events $EVENTS from height $startingBlockHeight")

    val producerProps = mapOf(
        CommonClientConfigs.BOOTSTRAP_SERVERS_CONFIG to kafkaServers,
        CommonClientConfigs.CLIENT_ID_CONFIG to ("test0"),
        ProducerConfig.ACKS_CONFIG to "all",
        ProducerConfig.ENABLE_IDEMPOTENCE_CONFIG to true,
    )

    val blockSink = kafkaBlockSink(producerProps, topicName)

    // initialize the event stream flow
    when (startingBlockHeight) {
        null -> liveBlockFlow(netAdapter, decoderAdapter) // no starting height, listen for live blocks
        else -> blockFlow(netAdapter, decoderAdapter, from = startingBlockHeight) // starting height provided, use combined historical/live flow
    }.onEach { blockData ->
        // we have received a block, drop it in the KafkaBlockSink
        println("Sending block to kafka topic (height: ${blockData.height})")
        blockSink.invoke(StreamBlockImpl(
            blockData.block,
            blockData.blockResult.blockEvents(blockData.block.dateTime()),
            blockData.blockResult.txsResults,
            blockData.blockResult.txEvents(blockData.block.dateTime()) { index -> blockData.block.txData(index) },
            blockData.blockResult.txErroredEvents(blockData.block.dateTime()) { index -> blockData.block.txData(index) },
        ))
    }.catch { println("received error when streaming blocks to kafka, ${it.message}") }
    .collect()
}

/**
 * A Kafka consumer reading block data off of the topic
 */
suspend fun startConsumer(kafkaServers: String, topicName: String) {
    val consumerProps = mapOf(
        CommonClientConfigs.BOOTSTRAP_SERVERS_CONFIG to kafkaServers,
        CommonClientConfigs.CLIENT_ID_CONFIG to ("test1"),
        ConsumerConfig.GROUP_ID_CONFIG to "group0",
    )

    val source = kafkaBlockSource(consumerProps, topicName)

    source.streamBlocks()
        .onEach { streamBlock ->
            // we got a block from kafka, act on it as necessary for your use case
            println("Received block from kafka! (height: ${streamBlock.height})")
            streamBlock.txEvents.joinToString("\n") { txEvent ->
                "${txEvent.eventType}: " + txEvent.attributes.joinToString("\n\t", prefix = "[\n\t", postfix = "\n]") { attribute -> "${attribute.key?.decodeBase64()}: ${attribute.value?.decodeBase64()}" }
            }.let(::println)
            streamBlock.ack()
        }.catch { println("received error when streaming blocks from kafka, ${it.message}") }
        .collect()
}
