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
relevant to the needs of your application.

If you make use of [Kafka](https://kafka.apache.org/) in your system, the Event Stream provides a
[Kafka Connector](https://github.com/provenance-io/event-stream/tree/main/es-kafka) that can be used to feed events
directly into a Kafka topic and consumed downstream.

Please see the [Event Stream Readme](https://github.com/provenance-io/event-stream#readme) for further documentation
