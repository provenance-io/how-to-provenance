#!/usr/bin/env bash

printf "\n...store wasm...\n"
build/provenanced tx wasm store bilateral_exchange.wasm \
  --from validator \
  --keyring-backend test \
  --home build/run/provenanced \
  --chain-id testing \
  --gas auto --gas-prices 1905nhash --gas-adjustment 2 \
  --broadcast-mode block \
  --yes \
  --testnet

printf "\n...instantiate contract...\n"
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

printf "\n...buyer balance...\n";
build/provenanced query bank balances "$(build/provenanced keys show buyer -at --home build/run/provenanced --keyring-backend test)" \
  --home build/run/provenanced \
  --testnet

printf "\n...seller balance...\n"
build/provenanced query bank balances "$(build/provenanced keys show seller -at --home build/run/provenanced --keyring-backend test)" \
  --home build/run/provenanced \
  --testnet

printf "\n...contract balance...\n"
build/provenanced query bank balances "$(provenanced q name resolve bilateral-ex.sc --testnet | awk '{print $2}')" \
  --home build/run/provenanced \
  --testnet

printf "\n...seller creating ask...\n"
build/provenanced tx wasm execute "$(provenanced q name resolve bilateral-ex.sc --testnet | awk '{print $2}')" \
  '{"create_ask":{"id":"ask_id", "quote":[{"amount":"8", "denom":"usd"}]}}' \
  --amount 1gme \
  --from "$(build/provenanced keys show -ta seller --home build/run/provenanced --keyring-backend test)" \
  --keyring-backend test \
  --home build/run/provenanced \
  --chain-id testing \
  --gas auto --gas-prices 1905nhash --gas-adjustment 2 \
  --broadcast-mode block \
  --yes \
  --testnet

printf "\n...buyer creating bid...\n";
build/provenanced tx wasm execute "$(provenanced q name resolve bilateral-ex.sc --testnet | awk '{print $2}')" \
  '{"create_bid":{"id":"bid_id", "base":[{"amount":"1", "denom":"gme"}]}}' \
  --amount 8usd \
  --from "$(build/provenanced keys show -ta buyer --home build/run/provenanced --keyring-backend test)" \
  --keyring-backend test \
  --home build/run/provenanced \
  --chain-id testing \
  --gas auto --gas-prices 1905nhash --gas-adjustment 2 \
  --broadcast-mode block \
  --yes \
  --testnet

printf "\n...buyer balance...\n";
build/provenanced query bank balances "$(build/provenanced keys show buyer -at --home build/run/provenanced --keyring-backend test)" \
  --home build/run/provenanced \
  --testnet


printf "\n...seller balance...\n";
build/provenanced query bank balances "$(build/provenanced keys show seller -at --home build/run/provenanced --keyring-backend test)" \
  --home build/run/provenanced \
  --testnet

printf "\n...contract balance...\n";
build/provenanced query bank balances "$(provenanced q name resolve bilateral-ex.sc --testnet | awk '{print $2}')" \
  --home build/run/provenanced \
  --testnet

printf "\n...ask order info...\n";
build/provenanced query wasm contract-state smart "$(provenanced q name resolve bilateral-ex.sc --testnet | awk '{print $2}')" \
  '{"get_ask":{"id":"ask_id"}}' \
  --ascii \
  --testnet

printf "\n...bid order info...\n";
build/provenanced query wasm contract-state smart "$(provenanced q name resolve bilateral-ex.sc --testnet | awk '{print $2}')" \
  '{"get_bid":{"id":"bid_id"}}' \
  --ascii \
  --testnet

printf "\n...contract info...\n";
build/provenanced query wasm contract-state smart \
  "$(provenanced q name resolve bilateral-ex.sc --testnet | awk '{print $2}')" \
  '{"get_contract_info":{}}' \
  --testnet

printf "\n...executing match...\n";
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

printf "\n...buyer balance...\n";
build/provenanced query bank balances "$(build/provenanced keys show buyer -at --home build/run/provenanced --keyring-backend test)" \
  --home build/run/provenanced \
  --testnet

printf "\n...seller balance...\n";
build/provenanced query bank balances "$(build/provenanced keys show seller -at --home build/run/provenanced --keyring-backend test)" \
  --home build/run/provenanced \
  --testnet

printf "\n...contract balance...\n";
build/provenanced query bank balances "$(provenanced q name resolve bilateral-ex.sc --testnet | awk '{print $2}')" \
  --home build/run/provenanced \
  --testnet

printf "\n...seller creating ask...\n";
build/provenanced tx wasm execute "$(provenanced q name resolve bilateral-ex.sc --testnet | awk '{print $2}')" \
  '{"create_ask":{"id":"ask_id", "quote":[{"amount":"8", "denom":"usd"}]}}' \
  --amount 1gme \
  --from "$(build/provenanced keys show -ta seller --home build/run/provenanced --keyring-backend test)" \
  --keyring-backend test \
  --home build/run/provenanced \
  --chain-id testing \
  --gas auto --gas-prices 1905nhash --gas-adjustment 2 \
  --broadcast-mode block \
  --yes \
  --testnet

printf "\n...seller canceling ask...\n";
build/provenanced tx wasm execute "$(provenanced q name resolve bilateral-ex.sc --testnet | awk '{print $2}')" \
  '{"cancel_ask":{"id":"ask_id"}}' \
  --from "$(build/provenanced keys show -ta seller --home build/run/provenanced --keyring-backend test)" \
  --keyring-backend test \
  --home build/run/provenanced \
  --chain-id testing \
  --gas auto --gas-prices 1905nhash --gas-adjustment 2 \
  --broadcast-mode block \
  --yes \
  --testnet
