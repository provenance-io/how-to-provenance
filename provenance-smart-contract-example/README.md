# Provenance Smart Contract Example

This example repositoriy contains a fully-functioning demo smart contract that utilizes Provenance's
[provwasm](https://github.com/provenance-io/provwasm) library to interact with a Provenance blockchain
instance.

## Project Prerequisites
- [Rust](https://doc.rust-lang.org/book/): Install Rust and the [Cargo](https://doc.rust-lang.org/book/ch01-03-hello-cargo.html) tool. Cosmwasm contracts are built in the Rust programming language. An understanding of how the language works, as well as its `Cargo` tool, are very important to understanding this smart contract and its contained source code.
- [Make](https://www.gnu.org/software/make/manual/make.html): Install the `Make` build tool.  The various build and test commands for this smart contract are contained within an accompanying [Makefile](./Makefile). An understanding of how to use the various `make` commands will facilitate contract usage.
  - Windows: http://gnuwin32.sourceforge.net/packages/make.htm
  - Mac: https://formulae.brew.sh/formula/make
- [Docker](https://docs.docker.com/get-started/): A completed smart contract must be compiled into a `.wasm` file for storage on the Provenance blockchain.  In order to do this, a temporary `Docker` container is created that runs a tool called [rust-optimizer](https://github.com/CosmWasm/rust-optimizer) that was created by the [cosmwasm](https://github.com/CosmWasm/cosmwasm) team.  Without a running `Docker` environment, this command `make optimize` will fail.
- [Go](https://go.dev/): Install golang.  The Provenance codebase uses golang for many of its actions, including the localnet servers that will be started for using this contract.
- [Git](https://docs.github.com/en/get-started/quickstart/set-up-git): Install the `git` CLI tooling.  This will be needed to locally clone the `Provenance` repository.
- [Provenance](https://github.com/provenance-io/provenance): Clone the repository. To demonstrate the usage of this contract, as well as setting up a local environment to interact with a localnet blockchain.

## Contract Setup
With all of the prerequisites covered, you should be ready to get everything running!

### Step #1: Compile the WASM
- Navigate to the [root directory of this project](./).

- Run the command `make optimize` from your commandline of choice.  Upon a successful run, this will present messages similar to:
```sh
Info: RUSTC_WRAPPER=sccache
Info: sccache stats before build
Compile requests                      0
Compile requests executed             0
Cache hits                            0
Cache misses                          0
Cache timeouts                        0
Cache read errors                     0
Forced recaches                       0
Cache write errors                    0
Compilation failures                  0
Cache errors                          0
Non-cacheable compilations            0
Non-cacheable calls                   0
Non-compilation calls                 0
Unsupported compiler calls            0

...

Cache location                  Local disk: "/root/.cache/sccache"
Cache size                            0 bytes
Max cache size                       10 GiB
done
```

- Check the size of WASM is acceptable for storage on the Provenance blockchain.  The WASM will be built and added to an `artifacts` directory in this project.  WASM files must be at most 600K to be accepted for storage on Provenance.  An example command for a unix-based system to check this would be:
```sh
ls -lf artifacts/provenance_smart_contract_example.wasm
```

You should see a file smaller than 600K in your `artifacts` directory.  If so, congratulations! You've successfully created your first smart contract, and are now ready to instantiate it on the Provenance blockchain.

### Step #2: Install Provenanced
To interact with the Provenance blockchain, Provenance has developed a commandline tool named `provenanced` that can interact with all of Provenance's modules.  Before starting a localnet, let's install this CLI tool to ensure we can easily install and interact with the smart contract.

- Navigate to the `provenance` directory that you installed in the prerequisites section.

- Ensure the `main` branch is checked out.  The command: `git rev-parse --abbrev-ref HEAD` should print `main`.  If it does not, simply run:
```sh
git checkout main
```

- Install the `provenanced` command.  Run the following command from the root of the `provenance` directory:
```sh
make install
```

- Verify that `provenanced` is installed.  Running `provenanced version` should produce something like: `main-<commit-hash>`.

Woot! You now have a way to interact with Provenance, and you have a WASM that's ready to install!

### Step #3: Run a Localnet
The `provenanced` can connect to and interact with live provenance nodes, but for the sake of this example, we're going to create ourselves a localnet to play around with.

- From the root of the `provenance` directory, run the command:
```sh
make localnet-start
```

You should see many logs that describe the process of starting the local containers, and, if successful, the logs should end with these messages:
```sh
Successfully initialized 4 node directories
docker-compose -f networks/local/docker-compose.yml --project-directory ./ up -d
Creating network "provenance_localnet" with driver "bridge"
Creating node0 ... done
Creating node1 ... done
Creating node3 ... done
Creating node2 ... done
``` 

- Check that your localnet is running correctly by displaying the node0 address. Run the following command:
```sh
provenanced keys show -a node0 --home build/node0 --testnet
``` 
If everything is running correctly, you should see a bech32 address, like: `tp13gzxe0cyqp70uedjrqhau2w93cjw8zr4ju4c95`.  Excellent!

### Step #4: Store the Smart Contract
- For simplicity's sake, this example will use the `node0` address to manage the smart contract and do executions.  Use the previous step's (#3) command to store the address in a variable for re-use:
```sh
export node0=$(provenanced keys show -a node0 --home build/node0 --testnet)
``` 

- From the `provenance` directory, run the following command to store your smart contract. Note:  This uses the `artifacts` directory from Step #1 when the WASM was generated.
```sh
provenanced tx wasm store my/path/to/how-to-provenance/provenance-smart-contract-example/artifacts/provenance_smart_contract_example.wasm \
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
This command will automatically store the generated smart contract on your localnet.  Note: This uses the `--yes` flag, which will automatically run the transaction after verification that it is without errors.  To initiate a manual-confirmation mode, simply omit the `--yes` flag.  

The resulting output is piped to `jq`, which makes it much more presentable.  Within the payload, look for the `code_id` value.  The code id is used as a reference point when instantiating the smart contract.  If you have followed this guide from a clean setup, the `code_id` value should be `1` because no other smart contracts have been stored on your localnet.  To automatically find this value with `jq`, modify the end of the command to be: `jq -r '.logs[] | select(.msg_index == 0) | .events[] | select(.type == "store_code") | .attributes[0].value`.

To further verify that the wasm was fully stored, you can query the `wasm` module to check for the code that was stored:
```sh
provenanced q wasm list-code --testnet
```

And you should see something similar to the following:
```yaml
code_infos:
- code_id: "1"
  creator: <some bech32>
  data_hash: F66F1F7217D986BEC5670EF517B557CD594F146A63A1032090B3EACE8BFC0639
```

### Step #5: Instantiate, Execute, Query

- Instantiate the contract with the following command:
```sh
provenanced tx wasm instantiate 1 \
'{"contract_base_name": "examples.pio", "starting_counter": "10"}' \
--admin "$node0" \
--from node0 \
--home build/node0 \
--label examples \
--chain-id chain-local \
--gas auto \
--gas-prices="1905nhash" \
--gas-adjustment=1.2 \
--broadcast-mode block \
--testnet \
--output json \
--yes | jq
```
This command will automatically instantiate the contract with a base name of `examples.pio` and a starting counter value of `10`.  The starting counter value is completely optional, so feel free to remove that to start the counter at its default value of `0`.  This example uses the `pio` base name because it is not restricted on the localnet, which will allow the contract to create the `examples.pio` name for itself without issues.

The important piece of output in the result from this transaction is the `contract_address` value.  When a contract is instantiated on the Provenance blockchain, it is assigned a bech32 address, and this address can be used to run `execute` and `query` functions against the contract.  To automatically find this value with `jq`, modify the end of the previous comamnd to be: `jq '.logs[] | select(.msg_index == 0) | .events[] | select(.type == "instantiate") | .attributes[] | select(.key == "_contract_address") | .value'`.

The the remaining steps, assume that the contract address has been located by hand or `jq`, and exported to the following variable:
```sh
export contract_address=tp1suhgf5svhu4usrurvxzlgn54ksxmn8gljarjtxqnapv8kjnp4nrs60nkqw
```

- Increment the counter value in the state of the contract with an execute route:
```sh
provenanced tx wasm execute \
"$contract_address" \
'{"increment_counter": {"increment_amount": "5"}}' \
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

The initial value of the counter was `10`, and this command increments it by `5`, which, unless my calculator failed me, should result in a grand total of `15` in our internally-stored counter variable.  Let's find out for sure, though, with a query!

- Query the state value from the contract, which contains the counter value:
```sh
provenanced query wasm contract-state smart \
"$contract_address" \
'{"query_state": {}}' \
> --testnet \
> --output json | jq
```

The query should yield the following payload:
```json
{
  "data": {
    "contract_base_name": "examples.pio",
    "contract_counter": "15"
  }
}
```

That math looks good to me! 

Well done! You can now store, instantiate, and communicate with a smart contract on the Provenance blockchain!

## Build your own

Want to build your own smart contract from scratch? The [provwasm](https://github.com/provenance-io/provwasm) repository has an excellent [tutorial](https://github.com/provenance-io/provwasm/tree/main/docs/tutorial) for building a smart contract repository on the Provenance blockchain.