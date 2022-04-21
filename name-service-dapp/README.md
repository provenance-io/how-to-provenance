# Name Service dApp Edxample

This directory contains an example dApp (decentralized application) to demonstrate the ability to create an application
consisting solely of a frontend and a smart contract running on Provenance Blockchain. For a more complicated application,
a backend application can be in the mix, but that is beyond the scope of this particular example.

The [contract](./contract/) directory contains the smart contract that this frontend utilizes, however the example will
connect to the Provenance Blockchain testnet and utilize a smart contract instance already initialized there, as transaction
signing will be facilitated by the Provenance Wallet, and setting that up to communicate with a local node is outside
the scope of this example.

## Features
This dApp provides functionality to register names (intended to be short, human-readable text) with accounts on Provenance Blockchain.
An account can have multiple names registered to it, and names can be resolved to the associated account via the smart contract, and a list of names
bound to an account can also be resolved by the account address. Additionally, there is a simple 'fuzzy' search, that allows looking up names by a partial
match.

The frontend (React UI) for this example provides a simple web interface exposing the smart contract functionality for:
* Registering names with your account (via transactions you will sign using the Provenance Wallet)
* Resolving the address bound to a name
* Listing all names bound to an address
* Searching for names via a partial match

### Note
The frontend is able to submit transactions via the Provenance Wallet, and thereby does not need direct access to your wallet/private key. The queries performed by the frontend are done via a [grpc-web proxy](https://github.com/grpc/grpc-web#proxy-interoperability) which translates requests and relays them directly to a Provenance Blockchain node. This proxy is necessary due to the nature of current browser support limitations for grpc.

### Get Started:

1) Run `npm i`
2) Run `npm run start` to run a file watch + local server.
  - Note:
    - If it doesn't automatically, navigate to `http://localhost:3000/`

* At this point, any changes you make to files should kick off a rebuilt and be visible in the application, so poke around and modify if desired