use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::core::state::{NameMeta, State};

/// A message sent to initialize the contract state.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub name: String,
    pub fee_amount: String,
    pub fee_collection_address: String,
}

/// A message sent to register a name with the name service
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Register { name: String },
}

/// A message sent to query contract config state.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    QueryRequest {},
    QueryAddressByName { name: String },
    QueryNamesByAddress { address: String },
    SearchForNames { search: String },
    Version {},
}

/// A type alias for contract state.
pub type QueryResponse = State;

/// Migrate the contract
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct MigrateMsg {
    pub new_fee_amount: Option<String>,
    pub new_fee_collection_address: Option<String>,
}
impl MigrateMsg {
    pub fn has_fee_changes(&self) -> bool {
        self.new_fee_amount.is_some() || self.new_fee_collection_address.is_some()
    }

    pub fn empty() -> MigrateMsg {
        MigrateMsg {
            new_fee_amount: None,
            new_fee_collection_address: None,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct NameResponse {
    pub address: String,
    pub names: Vec<String>,
}
impl NameResponse {
    pub fn new(address: String, names: Vec<String>) -> NameResponse {
        NameResponse { address, names }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct NameSearchResponse {
    pub search: String,
    pub names: Vec<NameMeta>,
}
impl NameSearchResponse {
    pub fn new(search: String, names: Vec<NameMeta>) -> NameSearchResponse {
        NameSearchResponse { search, names }
    }
}
