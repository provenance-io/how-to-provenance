# Provenance Blockchain Scope (NFT) Creation Example

Provenance Blockchain's metadata module contains a core concept called a 'scope'. This is essentially a non-fungible token
(NFT) with a well-defined structure and ownership constructs that can contain records pointing to data off-chain.

A Scope may contain zero or more 'Sessions', which are references to some off-chain process that generated/controls
a set of 'Records'. A Record is a reference to some data off-chain with some metadata about the inputs that
went into generating that record, as well as outputs that were generated, along with the datatypes that these references
represent and information about the process/method that generated those references.
Typically a reference to data off-chain is in the form of a hash of binary data (traditionally serialized Protobuf
Messages).

For more information about the Metadata Module, please reference the [Provenance Blockchain Docs](https://docs.provenance.io/modules/metadata-module).

To run the example, use the following command `./gradlew run`. Once you have run this once, you may re-use the same scope
specification id by setting the env var `SCOPE_SPEC_UUID` (i.e. `SCOPE_SPEC_UUID=<your_uuid_here> ./gradlew run`). The
example has defaults for your local chain-id and node uri, but you may override these as need be