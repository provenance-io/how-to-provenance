# Bilateral Trade Example

This folder contains an example contract that can be used to transfer value between two parties without the need for an
intermediary to hold the asset/funds (the contract acts as escrow and holds the thing for sale and the funds purchasing that thing)
and then swaps the two to the appropriate parties atomically in one transaction.

## Index
1. [Bilateral Exchange Contract](src/contract.rs): Please see the [Provenance Smart Contract Example](../provenance-smart-contract-example) for details on how to store/instantiate a smart contract
2. [Kotlin Examples](./examples/kotlin/scope-exchange): Each example in this directory is a representation of how to interact with a stored smart contract via Kotlin, using Provenance Blockchain's GRPC [PbClient](https://github.com/provenance-io/pb-grpc-client-kotlin):
   1. [Scope for Coin Exchange](src/main/kotlin/ScopeExchange.kt): Connects to the Provenance Blockchain testnet and trades a [scope](https://docs.provenance.io/modules/metadata-module#scope-data-structures) owned by an [account](https://docs.provenance.io/blockchain/basics/accounts) to another account in exchange for coin.
   2. [Marker-owned Scope exchange via Marker's coin for other coin](examples/kotlin/scope-exchange/src/main/kotlin/MarkerOwnedScopeExchange.kt): An example of trading a [marker](https://docs.provenance.io/modules/marker-module)'s coins for some other coin as a proxy for exchanging scope value

## Message Structure Quick Reference
Note: Each message is described fully in the [Schema](schema) directory, as well as in the [msg.rs](src/msg.rs) file.

1. _Instantiate_:

```json
{
   "bind_name": "somename.sc.pb",
   "contract_name": "A descriptive name for this contract",
   "ask_fee": "100",
   "bid_fee": "250"
}
```

2. _Create Ask_:

_Note_: 
If no scope address is provided, the request must include funds that constitute the `base` that the bidder will
be matched with in their `coin` variant declaration of the `base` in their message.  

If a scope address is provided, the asker must not provide any funds, and must transfer ownership of a Provenance 
Blockchain Metadata Scope to the smart contract's bech32 address, setting it as the owner in the scope's `owners` 
array, as well as the scope's `value_owner_address` value.  It is recommended that this transfer be done in the same
transaction as the execution of the contract's `create_ask` message, to ensure that the scope does not get transferred
to the contract with no record of doing so if the contract's validation rejects the `create_ask`.

```json
{
   "create_ask": {
      "id": "my-ask-id",
      "quote": [{
         "amount": "150",
         "denom": "biddercoin"
      }],
      "scope_address": "scope1qzrptuwxpht3rmv42ape63wesgfsntxa5h"  
   }
}
```

3. _Create Bid_:

_Coin Variant_: If the asker omitted a scope address and provided `base` funds, then the bidder must choose the `coin`
variant and provide a coin amount that matches the asker's `base`, as well as funds that match the asker's `quote`.

```json
{
   "create_bid": {
      "id": "my-bid-id",
      "base": {
         "coin": {
            "coins": [{
               "amount": "200",
               "denom": "askercoin"
            }]
         }
      },
      "effective_time": "1658945674349115000"
   }
}
```

_Scope Variant_: If the asker provided a Provenance Blockchain Metadata Scope and no `base` funds, then the bidder must 
provide funds to constitute the asker's `quote` and directly refer to the scope address required in the trade using the 
`scope` variant.

```json
{
   "create_bid": {
      "id": "my-bid-id",
      "base": {
         "scope": {
            "scope_address": "scope1qzrptuwxpht3rmv42ape63wesgfsntxa5h"
         }
      },
      "effective_time": "1658945674349115000"
   }
}
```

4. _Execute Match_:

_Note_: Only the contract's admin account may execute matches.

```json
{
   "execute_match": {
      "ask_id": "my-ask-id",
      "bid_id": "my-bid-id"
   }
}
```

5. _Cancel Ask_: 

```json
{
   "cancel_ask": {
      "ask_id": "my-ask-id"
   }
}
```

6. _Cancel Bid_:

```json
{
   "cancel_bid": {
      "bid_id": "my-bid-id"
   }
}
```

7. _Get Ask_:

```json
{
   "get_ask": {
      "id": "my-ask-id"
   }
}
```

8. _Get Bid_: 

```json
{
   "get_bid": {
      "id": "my-bid-id"
   }
}
```

9. _Get Contract Info_:

```json
{
   "get_contract_info": {}
}
```

10. _Migrate Contract_:

```json
{
   "new_version": {}
}
```
