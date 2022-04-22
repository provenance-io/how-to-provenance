# Name Smart Contact Example

This code is copied from https://github.com/piercetrey-figure/name-service-smart-contract and annotated for illustration purposes.
For the purposes of the containing dApp example, this won't need to be instantiated locally, as the Provenance Mobile Wallet
will need to be used with testnet.

## Getting Started
To build an un-optimized wasm, just run:
```shell
make
```

To build an optimized wasm for deployment in a provenance environemnt, run:
```shell
make optimize
```
This command will produce a file called `name_smart_contract.wasm` that can then be deployed to a provenance environment
by using the `provenanced` command.  

The command stems from: https://github.com/provenance-io/provenance
A great tutorial for getting a wasm built and deployed: https://github.com/provenance-io/provwasm/tree/main/docs/tutorial

