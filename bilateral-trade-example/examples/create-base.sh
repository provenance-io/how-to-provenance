#!/usr/bin/env bash

make clean build

build/provenanced -t --home build/run/provenanced init --chain-id=testing testing
build/provenanced -t --home build/run/provenanced keys add validator --keyring-backend test
build/provenanced -t --home build/run/provenanced keys add buyer --keyring-backend test
build/provenanced -t --home build/run/provenanced keys add seller --keyring-backend test
build/provenanced -t --home build/run/provenanced add-genesis-root-name validator pio --keyring-backend test
build/provenanced -t --home build/run/provenanced add-genesis-root-name validator pb --restrict=false --keyring-backend test
build/provenanced -t --home build/run/provenanced add-genesis-root-name validator io --restrict --keyring-backend test
build/provenanced -t --home build/run/provenanced add-genesis-root-name validator provenance --keyring-backend test
build/provenanced -t --home build/run/provenanced add-genesis-root-name validator sc --restrict=false --keyring-backend test
build/provenanced -t --home build/run/provenanced add-genesis-account validator 100000000000000000000nhash --keyring-backend test
build/provenanced -t --home build/run/provenanced add-genesis-account buyer 100usd,100000000000000nhash --keyring-backend test
build/provenanced -t --home build/run/provenanced add-genesis-account seller 100gme,100000000000000nhash --keyring-backend test
build/provenanced -t --home build/run/provenanced add-genesis-marker 200000000000000000000nhash --manager validator --access mint,burn,admin,withdraw,deposit --activate --keyring-backend test
build/provenanced -t --home build/run/provenanced add-genesis-marker 1000gme --manager validator --access mint,burn,admin,withdraw,deposit --activate --keyring-backend test
build/provenanced -t --home build/run/provenanced add-genesis-marker 1000usd --manager validator --access mint,burn,admin,withdraw,deposit --activate --keyring-backend test
build/provenanced -t --home build/run/provenanced gentx validator 1000000000000000nhash --keyring-backend test --chain-id=testing
build/provenanced -t --home build/run/provenanced collect-gentxs

make run
