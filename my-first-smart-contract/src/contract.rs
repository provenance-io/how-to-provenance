use cosmwasm_std::{
    entry_point, to_binary, BankMsg, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo,
    Response, Uint128,
};
use provwasm_std::{
    add_attribute, bind_name, AttributeValueType, NameBinding, ProvenanceMsg, ProvenanceQuerier,
    ProvenanceQuery,
};

use crate::{
    error::ContractError,
    msg::{ExecuteMsg, InitMsg, QueryMsg},
    state::{state, state_read, State},
};

/// The instantiation entry_point is the first function that is ever executed in a smart contract, and
/// it is only ever executed once.  This function is required to have each argument it specifies, and is
/// configured based on the final argument, msg.  
#[entry_point]
pub fn instantiate(
    deps: DepsMut<ProvenanceQuery>,
    env: Env,
    info: MessageInfo,
    msg: InitMsg,
) -> Result<Response<ProvenanceMsg>, ContractError> {
    // Verify that no funds were sent into the contract by the address that is instantiating it.  Those funds
    // would be held permanently by the contract's address, so any request attempting to do so should be rejected.
    check_funds_are_empty(info.funds)?;
    // Create an instance of the contract's State, which holds the contract's base name and a counter for later.
    // The base name will be used to create attributes later, so it's very important that that value is recorded
    // in a place that can be located later.
    let contract_state = State {
        contract_base_name: msg.contract_base_name,
        // Important: Although the input value to the contract can be a u128, actually storing values in
        // cosmwasm's storage requires them to be wrapped for serialization.  These wrappers can be hand-built
        // to work with serde, but cosmwasm has been kind enough to develop many different wrappers for these
        // purposes.  In this case, the wrapper used is Uint128, and has an implementation for Into<u128>, which
        // automatically allows this u128 value to be converted with a simple .into() call.
        contract_counter: msg.starting_counter.unwrap_or(0).into(),
    };
    // Store the initial state in the contract's internal storage, which can be referenced during execution
    // and query routes later.
    state(deps.storage).save(&contract_state)?;
    // Create a ProvenanceMsg that will bind the specified name to the contract.
    // Note: If binding the name fails for any reason, all other changes will also be reverted.
    // This ensures that the contract will safely be created with a fully-stored state and properly-formed
    // and accessible base name.
    let bind_name_msg = bind_name(
        // name: The name to bind to the contract's address.
        // Note: The contract base name is passed by reference here to ensure it can also be passed into the Response
        &contract_state.contract_base_name,
        // address: The contract's address, wrapped in cosmwasm's Addr struct
        env.contract.address,
        // binding: The type of binding (restricted or unrestricted). Restricted is chosen, here, to ensure
        // that only the contract itself has access to create and modify attributes bound to the name or its
        // various sub-names.
        NameBinding::Restricted,
    )?;
    // After successful instantiation, a response must be returned containing the various messages and attributes
    // that will be included in the transaction that this instantiation creates.  Upon successful instantiation,
    // all messages included in the response will be executed and their actions will be completed.  In this case,
    // a new name will be bound to the contract's address.
    Ok(Response::new()
        // Simply adding the created "bind name" message to the Response will ensure that the name is bound as part of
        // the resulting transaction.
        .add_message(bind_name_msg)
        // Any number of free-form attributes can also be included in the transaction, which allows the transaction
        // within the block it is written to to broadcast various characteristics about itself.  This allows
        // downstream consumers of any events emitted to have a clearer indication of the actions taken during
        // the transaction.
        .add_attribute("action", "instantiate")
        .add_attribute("contract_base_name", &contract_state.contract_base_name))
}

/// The execute entry_point is used for contract actions that mutate its internal state and/or execute transactions
/// in the Provenance blockchain using provwasm or cosmwasm's helper functions to generate messages.  As a smart
/// contract grows more complex, it's important to keep things readable.  This example uses a match, funneling the
/// values of each enum into private functions within this same file.  However, in much larger contracts, it might
/// be appropriate to create sub modules to further abstract the codebase and keep things tidy.
#[entry_point]
pub fn execute(
    deps: DepsMut<ProvenanceQuery>,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response<ProvenanceMsg>, ContractError> {
    match msg {
        ExecuteMsg::IncrementCounter { increment_amount } => {
            increment_counter(deps, info, increment_amount)
        }
        ExecuteMsg::AddAttribute {
            attribute_prefix,
            attribute_text,
        } => add_attribute_to_contract(deps, info, env, attribute_prefix, attribute_text),
        ExecuteMsg::SendFunds { recipient_address } => send_funds(deps, info, recipient_address),
    }
}

/// The query entry_point is used for contract actions that operate on a read-only basis.  This is further
/// evidenced by the requirement for this function to only take a Deps instead of a DepsMut.  Additionally,
/// the response value is a Result<Binary, X> instead of a Result<Response<X>, X>.  This is because the
/// resulting value should be serialized Binary that can be sent to the caller for deserialization. Cosmwasm
/// requires these standards.  Attempting to run "make optimize" after modifying the query route to include
/// a different response type or mutable Deps will cause the build to fail.
#[entry_point]
pub fn query(
    deps: Deps<ProvenanceQuery>,
    env: Env,
    msg: QueryMsg,
) -> Result<Binary, ContractError> {
    // Fetch the contract state value from storage.  Due to the contract having a persistent state internal
    // storage available, and because the query entry_point cannot be executed until instantiation has taken
    // place, this value can be guaranteed to be present.
    let contract_state = state_read(deps.storage).load()?;
    match msg {
        QueryMsg::QueryAttribute { attribute_prefix } => {
            // Construct the expected attribute name from the prefix and the contract base name.  This mirrors
            // the formatting used in the execute route: AddAttribute.
            let target_attribute_name =
                format_attribute_name(&attribute_prefix, &contract_state.contract_base_name);
            // Provwasm provides a wrapper for the cosmwasm's QuerierWrapper, which is contained in deps.querier.
            // This allows for Provenance modules to be queried simply and easily.
            let provenance_querier = ProvenanceQuerier::new(&deps.querier);
            // This check is to ensure that the attribute being searched for exists.  The AddAttribute route
            // ensures that only a single attribute for a single name can be added, so this check verifies that
            // that state exists.
            let attribute_wrapper = provenance_querier
                .get_attributes(env.contract.address, Some(target_attribute_name))?;
            if attribute_wrapper.attributes.len() != 1 {
                return Err(ContractError::generic_err(format!(
                    "Expected only one attribute to exist at the specified name, but found {}",
                    attribute_wrapper.attributes.len()
                )));
            }
            // Note that this response does not use to_binary.  This is because the ProvenanceQuerier will
            // respond with the attribute value already wrapped in cosmwasm's Binary struct, so that step
            // can be skipped entirely.
            Ok(attribute_wrapper
                .attributes
                .first()
                .unwrap()
                .value
                .to_owned())
        }
        // The state has been pre-fetched before all query routes.  It derives Serialize and Deserialize, so
        // it is safe to use to_binary on it to use the entire value as a response and serialize it to a Binary
        // struct.
        QueryMsg::QueryState {} => Ok(to_binary(&contract_state)?),
    }
}

/// A function for standardizing the format for sub-names of the base contract name.
/// Ensures that all contract functionality that interacts with created base attributes
/// will produce the same names, given the same input.
///
// All names in the Provenance name module are separated by ".", which indicates sub-names.
// Example: If the contract's base name is test.pb, then the contract's name is a sub-name of the
// name "pb".  If the prefix "my" is provided to this route, the resulting name would be "my.test.pb"
// which will be a sub-name of the contract's base name of "test.pb"
fn format_attribute_name(prefix: &str, base_name: &str) -> String {
    format!("{prefix}.{base_name}")
}

/// The flow of a contract is controlled by its return values to its various entry_point functions.
/// If any scenario arises during contract execution that is undesirable or would cause a bad state,
/// returning an error like this is a way to ensure that all changes are completely disregarded.
///
/// If funds are ever included in the MessageInfo.funds, the smart contract itself is transferred those funds.
/// This check prevents an instantiation message from seeding the contract with funds that it does not need.
fn check_funds_are_empty(funds: Vec<Coin>) -> Result<(), ContractError> {
    if !funds.is_empty() {
        Err(ContractError::InvalidFunds {
            explanation: format!("Funds should not be included in the transaction when instantiating the contract. Found funds: {:?}", funds), 
        })
    } else {
        Ok(())
    }
}

fn increment_counter(
    deps: DepsMut<ProvenanceQuery>,
    info: MessageInfo,
    increment_amount: Option<u128>,
) -> Result<Response<ProvenanceMsg>, ContractError> {
    // Leverage the funds check to ensure that this free execution route does not receive funds at all
    check_funds_are_empty(info.funds)?;
    // If the increment amount provided in the message was present, use it.
    // Otherwise, default to the standard increment amount of 1. This allows the user to
    // completely omit the value from the request payload and still get an increment.
    let amount_to_increment: Uint128 = increment_amount.unwrap_or(1).into();
    let mut state_storage = state(deps.storage);
    // Load the contract state in a mutable manner, allowing the internals to be modified in this execution route
    let mut contract_state = state_storage.load()?;
    contract_state.contract_counter += amount_to_increment;
    // After incrementing the counter, it must be saved to the contract's internal state. This will persist
    // the value, and subsequent increments will see the new value. This will also be available and evident in
    // the query routes.
    state_storage.save(&contract_state)?;
    Ok(Response::new()
        .add_attribute("action", "execute")
        .add_attribute(
            "new_counter_value",
            contract_state.contract_counter.to_string(),
        ))
}

fn add_attribute_to_contract(
    deps: DepsMut<ProvenanceQuery>,
    info: MessageInfo,
    env: Env,
    attribute_name: String,
    attribute_text: String,
) -> Result<Response<ProvenanceMsg>, ContractError> {
    // Leverage the funds check to ensure that this free execution route does not receive funds at all
    check_funds_are_empty(info.funds)?;
    let contract_state = state(deps.storage).load()?;
    let new_attribute_name =
        format_attribute_name(&attribute_name, &contract_state.contract_base_name);
    let provenance_querier = ProvenanceQuerier::new(&deps.querier);
    // Check to ensure that the new name does not exist.  If the ProvenanceQuerier does not return an error when
    // searching for the name, that means that the name was correctly resolved, and is already set on the contract.
    // This check will prevent execution calls from adding duplicate attributes and/or names, which is allowed in the name module of the
    // Provenance blockchain but not the desired functionality for this contract.  For the purposes of this contract, only one attribute
    // should exist per sub-name.
    if let Ok(name_result) = provenance_querier.resolve_name(&new_attribute_name) {
        return Err(ContractError::NameAlreadyExists {
            name: name_result.name,
            owner_address: name_result.address.to_string(),
        });
    }
    // Like in the instantiation function, construct another bind name message using the constructed attribute name.
    let bind_name_msg = bind_name(
        &new_attribute_name,
        // Set the owner of the name as the contract, ensuring that only the contract's address can add
        // attributes to this name
        env.contract.address.clone(),
        // Restrict this name to ensure that only one attribute can be added to it, and only by the contract
        NameBinding::Restricted,
    )?;
    // Finally, after creating the newly-desired name, craft an add_attribute message that will store an attribute
    // at the newly-created name.  Attributes can only be assigned to existing names, so this must occur after the
    // name is bound to the contract.
    let add_attribute_msg = add_attribute(
        // Bind the attribute to the contract itself.  In a normal use-case, the contract itself would not get
        // the attribute, because granting an attribute to the contract essentially has no value.  For the purposes
        // of this example, though, using the contract as the attribute owner simplifies things.
        env.contract.address,
        // Use the previously-bound name as the attribute's name
        &new_attribute_name,
        // Serialize the provided text as Binary.  Cosmwasm provides a set of to_binary and from_binary functions
        // that allow any serializable value to easily be converted.  Serializing custom structs is easy, as well!
        // Simply #derive(Serialize, Deserialize) using serde and these binary helper functions will automatically
        // know how to convert them into cosmwasm's Binary struct.
        to_binary(&attribute_text)?,
        // Provenance requires that each attribute be tagged with its type.  Custom structs would use type AttributeValueType::Json,
        // but this simple example just uses a String.
        AttributeValueType::String,
    )?;
    Ok(Response::new()
        // IMPORTANT: The name binding message must be added to the response before the attribute message.
        // All Response messages are executed sequentially, and, in this case, the name must exist before
        // it is used to bind an attribute.  Attempting to swap this order will cause an error, because the
        // attribute module requires an associated existing name, which doesn't exist until the first name
        // bind is executed.
        .add_message(bind_name_msg)
        .add_message(add_attribute_msg)
        .add_attribute("action", "execute_add_attribute")
        .add_attribute("new_attribute_name", new_attribute_name))
}

/// Sends funds provided by the sender in the "amount" field to the specified recipient address.
/// Note: This functionality can easily be accomplished simply by using Provenance's bank module,
/// but this route is here to show how simple it is to send funds in a smart contract.  Using this
/// simple baseline, many complex scenarios can be covered by a smart contract, like holding funds
/// temporarily, or distributing funds to multiple parties within a single execution route.
fn send_funds(
    deps: DepsMut<ProvenanceQuery>,
    info: MessageInfo,
    recipient_address: String,
) -> Result<Response<ProvenanceMsg>, ContractError> {
    // Ensure that funds were provided by the sender, and at least one of the provided values is greater than zero.
    // This is to prevent unnecessary gas fees from being charged for an address that mistakenly attempts to send
    // nothing to another address.
    if !info.funds.iter().any(|coin| coin.amount > Uint128::zero()) {
        return Err(ContractError::InvalidFunds { explanation: "Sender provided no non-zero coins, but the send_funds route requires some funds to be sent".to_string() });
    }
    // Validate that the address is properly-formatted before attempting a send for it.
    // This will create an explicit error denoting a problem with the address, as opposed to a
    // potentially-cryptic error from the bank send msg when it fails to locate the recipient
    // during the send attempt.
    deps.api.addr_validate(&recipient_address)?;
    // Create a BankMsg::Send to shift the funds provided by the address executing the contract to the
    // recipient address. The cosmos CLI allows for multiple funds to be sent at once, so this route can
    // easily and syntactically simply do a multi-coin send.
    let send_msg = CosmosMsg::Bank(BankMsg::Send {
        // Send to the requested target, validated to be a properly-formed address.
        // The recipient address is cloned here to allow it to be moved into the Response as
        // an attribute.
        to_address: recipient_address.clone(),
        // Funnel all funds provided in the "amount" of the transaction directly to the recipient.
        // This ensures that all coin provided goes to the recipient, and that the contract does not
        // hold any of these funds for itself.
        amount: info.funds,
    });
    Ok(Response::new()
        .add_message(send_msg)
        .add_attribute("action", "execute_send_funds")
        .add_attribute("recipient_address", recipient_address))
}
