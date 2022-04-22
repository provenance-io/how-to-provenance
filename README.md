# How to Provenance

Welcome to the 'How To Provenance' repository, where you can find examples of Provenance Blockchain usage, Provenance Blockchain smart contract development, Provenance Blockchain application development and related topics.

The Provenance Blockchain Foundation is always eager for contributors.  If you have an idea not covered by these examples, we want to hear about it!  Love our software and want to contribute new functionality?  Make a proposal through our [Grants Program](https://provenance.io/grants)!

## Index
1. [Authz Example](authz-example): (Kotlin/Gradle) Use the blockchain [Authz Module](https://docs.cosmos.network/master/modules/authz) to grant privileges from one account to another, including delegated signing and fee allowances.
2. [Bilateral Trade Example](bilateral-trade-example): (Rust/Cargo) Execute a smart contract that trades coins between two parties, demonstrating a complete transfer of funds without the need for another intermediary entity.
3. [Create NFT Example](create-nft-example): (Kotlin/Gradle) Use the [Provenance Blockchain Metadata Module](https://docs.provenance.io/modules/metadata-module) with [scopes](https://docs.provenance.io/modules/metadata-module#scope-data-structures) to store NFTs.
4. [Event Stream Example](event-stream-example): (Kotlin/Gradle) Use the event stream library to watch for Provenance Blockchain blocks, acting as an external consumer to observe blockchain actions asynchronously.
5. [HDWallet Example](hdwallet-example): (Kotlin/Gradle) Use Provenance Blockchain's [HDWallet library](https://github.com/provenance-io/hdwallet) to generate mnemonics and sign payloads.
6. [P8e Contract + SDK Example](p8e-contract-sdk-example): (Kotlin/Gradle/Docker) Run local [docker](https://www.docker.com) containers to use the [Contract Execution Environment](https://docs.provenance.io/p8e/overview) and how to run a contract.  This example illustrates how with [p8e](https://github.com/provenance-io/p8e) one might execute complex data storage and manipulation using Kotlin as a communication layer with the Provenance Blockchain.
7. [Provenance Smart Contract Example](provenance-smart-contract-example): (Rust/Cargo) Store and instantite a smart contract on the [Provenance Blockchain](https://github.com/provenance-io/provenance) written in Rust. Uses the [provwasm library](https://github.com/provenance-io/provwasm) to do Provenance Blockchain interactions, like using the [name module](https://docs.provenance.io/modules/name-module) and adding [attributes](https://docs.provenance.io/modules/account) to persist data directly on the blockchain.
8. [Provenance Contract Migration Example](provenance-contract-migration-example): (Rust/Cargo) Migrate the code of an existing smart contract on the Provenance Blockchain to a new version.  A continuation of the [Provenance Smart Contract Example](provenance-smart-contract-example).
9. [Name Service dApp](name-service-dapp-example): Run a simple dApp that registers human-readable names to an account using a smart contract, the Provenance Wallet (to be publicly available soon) and a [React](reactjs.org) frontend (no backend other than the blockchain itself)

## Examples: Ordered by Domain Knowledge

Each example is listed below in order of knowledge needed to proceed, leveled:

### 100
- 100: [Provenance Smart Contract Example](provenance-smart-contract-example)
- 101: [Provenance Contract Migration Example](provenance-contract-migration-example)
- 102: [Event Stream Example](event-stream-example)

### 200
- 200: [Bilateral Trade Example](bilateral-trade-example)
- 201: [Create NFT Example](create-nft-example)
- 202: [P8e Contract + SDK Example](p8e-contract-sdk-example)

### 300
- 300: [HDWallet Example](hdwallet-example)
- 301: [Authz Example](authz-example)

### 400
- 400: [Name Service dApp](name-service-dapp)
