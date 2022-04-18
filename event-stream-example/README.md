# Event Stream Examples

The [Provenance Event Stream Library](https://github.com/provenance-io/event-stream) provides an interface for consuming
events resulting from activity on the Provenance Blockchain.

This directory contains a collection of examples showing usage of the event stream. The basic structure of Provenance
Blockchain events are as follows:
* A block may contain zero or more transactions
* A transaction may contain one or more messages
* A message may result in the emission of zero or more events of various types pertaining to the actions executed as
  part of that message
* An event may contain zero or more attributes (key-value pairs) containing details about the corresponding event

The goal of the Event Stream Library is to make it easy to consume these events and take action on any events that are
relevant to the needs of your application. A block is cut approximately every 5 seconds, so under normal circumstances
listening to live blocks you should receive one block every 5 seconds. If you are catching up from a historical block
height, you can expect to receive a much faster stream of blocks until you catch up with the live height.

You may find it useful to look at various transactions on [Provenance Explorer](https://explorer.provenance.io/txs) to
understand what events result from transactions you are interested in for your application.

Please see the [Event Stream Readme](https://github.com/provenance-io/event-stream#readme) for further documentation

## Project Prerequisites
* Java JDK 11 (install via an sdk manager, like [SdkMan](https://sdkman.io/))
* [Gradle](https://gradle.org/install/) for building/running the examples
* [Docker Compose](https://docs.docker.com/compose/) in order to spin up supporting docker-compose setup for the kafka example

## Kafka
If you make use of [Kafka](https://kafka.apache.org/) in your system, the Event Stream provides a
[Kafka Connector](https://github.com/provenance-io/event-stream/tree/main/es-kafka) that can be used to feed events
directly into a Kafka topic and consumed downstream. Please see the [KafkaConsumerExample](src/main/kotlin/io/provenance/example/KafkaConsumerExample.kt)
for an example usage.

## Example Index
1. [SimpleEventStreamListener](src/main/kotlin/io/provenance/example/SimpleEventStreamListener.kt): a simple example of
   listening to the event stream and filtering down to blocks relevant to your application
2. [KafkaConsumerExample](src/main/kotlin/io/provenance/example/KafkaConsumerExample.kt): a simple producer/consumer setup
   illustrating publishing blocks to/reading blocks from a Kafka topic. Note: There is an included [Docker Compose](https://docs.docker.com/compose/)
   setup to stand up a local single-node kafka instance for testing purposes, can be run using the [kafka.yml](src/main/docker/kafka.yml)
   configuration (i.e. `docker compose -f src/main/docker/kafka.yml up -d` and then `docker compose -f src/main/docker/kafka.yml down` to stop)

To run an example, simply run `./gradlew run -PmainClass=<className>` where `className` is the fully-qualified class name of the example file (i.e. `io.provenance.example.KafkaConsumerExampleKt` [note the `Kt`suffix])

Each example contains various configuration options that can be used to control the node to connect to for the event stream,
kafka connection/topic information, etc.
