use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// The InitMsg is used once in the smart contract lifecycle. When the instantiate execution
/// route is invoked, this message is expected as input.  Clearly defining all requirements
/// for the initial state of the smart contract is key to a well-made and useful contract.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct InitMsg {
    /// This value will be used to bind a name to the smart contract, using Provenance's
    /// name module.  This is assisted by the provwasm library.
    pub contract_base_name: String,
    /// This value will be the initial counter value, which will be used to display some
    /// functionality in simple routes. Note that it is wrapped in an Option, which makes
    /// it an optional input parameter during instantiation.  If left blank, the initial
    /// value will be zero.
    pub starting_counter: Option<u128>,
}

/// The ExecuteMsg will generally be an enum to allow for multiple different types of contract
/// execution, but for simple contracts, it can certainly simply be a struct.  The execute
/// message defines a route in the contract that can execute transactions on the Provenance
/// blockchain. These endpoints should be for various CRUD operations, and/or mutating the
/// contract state for certain scenarios.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// This execution route increments the internal counter created during instantiation and
    /// stored in the contract state.  Note that the optional value used here is an unsigned
    /// type, ensuring that the account invoking the contract on this endpoint will be unable
    /// to enter a negative value by default.  Deserialization of the input JSON will fail if
    /// negatives are included.  This could be changed simply by changing the type to signed,
    /// if one were so inclined to do so.
    IncrementCounter {
        /// The amount to add to the internal contract counter.  If left blank, a default value
        /// of one will be used.
        increment_amount: Option<u128>,
    },
    /// This execution route will append an attribute to the contract itself, using its reserved
    /// contract_base_name value.  For example, if the base name of the contract was "testcontract.pb"
    /// and the attribute_name value used in this route was "new", the newly-created attribute would be
    /// created with the name "new.testcontract.pb."
    AddAttribute {
        /// The sub-name of contract_base_name to be used when creating the attribute. This value will
        /// be used to prefix the contract_base_name, stored in contract state, and will be combined with
        /// it to reserve a new name and attribute combination.
        attribute_prefix: String,
        /// The text to use as the attribute body.  This value will be stored in the new attribute created
        /// at name "{attribute_prefix}.{contract_base_name}" and will be plain text.
        attribute_text: String,
    },
    /// This execution route will send the funds passed in to the contract to the target address.
    /// This example will illustrate using cosmwasm's provided functionality for Coin management.
    SendFunds {
        /// The bech32 address of the recipient on the Provenance network.  All funds provided to
        /// the contract will be sent to this address.
        recipient_address: String,
    },
}

/// The QueryMsg will generally be an enum to allow for multiple different types of queries.
/// The entry_point for queries allows a read-only Deps<ProvenanceQuery>, so mutation of values
/// is not expected for these routes.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// This query will attempt to find an attribute with a name that starts with the given
    /// prefix, and ends with the contract_base_name.  This mirrors the AddAttribute execution
    /// route, and is designed to lookup existing attributes created by it. The AddAttribute
    /// route creates String attributes, so the expected response value upon finding an attribute
    /// will be the String value contained within the attribute.
    QueryAttribute {
        /// The prefix to an existing attribute, always ending in contract_base_name.
        attribute_prefix: String,
    },
    /// This query will return the current values of the contract's State value, which is held
    /// in internal storage.  The value of the counter is stored in State, so this route can also
    /// be used to fetch the current counter value.
    QueryState {},
}
