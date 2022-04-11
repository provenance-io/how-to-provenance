use std::ops::{Add, AddAssign};

use cosmwasm_std::{entry_point, Binary, Deps, DepsMut, Env, MessageInfo, Response, to_binary, Uint128};
use provwasm_std::{ProvenanceMsg, ProvenanceQuery, bind_name, NameBinding, ProvenanceQuerier};

use crate::{
    error::ContractError,
    msg::{ExecuteMsg, InitMsg, MigrateMsg, QueryMsg}, state::{state, State, state_read},
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
    // The flow of a contract is controlled by its return values to its various entry_point functions.
    // If any scenario arises during contract execution that is undesirable or would cause a bad state,
    // returning an error like this is a way to ensure that all changes are completely disregarded.
    // 
    // If funds are ever included in the MessageInfo.funds, the smart contract itself is transferred those funds.
    // This check prevents an instantiation message from seeding the contract with funds that it does not need.
    if !info.funds.is_empty() {
        return Err(ContractError::InvalidFunds { 
            explanation: format!("Funds should not be included in the transaction when instantiating the contract. Found funds: {:?}", info.funds), 
        });
    }
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
    // that will be included in the transaction that this instantiation creates.
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

#[entry_point]
pub fn execute(
    deps: DepsMut<ProvenanceQuery>,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response<ProvenanceMsg>, ContractError> {
    match msg {
        ExecuteMsg::IncrementCounter { increment_amount } => { 
            // If the increment amount provided in the message was present, use it.
            // Otherwise, default to the standard increment amount of 1.
            let amount_to_increment: Uint128 = increment_amount.unwrap_or(1).into();
            let mut state_storage = state(deps.storage);
            // Load the contract state in a mutable manner, allowing the internals to be modified in this execution route
            let mut contract_state = state_storage.load()?;
            contract_state.contract_counter += amount_to_increment;
            state_storage.save(&contract_state)?;
            Ok(Response::new().add_attribute("action", "execute").add_attribute("new_counter_value", contract_state.contract_counter.to_string()))
        }
        ExecuteMsg::AddAttribute { attribute_name, attribute_text } => {
            return Err(ContractError::generic_err("Not complete yet"))
        }
    }
}

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
        QueryMsg::QueryAttribute { attribute_name } => {
            let target_attribute_name = format!("{}.{}", attribute_name, contract_state.contract_base_name);
            let provenance_querier = ProvenanceQuerier::new(&deps.querier);
            let attribute_wrapper = provenance_querier.get_attributes(env.contract.address, Some(target_attribute_name))?;
            if attribute_wrapper.attributes.len() != 1 {
                return Err(ContractError::generic_err(format!("Expected only one attribute to exist at the specified name, but found {}", attribute_wrapper.attributes.len())));
            }
            Ok(attribute_wrapper.attributes.first().unwrap().value.to_owned())
        },
        QueryMsg::QueryCounter {} => {
            Ok(to_binary(&contract_state.contract_counter)?)
        },
        QueryMsg::QueryState {} => {
            Ok(to_binary(&contract_state)?)
        }
    }
}

#[entry_point]
pub fn migrate(
    deps: DepsMut<ProvenanceQuery>,
    _env: Env,
    msg: MigrateMsg,
) -> Result<Response, ContractError> {
    Err(ContractError::generic_err("No migrate exists"))
}
