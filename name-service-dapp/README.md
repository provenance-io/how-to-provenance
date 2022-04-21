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

# Frontend Setup
The frontend utilizes the [Provenance Blockchain walletconnect-js library](https://github.com/provenance-io/walletconnect-js/) for communication with Provenance Wallet. The setup for this is quite simple and can be seen in [App.js](src/App.js) as this library provides a hook called `useWalletConnect`. This hook provides us with information about the current connection status (i.e. connected/disconnected, if connected, what is the address/basic details of the connected account), as well as a service that can be used for submitting transactions for signing to the Provenance Wallet, and registering event listeners for connect/disconnect events, as well as transaction success/failure.

The [Actions](src/Components/Actions) directory provides a couple of simple components for managing connecting to/disconnecting from your Provenance Wallet, that essentially just manage calls to the connect/disconnect methods on WalletConnectService provided by walletconnect-js, and managing state when the connection changes from connected -> disconnected, or the other way around.

The [RegisterName.tsx](src/Components/RegisterName/RegisterName) component provides us with a simple form that can accept a name to register and submit it using the following function passed in from [App.js](src/App.js)
```javascript
  async (name) => {
      return wcs.customAction({ // wcs is the WalletConnectService provided by walletconnect-js, this customAction method will submit the request for signing/broadcasting to our wallet, and the promise returned by this function will indicate the success/failure of the transaction
        message:
          await nameContractService.generateNameRegisterBase64Message( // this is a helper function from src/services/NameContractService.ts that generates the proper base64-encoded proto message to execute the smart contract's register entrypoint
            name,
            address
          ),
        description: `Register ${name} to ${address}`,
        method: "provenance_sendTransaction",
      });
  }
``` 

Any names already registered to your connected wallet's address will be listed above the form to register a new name. The functionality to fetch these names can be seen in the [NameContractService](src/services/NameContractService.ts) `listNames` method, which forms a query message and dispatches it to the smart contract's query endpoint via the [WasmService](src/services/WasmService.ts) (via the grpc-web proxy discussed in the note below).

The [NameLookup](src/Components/NameLookup/NameLookup.tsx) and [NameSearch](src/Components/NameSearch/NameSearch.tsx) components both provide similar functionality to the name list, but with different messages according to the type of query they are each performing.

The rest of the app is mostly just some supporting components to make the ui work, but this makes up the core functionality around building a Provenance Wallet connected dApp. The names registered to your account in this example can be seen attached to your account as attributes in the Provenance Blockchain Explorer, and other systems can now be built off of this functionality to, say, provide a smart contract that allows you to send funds to a human-readable name registered by this name service, as opposed to sending to a not-very-human-readable/memorable bech32-encoded address.

### Note
The frontend is able to submit transactions via the Provenance Wallet, and thereby does not need direct access to your wallet/private key. The queries performed by the frontend are done via a [grpc-web proxy](https://github.com/grpc/grpc-web#proxy-interoperability) which translates requests and relays them directly to a Provenance Blockchain node. This proxy is necessary due to the nature of current browser support limitations for grpc.

### Get Started:

1) Run `npm i`
2) Run `npm run start` to run a file watch + local server.
  - Note:
    - If it doesn't automatically, navigate to `http://localhost:3000/`

* At this point, any changes you make to files should kick off a rebuilt and be visible in the application, so poke around and modify if desired