use cosmwasm_std::{Storage, Uint128};
use cosmwasm_storage::{singleton, singleton_read, ReadonlySingleton, Singleton};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// Each value saved into cosmwasm standard storage must have a byte array as its key.
// Cosmwasm Docs: https://docs.cosmwasm.com/dev-academy/develop-smart-contract/intro/
// Their tutorial also goes over using cosmwasm storage plus's Item struct, which is
// another simple way to store a persistent value in a smart contract.
static STATE_KEY: &[u8] = b"contract_state";

/// The State struct contains all persistent data associated with the contract.
/// A struct such as this should be used for maintaining values across various
/// transactions at a global level.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct State {
    /// The base name of the contract.  This value will be used to establish new sub-names
    /// in the AddAttribute execution route of the contract.
    pub contract_base_name: String,
    /// A Uint128 counter value.  This value can be incremented by the smart contract's
    /// IncrementCounter execution route, and will be used to demonstrate mutating the
    /// internal contract storage.
    pub contract_counter: Uint128,
}

/// This function loads the state in a mutable manner, taking a mutable reference to the
/// storage value provided by DepsMut.  This function should be used when saving a new State
/// instance, or mutating an existing one.
pub fn state(storage: &mut dyn Storage) -> Singleton<State> {
    singleton(storage, STATE_KEY)
}

/// This function loads the state in a read-only manner, taking an immutable reference to the
/// storage value provided by Deps.  Note that DepsMut's storage can also be used with this function.
pub fn state_read(storage: &dyn Storage) -> ReadonlySingleton<State> {
    singleton_read(storage, STATE_KEY)
}
