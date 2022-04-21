# Provenance Blockchain Scope (NFT) Creation Example

Provenance Blockchain's metadata module contains a core concept called a ['scope'](https://github.com/provenance-io/provenance/tree/main/x/metadata/spec). 
This is essentially a non-fungible token (NFT) with a well-defined structure and ownership constructs that can contain 
records pointing to data off-chain.  As with all Provenance Blockchain data structures, a scope is defined as a [protobuf message](https://github.com/provenance-io/provenance/blob/main/docs/proto-docs.md).

A Scope may contain zero or more 'Sessions', which are references to some off-chain process that generated/controls
a set of 'Records'. A Record is a reference to some data off-chain with some metadata about the inputs that
went into generating that record, as well as outputs that were generated, along with the datatypes that these references
represent and information about the process/method that generated those references.
Typically a reference to data off-chain is in the form of a hash of binary data (traditionally serialized Protobuf
Messages).

For more information about the Metadata Module, please reference the [Provenance Blockchain Docs](https://docs.provenance.io/modules/metadata-module).

## Project Prerequisites
* [Provenance Blockchain Environment](https://github.com/provenance-io/provenance): See the [provenance-smart-contract-example](../provenance-smart-contract-example) for a guide on running a local Provenance Blockchain environment.
* Java JDK 11 (install via an sdk manager, like [SdkMan](https://sdkman.io/))
* [Gradle](https://gradle.org/install/) for building/running the examples

## Running the Project

To run the example, use the following command:

```shell
./gradlew run
```

## Optional Arguments

This project uses various environment variables to customize its execution.  None of these values need to be provided
in order for the application to run correctly.  The values are as follows:

* `CHAIN_ID`: The blockchain identifier to which the application connects.  The default value is `chain-local`.
* `NODE_URI`: The blockchain node to which the application connects.  The default value is `grpc://localhost:9090`
* `SCOPE_SPEC_UUID`: The UUID value that is used to derive the [scope specification](https://docs.provenance.io/modules/metadata-module#scope-specification)'s bech32 address.  The default value is randomly generated.  The application prints this value during execution for re-use in multiple runs.

To run the application with a specified environment variable, simply add it to the beginning of the gradle script execution:

```shell
SCOPE_SPEC_UUID=<your_uuid_here> ./gradlew run
```
