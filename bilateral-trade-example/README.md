## Bilateral Exchange Smart Contract

This a CosmWasm smart contract that provides the bilateral exchange of `provenance` `markers`.

## Build

```bash
make
```

## Quickstart

```bash
git clone git@github.com:provenance-io/provenance.git
git clone git@github.com:provenance-io/bilateral-exchange.git

cp bilateral-exchange/examples/bilateral.sh bilateral-exchange/examples/create-base.sh provenance
cd bilateral-exchange
make
cp artifacts/bilateral_exchange.wasm ../provenance
cd ../provenance
./create-base.sh
./bilateral.sh
```

## Example Usage

_NOTE: Address bech32 values and other generated params may vary._

0. Configure the following:
    1. Accounts:
        - Asker
        - Buyer
    2. Markers:
        - Base
        - Quote

1. Store the `bilateral-exchange` WASM:
    ```bash
    build/provenanced tx wasm store bilateral_exchange.wasm \
    --from validator \
    --keyring-backend test \
    --home build/run/provenanced \
    --chain-id testing \
    --gas auto --gas-prices 1905nhash --gas-adjustment 2 \
    --broadcast-mode block \
    --yes \
    --testnet
    ```
   
2. Instantiate the contract, binding the name `bilateral-ex.sc.pb` to the contract address:
    ```bash
    build/provenanced tx wasm instantiate 1 '{"bind_name":"bilateral-ex.sc","contract_name":"bilateral-ex"}' \
    --admin "$(build/provenanced keys show -ta validator --home build/run/provenanced --keyring-backend test)" \
    --from validator \
    --keyring-backend test \
    --home build/run/provenanced \
    --chain-id testing \
    --label ats-gme-usd \
    --gas auto --gas-prices 1905nhash --gas-adjustment 2 \
    --broadcast-mode block \
    --yes \
    --testnet
    ```

3. Create an `ask` order:

    _NOTE: Replace `M2` with the `ask` base marker. Replace `M1_AMT` and `M1_DENOM` with quote marker_
   
    _NOTE++: The json data '{"create_ask":{}}' represents the action and additional data to pass into the smart contract. The actual coin with the transaction is the `--amount` option._
    
    ```bash
    build/provenanced tx wasm execute "$(provenanced q name resolve bilateral-ex.sc --testnet | awk '{print $2}')" \
    '{"create_ask":{"id":"ask_id", "quote":[{"amount":"M1_AMT", "denom":"M1_DENOM"}]}}' \
    --amount M2 \
    --from (build/provenanced keys show -ta seller --home build/run/provenanced --keyring-backend test) \
    --keyring-backend test \
    --home build/run/provenanced \
    --chain-id testing \
    --gas auto --gas-prices 1905nhash --gas-adjustment 2 \
    --broadcast-mode block \
    --yes \
    --testnet
    ```

4. Create a `bid` order:

    _NOTE: Replace `M1` with the `bid` quote marker. Replace `M2_AMT` and `M2_DENOM` with base marker_
    
    _NOTE++: The json data '{"create_bid":{}}' represents the action and additional data to pass into the smart contract. The actual coin with the transaction is the `--amount` option._
    ```bash
    build/provenanced tx wasm execute "$(provenanced q name resolve bilateral-ex.sc --testnet | awk '{print $2}')" \
    '{"create_bid":{"id":"bid_id", "base":[{"amount":"M2_AMT", "denom":"M2_DENOM"}]}}' \
    --amount M1 \
    --from (build/provenanced keys show -ta buyer --home build/run/provenanced --keyring-backend test) \
    --keyring-backend test \
    --home build/run/provenanced \
    --chain-id testing \
    --gas auto --gas-prices 1905nhash --gas-adjustment 2 \
    --broadcast-mode block \
    --yes \
    --testnet
    ```

5. Match and execute the ask and bid orders.
   ```bash
    build/provenanced tx wasm execute "$(provenanced q name resolve bilateral-ex.sc --testnet | awk '{print $2}')" \
    '{"execute_match":{"ask_id":"ask_id", "bid_id":"bid_id"}}' \
    --from validator \
    --keyring-backend test \
    --home build/run/provenanced \
    --chain-id testing \
    --gas auto --gas-prices 1905nhash --gas-adjustment 2 \
    --broadcast-mode block \
    --yes \
    --testnet
    ```

## Other actions

Cancel the contract.

```bash
build/provenanced tx wasm execute "$(provenanced q name resolve bilateral-ex.sc --testnet | awk '{print $2}')" \
'{"cancel_ask":{"id":"ask_id"}}' \
--from (build/provenanced keys show -ta seller --home build/run/provenanced --keyring-backend test) \
--keyring-backend test \
--home build/run/provenanced \
--chain-id testing \
--gas auto --gas-prices 1905nhash --gas-adjustment 2 \
--broadcast-mode block \
--yes \
--testnet
```

Query for ask order information:
```bash
provenanced query wasm contract-state smart "$(provenanced q name resolve bilateral-ex.sc --testnet | awk '{print $2}')" \
'{"get_ask":{"id":"ask_id"}}' \
--testnet
```

Query for bid order information:
```bash
provenanced query wasm contract-state smart "$(provenanced q name resolve bilateral-ex.sc --testnet | awk '{print $2}')" \
'{"get_bid":{"id":"bid_id"}}' \
--testnet
```

Query for contract instance information
```bash
provenanced query wasm contract-state smart "$(provenanced q name resolve bilateral-ex.sc --testnet | awk '{print $2}')" \
'{"get_contract_info":{}}' \
--testnet
```