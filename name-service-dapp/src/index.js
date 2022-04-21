import ReactDOM from "react-dom";
import { WalletConnectContextProvider } from "@provenanceio/walletconnect-js";
import { WalletContextProvider } from "@provenanceio/wallet-lib";
import { StrictMode } from "react";
import { App } from "./App";
// Bring in Google Fonts and base styles
import "./base.css";
import { GRPC_URL, NETWORK, WALLET_URL } from "./consts/network";

ReactDOM.render(
  <StrictMode>
    <WalletConnectContextProvider network={NETWORK}>
      <WalletContextProvider
        grpcServiceAddress={GRPC_URL}
        walletUrl={WALLET_URL}
      >
        <App />
      </WalletContextProvider>
    </WalletConnectContextProvider>
  </StrictMode>,
  document.getElementById("root")
);
