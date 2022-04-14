# Provenance Contract Migration Example

This example repository contains a fully-functioning smart contract that demonstrates how to add a `migrate` function to a smart contract, as
well as enact a migration on a self-hosted localnet.

## Project Prerequisites

This project assumes that the user has already followed all the `Contract Setup` steps from the [Provenance Smart Contract Example](../provenance-smart-contract-example) directory.  The contract from that step should already be running on a self-hosted localnet.

## Migration Setup

At this point, you should have a smart contract running that has no migration capabilities.  No worries!  The contract from this directory has you covered!  Let's get those features added!

### Step #1: Build the WASM

Migrating a smart contract starts similarly to instantiating a new one.  In order to migrate an existing contract, you must first obtain a compiled WASM to store on the Provenance blockchain.  From this directory, run:
```sh
make optimize
```

Like in the previous example, a temporary Docker container will be created to run `rust-optimizer` and create a new WASM.  After the command completes successfully, run the following to verify that it exists and is below the 600K threshold required for upload to Provenance:

```sh
ls -lf ./artifacts/provenance_contract_migration_example.wasm
```

### Step #2: Store the WASM

Just like before, the contract will need to be stored on the Provenance blockchain.  In order for an existing contract to be migrated to a new contract, the WASM must be stored and have a valid `code_id`.  

Remember, you will need to be in the `provenance` repository's directory to execute `provenanced` commands with a `home` of `build/node0`.  This is because the repository's `make localnet-start` command automatically creates a nhash-funded account called `node0`, and the `build/node0` directory in this repository contains the relevant key information in order for you to act upon that account's behalf.  If you no longer have the `node0` variable established with the proper address, you can find it again with:
```sh
export node0=$(provenanced keys show -a node0 --home build/node0 --testnet)
```

Store it with the following command:
```sh
provenanced tx wasm store my/path/to/how-to-provenance/provenance-contract-migration-example/artifacts/provenance_contract_migration_example.wasm \
--instantiate-only-address "$node0" \
--from node0 \
--home build/node0 \
--chain-id chain-local \
--gas auto \
--gas-prices="1905nhash" \
--gas-adjustment=1.2 \
--broadcast-mode block \
--testnet \
--output json \
--yes | jq
```

Like before, you'll need your `code_id`.  So, either sift through that output json to find it, or adapt the previous command with `jq -r '.logs[] | select(.msg_index == 0) | .events[] | select(.type == "store_code") | .attributes[0].value'` at the end.  If you are following these guides sequentially, your code id from this storage will likely be `2`.

### Step #3: Setup a Fee Collector Account

One of the new features of the smart contract is that it can charge a fee when the counter is incremented.  Where should that fee go, though?  Well, fortunately, due to this environment being a localnet, we have TOTAL CONTROL!  Let's abuse some of that power to create ourselves a new account.

One of the easiest ways to build a new account is to generate a mnemonic and import it as a key.  An account can be build without generating a mnemonic, but then we'll lose access to that mnemonic in the future for rebuilding the account.  Fortunately, the `provenanced` tool includes some useful features to generate these things for us.

Start by generating yourself a mnemonic:
```sh
export my_mnemonic=$(provenanced keys mnemonic) && echo $my_mnemonic
```

Your mnemonic should look something like this:
```sh
rich mimic gravity wool moon above round insect rookie bridge feed blast later rubber remain make cram random regret before edit ten firm spike
```

Now that you have your own mnemonic, it's time to import it into the `build/node0` directory keychain for usage in your `provenanced` commands.  Run the following:
```sh
provenanced keys add my_fee_collector \                                                                                  (testfiguretech/onboarding)
--home build/node0 \
--hd-path "44'/1'/0'/0/0" \
--recover \
--testnet \
--output json | jq -r '.address'
```

You will be prompted to enter the mnemonic from the previous command.  It's also totally fine to just use the mnemonic from this guide.  Now, with this address in your keychain, you can execute provenanced commands with `--from my_fee_collector` and `--home build/node0`.  You're now enabled to sign your own transactions!  After running this command, you should see output like:
```sh
tp16ha0up3mgnqvespturrzmdgw7mgqxq23vfc4gv
```

That's the address of your new account!  *IMPORTANT:* Make sure to save it to an environment variable for executing the next command, like:
```sh
export my_fee_collector=tp16ha0up3mgnqvespturrzmdgw7mgqxq23vfc4gv
```

### Step #4: Migrate the Contract

To run a migration, you'll need three primary things:
- 1: The address of the contract being migrated.  If you're following the guide sequentially, you may still have that value stored in the `contract_address` environment variable.  However, if not, you can always find your contract's address by resolving its name.  The contract itself establishes a name when it is instantiated.  The previous example use the name `examples.pio`, so we'll use that in this example command for finding the address dynamically:
```sh
export contract_address=$(provenanced q name resolve examples.pio --testnet --output json | jq -r '.address')
```
- 2: The `code_id` of the WASM that's intended to be migrated.  For the purposes of this example, we will assume the guide is being followed and this newly-stored contract code generated a `code_id` of `2`.
- 3: Control over the admin account of the contract.  Due to these commands being run in the `provenance` repository with its pre-existing `build/node0` values, you will already have the proper control needed to migrate the contract.

Now, let's get cookin'!  Let's migrate the contract to the new codebase, and include all possible optional parameters in order to get as spicy as we can!  Run the following:
```sh
provenanced tx wasm migrate \
"$contract_address" \
2 \
'{
  "new_counter_value": "1000",
  "increment_counter_fee": {
    "fee_collector_address": "'"$my_fee_collector"'",
    "fee_collection_amount": {
      "amount": "100",
      "denom": "nhash"
    }
  }
}' \
--from node0 \
--chain-id chain-local \
--gas auto \
--gas-prices 1905nhash \
--gas-adjustment 1.2 \
--home build/node0 \
--broadcast-mode block \
--testnet \
--output json \
--yes | jq
```

Upon a success, you'll get a cozy set of output attributes:
```json
"attributes": [
  {
    "key": "_contract_address",
    "value": "tp14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s96lrg8"
  },
  {
    "key": "action",
    "value": "migrate"
  },
  {
    "key": "new_version",
    "value": "0.0.2"
  },
  {
    "key": "modified_counter_value",
    "value": "1000"
  },
  {
    "key": "modified_increment_counter_fee_address",
    "value": "tp16ha0up3mgnqvespturrzmdgw7mgqxq23vfc4gv"
  },
  {
    "key": "modified_increment_counter_fee_amount",
    "value": "100nhash"
  }
]
```

To verify that all those cool new features got added, let's query a new endpoint that was added:
```sh
provenanced q wasm contract-state smart \
"$contract_address" \
'{"query_version": {}}' \
-t --output=json | jq
```

You should see that the new `VersionInfo` functionality was added:
```json
{
  "data": {
    "contract": "provenance_contract_migration_example",
    "version": "0.0.2"
  }
}
```

Additionally, the `FeeCollectionDetail` should be added to the state.  All counter increments should now cost 100nhash and pay to the account setup in step #3.  Let's query up the state and make sure that info is in there.  Additionally, the counter value should now be set to 1000.

```sh
provenanced query wasm contract-state smart \
"$contract_address" \
'{"query_state": {}}' \
--testnet \
--output json | jq
```

You should see the much-more-exciting output of:
```json
{
  "data": {
    "contract_base_name": "examples.pio",
    "contract_counter": "1000",
    "increment_counter_fee": {
      "fee_collector_address": "tp16ha0up3mgnqvespturrzmdgw7mgqxq23vfc4gv",
      "fee_collection_amount": {
        "denom": "nhash",
        "amount": "100"
      }
    }
  }
}
```

### Step #4: Increment that Counter!
Now that the migration has been completed, executing the increment counter execution route should now cost 100nhash, and those funds should be immediately delivered to the address stored in your `$my_fee_collector` environment variable.

To verify that fee collector account has no funds, first query up its balances using the `bank` module:
```sh
provenanced query bank balances "$my_fee_collector" --testnet --output json | jq
```

Unless you were experimenting with `provenanced` between commands in this example, the result should be akin to the following JSON payload.  This indicates the fee collector currently has no funds whatsoever.
```json
{
  "balances": [],
  "pagination": {
    "next_key": null,
    "total": "0"
  }
}
```

Let's invoke the `increment_counter` route with some provided funds, using the already-funded `node0` account.  This time, we'll omit the `increment_amount` value and let the contract use its default value of incrementing by 1.  It's important to note the `--amount 100nhash` flag on this request.  This tells provenance that `100nhash` should be supplied from `node0` and sent into the contract.  The contract is coded to immediately send those funds to the previous-specified fee address.  Run the following:
```sh
provenanced tx wasm execute \
"$contract_address" \
'{"increment_counter": {}}' \
--from node0 \
--home build/node0 \
--chain-id chain-local \
--amount 100nhash \
--gas auto \
--gas-prices="1905nhash" \
--gas-adjustment=1.2 \
--broadcast-mode block \
--testnet \
--output json \
--yes | jq
```

Take some time to inspect the resulting JSON.  You should notice that there is a `coin_received` node.  This indicates that the contract received coin from the `node0` account.  You should also notice a `coin_spent` node.  This indicates that the contract spent some coin as well.  Based on the code, we know that the contract spent all of the funds that it received to send them to the fee collector.

Now, let's verify that the fee collector received those funds!

Run the following:
```sh
provenanced query bank balances "$my_fee_collector" --testnet --output json | jq
```

The command, before, showed that the fee collector had no funds at all.  Now, you should see that it has the 100nhash that was supplied to the contract.  This is further evidence that the contract did not withhold any of the funds that were sent into it:
```json
{
  "balances": [
    {
      "denom": "nhash",
      "amount": "100"
    }
  ],
  "pagination": {
    "next_key": null,
    "total": "0"
  }
}
```

Well done!  You can now use the `provenanced` client to store, instantiate, execute, query and migrate smart contracts.  You also learned how to use a few other modules along the way!  The contract contains a few other endpoints: `add_attribute` and `send_funds`.  Feel free to use the execution logic you learned here to try out those other execution routes.

Optional:  The execution routes map directly to the `ExecuteMsg` struct in the [msg.rs](src/msg.rs) file.  Additionally, the queries are mapped to the `QueryMsg` struct in that file.  To add a new route, add a new enum variant to these messages, and map its functionality in the [contract.rs](src/contract.rs) file.  

Reminder:  The [provenance smart contract tutorial](https://github.com/provenance-io/provwasm/blob/main/docs/tutorial/01-overview.md) contains an in-depth explanation of the inner-workings of a smart contract.  If you're interesting, that's a great place to start building your knowledge, and, more importantly, your own smart contract!