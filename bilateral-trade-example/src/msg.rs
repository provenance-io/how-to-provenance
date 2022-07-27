use cosmwasm_std::{Coin, Timestamp, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::state::BaseType;

/// Constructs a new instance of the smart contract.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    /// A name that will be bound to the smart contract using the Provenance Blockchain Name Module.
    /// This name must be unrestricted, or a failure will occur.  Note: The Provenance Blockchain
    /// provides a parent name, "sc.pb" on both testnet and mainnet that is unrestricted, specifically
    /// for binding smart contracts on instantiation.
    pub bind_name: String,
    /// A free-form name for the smart contract, purely for description and display purposes.
    pub contract_name: String,
    /// An amount to be charged to the sender when an ask is created.  This uses the Provenance
    /// Blockchain Fee Module, which will take 50% of the fees sent and redistribute them to various
    /// external entities.  The other 50% will be retained and sent to the contract's admin account.
    pub ask_fee: Option<Uint128>,
    /// An amount to be charged to the sender when a bid is created.  This uses the Provenance
    /// Blockchain Fee Module, which will take 50% of the fees sent and redistribute them to various
    /// external entities.  The other 50% will be retained and sent to the contract's admin account.
    pub bid_fee: Option<Uint128>,
}

/// Executes the smart contract, causing changes reflected in Provenance Blockchain transactions.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Removes an ask from the contract's storage and refunds the base (Provenance Blockchain
    /// Metadata Scope or Coin).  Ask creation fees are not refunded.
    CancelAsk {
        /// The unique identifier for the ask to cancel.  If no ask with this value exists in
        /// contract storage, an error will be returned.
        id: String,
    },
    /// Removes a bid from the contract's storage and refunds the quote funds provided.  Bid creation
    /// fees are not refunded.
    CancelBid {
        /// The unique identifier for the bid to cancel.  If no bid with this value exists in
        /// contract storage, an error will be returned.
        id: String,
    },
    /// Creates a new AskOrder, holding the given base Coin or Provenance Blockchain Metadata Scope
    /// in the smart contract until a cancellation occurs or a match is made.
    CreateAsk {
        /// The unique identifier for the new ask to create.  If an ask already exists with the
        /// given id, an error will be returned.
        id: String,
        /// The funds that a bidder must provide for a match to be successfully executed.
        quote: Vec<Coin>,
        /// The address of a scope to list for trade.  If this value is omitted, funds must be
        /// provided in the execute message transaction.
        scope_address: Option<String>,
    },
    /// Creates a new BidOrder, holding the given quote Coin in the smart contract until a
    /// cancellation occurs or a match is made.
    CreateBid {
        /// The unique identifier for the new bid to create.  If a bid already exists with the
        /// given id, an error will be returned.
        id: String,
        /// Indicates the type of exchange that will be made: scope or coin.
        base: BaseType,
        /// An optional timestamp denoting when the bid was created.
        effective_time: Option<Timestamp>,
    },
    /// Attempts to match an AskOrder with a BidOrder, performing an exchange of the asker's base
    /// with the bidder's quote.  This will only be successful if the bidder's base matches the
    /// asker's base, and the asker's quote matches the bidder's quote.
    ExecuteMatch {
        /// The unique identifier of the ask to attempt a match on.  If no ask exists within the
        /// contract storage with this id, an error will be returned.
        ask_id: String,
        /// The unique identifier of the bid to attempt a match on.  If no bid exists within the
        /// contract storage with this id, an error will be returned.
        bid_id: String,
    },
}

/// Fetches data from the smart contract.  No query routes make changes to blockchain data.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// Fetches an existing AskOrder from contract storage.
    GetAsk {
        /// The unique identifier of the AskOrder to fetch.  If no order exists in storage for the
        /// given id, an error will be returned.
        id: String,
    },
    /// Fetches an existing BidOrder from contract storage.
    GetBid {
        ///  The unique identifier of the BidOrder to fetch.  If no order exists in storage for the
        /// given id, an error will be returned.
        id: String,
    },
    /// Fetches the ContractInfo from contract storage.  This value is created as part of the
    /// instantiation process, so this query should only ever fail if the blockchain is experiencing
    /// downtime.
    GetContractInfo {},
}

/// Migrates the smart contract to a new version of its source code.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MigrateMsg {
    /// Overwrites the contract's base code with a new version.  This route will never modify values
    /// stored in contract storage (like Ask or Bid orders).
    NewVersion {},
}
