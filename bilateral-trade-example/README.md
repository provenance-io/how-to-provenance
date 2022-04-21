# Bilateral Trade Example

This folder contains an example contract that can be used to transfer value between two parties without the need for an
intermediary to hold the asset/funds (the contract acts as escrow and holds the thing for sale and the funds purchasing that thing)
and then swaps the two to the appropriate parties atomically in one transaction.

## Index
1. [Bilateral Exchange Contract](src/contract.rs): Please see the [Provenance Smart Contract Example](../provenance-smart-contract-example) for details on how to store/instantiate a smart contract
2. [Kotlin Examples](./examples/kotlin/scope-exchange): Each example in this directory is a representation of how to interact with a stored smart contract via Kotlin, using Provenance Blockchain's GRPC [PbClient](https://github.com/provenance-io/pb-grpc-client-kotlin):
   1. [Scope for Coin Exchange](src/main/kotlin/ScopeExchange.kt): Connects to the Provenance Blockchain testnet and trades a [scope](https://docs.provenance.io/modules/metadata-module#scope-data-structures) owned by an [account](https://docs.provenance.io/blockchain/basics/accounts) to another account in exchange for coin.
   2. [Marker-owned Scope exchange via Marker's coin for other coin](examples/kotlin/scope-exchange/src/main/kotlin/MarkerOwnedScopeExchange.kt): An example of trading a [marker](https://docs.provenance.io/modules/marker-module)'s coins for some other coin as a proxy for exchanging scope value
