# Bilateral Trade Example

This folder contains an example contract that can be used to transfer value between two parties without the need for an
intermediary to hold the asset/funds (the contract acts as escrow and holds the thing for sale and the funds purchasing that thing)
and then swaps the two to the appropriate parties atomically in one transaction.

## Index
1. [Bilateral Exchange Contract](src/contract.rs): Please see the [Provenance Smart Contract Example](../provenance-smart-contract-example) for details on how to store/instantiate a smart contract
2. [Scope for Coin Exchange](examples/kotlin/scope-exchange/src/main/kotlin/ScopeExchange.kt): An example of trading a scope via the contract in exchange for some coin
3. [Marker-owned Scope exchange via Marker's coin for other coin](examples/kotlin/scope-exchange/src/main/kotlin/MarkerOwnedScopeExchange.kt): An example of trading a marker's coins for some other coin as a proxy for exchanging scope value

## Step-by-step examples on how to run this repo
