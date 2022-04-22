export const NETWORK = process.env.REACT_APP_NETWORK
export const PRODUCTION = NETWORK == 'mainnet'
export const EXPLORER_URL = PRODUCTION ? 'https://explorer.provenance.io' : 'https://explorer.test.provenance.io'
export const GRPC_URL = PRODUCTION ? 'https://wallet.provenance.io/proxy' : 'https://wallet.test.provenance.io/proxy'
// export const GRPC_URL = 'http://localhost:8080'
export const WALLET_URL = PRODUCTION ? 'https://wallet.provenance.io' : 'https://wallet.test.provenance.io'
export const ROOT_NAME = 'wallettest3.pb'
export const FEE_DENOM = 'nhash'
