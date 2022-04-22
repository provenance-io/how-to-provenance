# Kotlin Scope Exchange Examples

## Project Prerequisites
* Java JDK 11 (install via an sdk manager, like SdkMan)
* Gradle for building/running the examples

## Examples
1. [Scope for Coin Exchange](src/main/kotlin/ScopeExchange.kt): Connects to the Provenance Blockchain testnet and trades a [scope](https://docs.provenance.io/modules/metadata-module#scope-data-structures) owned by an [account](https://docs.provenance.io/blockchain/basics/accounts) to another account in exchange for coin.  Use the `ScopeExchangeKt` class to launch this example.
2. [Marker-owned Scope exchange via Marker's coin for other coin](examples/kotlin/scope-exchange/src/main/kotlin/MarkerOwnedScopeExchange.kt): An example of trading a [marker](https://docs.provenance.io/modules/marker-module)'s coins for some other coin as a proxy for exchanging scope value. Use the  `MarkerOwnedScopeExchangeKt` class to launch this example.

## Running an Example

To run an example, execute the following command.  Note the `Kt` suffix.  The Kotlin compiler appends this suffix to the file name when it builds the class files.
```shell
./gradlew run -PmainClass=<example class name>
```
