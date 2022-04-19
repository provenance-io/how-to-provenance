# Authz Example

Authz is a module provided by the Cosmos SDK, and utilized by the Provenance Blockchain in order to allow accounts to 
delegate authorization to execute transactions and take actions on each others' behalf.

This example illustrates how to use the Authz module to allow gas and message fees to be paid by an account other than
the signer of a transaction, as well as using Authz to allow an account to execute transactions with the privileges of
another account.

## Project Prerequisites
- Java JDK 11 (install via an sdk manager, like [SdkMan](https://sdkman.io/))
- [Gradle](https://gradle.org/install/): In order to build and execute examples
- [Provenance](https://github.com/provenance-io/provenance): An understanding of the ecyosystem and the ability to connect to a node
- Two provenance accounts, funded with enough hash to execute messages to bind names, create scope specifications, etc

## Running the Example

To run the example codebase, simply execute the gradle wrapper from this directory as an application:
```shell
./gradlew run -q
```

## Local Provenance Setup

If running the project locally from the provenance repository, the following commands can establish the required 
accounts and their respective mnemonics:

- Export the `node0` address to use for bank transfers:
```shell
export node0=$(provenanced keys show -a node0 --home build/node0 --testnet)
```

- Create a "main account" to use in the examples (remember to record the mnemonic somewhere - it's a prompt in the examples):
```shell
# Generate a mnemonic
main_account_mnemonic=$(provenanced keys mnemonic)
echo "Main account mnemonic: $main_account_mnemonic"
# Pipe the mnemonic as input to an account recovery command
echo "$main_account_mnemonic" | provenanced keys add main-account --home build/node0 -t --hd-path "44'/1'/0'/0/0'" --output json --recover | jq
# Record the main account's address in a variable for use in funding
main_account_address=$(provenanced keys show -a main-account --home build/node0 --testnet)
echo "Main account address: $main_account_address"
```

- Fund the "main account" using `node0`:
```shell
provenanced tx bank send \
    "$node0" \
    "$main_account_address" \
    200000000000nhash \
    --from node0 \
    --home build/node0 \
    --chain-id chain-local \
    --gas auto \
    --gas-prices="1905nhash" \
    --gas-adjustment=1.2 \
    --broadcast-mode block \
    --yes \
    --testnet \
    --output json | jq
```

- Create a "helper account" to use in the examples (remember to record the mnemonic somewhere - it's a prompt in the examples):
```shell
# Generate a mnemonic
helper_account_mnemonic=$(provenanced keys mnemonic)
echo "Helper account mnemonic: $helper_account_mnemonic"
# Pipe the mnemonic as input to an account recovery command
echo "$helper_account_mnemonic" | provenanced keys add helper-account --home build/node0 -t --hd-path "44'/1'/0'/0/0'" --output json --recover | jq
# Record the helper account's address in a variable for use in funding
helper_account_address=$(provenanced keys show -a helper-account --home build/node0 --testnet)
echo "Helper account address: $helper_account_address"
```

- Fund the "helper account" using `node0`:
```shell
provenanced tx bank send \
    "$node0" \
    "$helper_account_address" \
    200000000000nhash \
    --from node0 \
    --home build/node0 \
    --chain-id chain-local \
    --gas auto \
    --gas-prices="1905nhash" \
    --gas-adjustment=1.2 \
    --broadcast-mode block \
    --yes \
    --testnet \
    --output json | jq
```

Success! With both accounts created and funded, all prompts should execute as desired when using the LOCAL PbClient
option.
