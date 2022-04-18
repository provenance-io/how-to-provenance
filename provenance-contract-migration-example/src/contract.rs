use cosmwasm_std::{
    entry_point, to_binary, Attribute, BankMsg, Binary, Coin, CosmosMsg, Deps, DepsMut, Env,
    MessageInfo, Response, Uint128,
};
use provwasm_std::{
    add_attribute, bind_name, AttributeValueType, NameBinding, ProvenanceMsg, ProvenanceQuerier,
    ProvenanceQuery,
};

use crate::{
    error::ContractError,
    msg::{ExecuteMsg, InitMsg, MigrateMsg, QueryMsg},
    state::{state, state_read, State},
    version_info::{get_version_info, migrate_version_info, VersionInfo},
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
    check_funds_are_empty(
        info.funds,
        "funds should not be provided when instantiating the contract",
    )?;
    // If a fee detail was provided, it should be validated to ensure that bad interactions do not occur downstream
    if let Some(fee_detail) = &msg.increment_counter_fee {
        fee_detail.self_validate(deps.api)?
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
        increment_counter_fee: msg.increment_counter_fee,
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
    // Before completing instantiation, the default contract name and version should be set in the version info
    // struct and saved to internal storage.  Use the default helper functions declared in the version_info.rs file
    // to do so
    migrate_version_info(deps.storage)?;
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
    match msg {
        QueryMsg::QueryAttribute { attribute_prefix } => {
            let contract_state = state_read(deps.storage).load()?;
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
                    "expected only one attribute to exist at the specified name, but found {}",
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
        // The state  derives Serialize and Deserialize, so it is safe to use to_binary on it to use the
        // entire value as a response and serialize it to a Binary struct.
        QueryMsg::QueryState {} => Ok(to_binary(&state_read(deps.storage).load()?)?),
        // Load the version info in the same way that the state is loaded.  It also derives Serialize and Deserialize,
        // so returning the entire VersionInfo struct as Binary is safe.
        QueryMsg::QueryVersion {} => Ok(to_binary(&get_version_info(deps.storage)?)?),
    }
}

#[entry_point]
pub fn migrate(
    deps: DepsMut<ProvenanceQuery>,
    _env: Env,
    msg: MigrateMsg,
) -> Result<Response, ContractError> {
    // If a previous version has been declared, it's important to ensure that the code that is being migrated
    // is not an older version of the contract.  Otherwise, future migrations can be downgrades, which is not
    // a desired state!  This check attempts to get the version info, but if none can be found due to an error,
    // it is because no version info exists in storage in an older version.
    if let Ok(version_info) = get_version_info(deps.storage) {
        let stored_version = version_info.parse_sem_ver()?;
        let current_version = VersionInfo::current_version().parse_sem_ver()?;
        // This is why VersionInfo leverages the semver crate.  Contract versions can be declared in any fashion one
        // would like, but keeping them in a semver structure allows the semver crate to read them and do comparisons
        // to check if one version is greater than another.  This keeps the code very concise.
        if stored_version >= current_version {
            return Err(ContractError::InvalidVersion { explanation: format!("stored contract version {stored_version} is greater than or equal to the attempted migration version {current_version}. no migration necessary") });
        }
    }
    // After verifying that the migration is to a new and higher version than previously-declared, it's safe to
    // simply invoke the migrate function, which will establish in memory the new version declared in the
    // migrating contract codebase.
    let version_info = migrate_version_info(deps.storage)?;
    // Similarly to how messages are appened in the increment_counter function, this declaration of a mutable
    // vector will store attributes that denote when optional values in the MigrateMsg are encountered. They
    // will be added to the response after all other migration tasks have been completed.
    let mut attributes: Vec<Attribute> = vec![];
    // Do an up-front check to see if any optional values are set.  If this becomes more complex, it may eventually
    // make sense to migrate this logic directly into an impl for MigrateMsg.  However, MigrateMsg currently only
    // contains two fields, so this if-statement is not currently logically cumbersome.
    if msg.new_counter_value.is_some() || msg.increment_counter_fee.is_some() {
        // Both optional values are requests for the contract State struct to be mutated, and it has been confirmed
        // that at least one of them has been requested.  Due to this, preemptively loading the state at this point
        // will never be pointless.
        let mut contract_state = state(deps.storage);
        let mut state = contract_state.load()?;
        if let Some(new_counter_value) = msg.new_counter_value {
            attributes.push(Attribute::new(
                "modified_counter_value",
                format!("{new_counter_value}"),
            ));
            state.contract_counter = Uint128::new(new_counter_value);
        }
        if let Some(increment_counter_fee) = msg.increment_counter_fee {
            // Ensure that the newly-provided fee detail is valid. Otherwise, the migration endpoint
            // would be a way to create an invalid state in the smart contract.
            increment_counter_fee.self_validate(deps.api)?;
            attributes.push(Attribute::new(
                "modified_increment_counter_fee_address",
                &increment_counter_fee.fee_collector_address,
            ));
            attributes.push(Attribute::new(
                "modified_increment_counter_fee_amount",
                increment_counter_fee.get_fee_amount_msg(),
            ));
            state.increment_counter_fee = Some(increment_counter_fee);
        }
        // After modifying the state with one or more optional values, it must be saved for the changes
        // to be persisted into the contract's internal storage
        contract_state.save(&state)?;
    }
    // Keep in mind: Attributes can be added to a Response for the migrate entry_point.  The entry_point
    // can even be declared similarly to the execute entry_point, including a CosmosMsg<ProvenanceMsg> generic
    // typing.  However, during a migration, any messages added to a Response will be completely ignored.
    // The migrate entry_point is not suitable for actions that require executing transactions on the blockchain.
    Ok(Response::new()
        .add_attribute("action", "migrate")
        .add_attribute("new_version", &version_info.version)
        .add_attributes(attributes))
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
fn check_funds_are_empty<S: Into<String>>(
    funds: Vec<Coin>,
    explanation: S,
) -> Result<(), ContractError> {
    if !funds.is_empty() {
        let explanation: String = explanation.into();
        Err(ContractError::InvalidFunds {
            explanation: format!("{explanation}. found funds: {:?}", funds),
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
    let mut state_storage = state(deps.storage);
    // Load the contract state in a mutable manner, allowing the internals to be modified in this execution route
    let mut contract_state = state_storage.load()?;
    // Establish a mutable vector of messages that will get appended to the response after all checks have been made
    let mut messages: Vec<CosmosMsg<ProvenanceMsg>> = vec![];
    if let Some(fee_detail) = &contract_state.increment_counter_fee {
        // Before attempting to charge the fee, ensure that the sender sent the exact fee amount into the contract.
        // If the user any number of coin definitions other than 1, then they either didn't provide any coin, or provided extra
        // coin denominations that won't be detected by the contract.  Rejecting this prevents funds from being stuck in the contract.
        // The second check to ensure that the funds sent are identical to the fee collection detail's amount ensures two things:
        // 1: The sender sent enough coin,
        // 2: The sender did not send too much coin.  Sending too much coin would cause the overage amount to be held in the contract's bank balances,
        // which would essentially "steal" those funds from the sender.
        if info.funds.len() != 1 || info.funds[0] != fee_detail.fee_collection_amount {
            return Err(ContractError::InvalidFunds {
                explanation: format!(
                    "the charge to increment the counter is [{}]. found funds: {:?}",
                    fee_detail.get_fee_amount_msg(),
                    // Map out the funds to a more readable manner, ensuring that they can be printed in a format like "50nhash" as opposed to
                    // the fully-expanded debug explanation for a Coin
                    info.funds
                        .iter()
                        .map(|coin| format!("{}{}", coin.amount.u128(), coin.denom))
                        .collect::<Vec<String>>(),
                ),
            });
        }
        // Append a bank send message that targets the fee collector specified in the fee detail, and
        // sends the exact amount specified in the fee detail.  Due to the verification above, it is
        // assured that this amount is the amount that the sender provided.  This will be a direct
        // pass-through for the provided funds.
        messages.push(CosmosMsg::Bank(BankMsg::Send {
            to_address: fee_detail.fee_collector_address.clone(),
            amount: vec![fee_detail.fee_collection_amount.clone()],
        }));
    } else {
        // Leverage the funds check to ensure that this free execution route does not receive funds at all
        check_funds_are_empty(
            info.funds,
            "funds should not be provided when incrementing the counter",
        )?;
    }
    // If the increment amount provided in the message was present, use it.
    // Otherwise, default to the standard increment amount of 1. This allows the user to
    // completely omit the value from the request payload and still get an increment.
    let amount_to_increment: Uint128 = increment_amount.unwrap_or(1).into();
    contract_state.contract_counter += amount_to_increment;
    // After incrementing the counter, it must be saved to the contract's internal state. This will persist
    // the value, and subsequent increments will see the new value. This will also be available and evident in
    // the query routes.
    state_storage.save(&contract_state)?;
    Ok(Response::new()
        // Include the messages vector from above in the response. If no fee is required by the contract, this vector
        // will be empty, which is completely fine and will not cause errors
        .add_messages(messages)
        .add_attribute("action", "execute_increment_counter")
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
    check_funds_are_empty(
        info.funds,
        "funds should not be provided when adding an attribute",
    )?;
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
        return Err(ContractError::InvalidFunds { explanation: "sender provided no non-zero coins, but the send_funds route requires some funds to be sent".to_string() });
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

// All state functionality for cosmwasm works correctly during test code.
// Provwasm has also supplied a very useful suite for mocking Provenance modules during test execution.
#[cfg(test)]
mod tests {
    use cosmwasm_std::{
        coin, from_binary,
        testing::{mock_env, mock_info, MOCK_CONTRACT_ADDR},
    };
    use provwasm_mocks::mock_dependencies;
    use provwasm_std::{AttributeMsgParams, NameMsgParams, ProvenanceMsgParams};
    use serde_json_wasm::to_string;

    use crate::{
        types::FeeCollectionDetail,
        version_info::{get_version_info, set_version_info, CONTRACT_NAME, CONTRACT_VERSION},
    };

    use super::*;

    // Testing all routes defined in a smart contract is incredibly important!  It can prevent unexpected bugs
    // during actual contract execution. While some aspects of contract execution are difficult to mock, having
    // tested code is vital to a properly-functioning contract.  Given all contract storage and instantiation will
    // incur gas gosts (at minimum - disregarding potential fee charges for other messages emitted), testing is a
    // much cheaper way to ensure your contract is ready than constantly storing and instantiating real contract
    // instances.
    #[test]
    fn test_instantiation_success() {
        // Provwasm provides a useful helper mock_dependencies function that allows for an initial contract balance.
        // This helper provides a mocked set of Deps/DepsMut, QuerierWrapper, etc that can be used to interact with
        // all execution routes.
        let mut deps = mock_dependencies(&[]);
        let response = instantiate(
            deps.as_mut(),
            mock_env(),
            // The mock_info function creates a mocked MessageInfo, which includes the sender address.
            // The sender for an instantiation entry_point call will likely be the contract admin that
            // is specified when the contract is stored. It is possible to not include an admin and allow
            // a contract to be instantiated by alternate sources, however.
            mock_info("admin", &[]),
            InitMsg {
                contract_base_name: "test.pio".to_string(),
                starting_counter: Some(150),
                increment_counter_fee: None,
            },
        )
        .expect("instantiation should complete successfully when all arguments are as expected");
        // Instantiation should produce a single message - a bind_name message
        assert_eq!(
            1,
            response.messages.len(),
            "expected a single message to be included in the instantiation result",
        );
        // It's important to verify that proper input produces the correct messages.
        // These messages will become transactions on the blockchain, so having all paramters set
        // correct is vital
        match response.messages.first().unwrap().to_owned().msg {
            CosmosMsg::Custom(ProvenanceMsg {
                params:
                    ProvenanceMsgParams::Name(NameMsgParams::BindName {
                        name,
                        address,
                        restrict,
                    }),
                ..
            }) => {
                assert_eq!(
                    "test.pio", name,
                    "expected the bound name to be the name included in the InitMsg"
                );
                assert_eq!(
                    MOCK_CONTRACT_ADDR,
                    address.to_string(),
                    "expected the address bound to be the contract address",
                );
                assert!(restrict, "expected the binding to be restricted");
            }
            msg => panic!(
                "unexpected message encountered during instantiation: {:?}",
                msg
            ),
        }
        // It's also important to verify that the proper attributes are added to the contract response.
        // These attributes can be viewed in the Provenance event stream and be used to intercept contract
        // actions as they occur
        assert_eq!(
            2,
            response.attributes.len(),
            "expected both attributes to be added to the response payload",
        );
        assert!(
            response
                .attributes
                .iter()
                .any(|attr| attr.key == "action" && attr.value == "instantiate"),
            "expected the action attribute to be added to the response payload",
        );
        assert!(
            response
                .attributes
                .iter()
                .any(|attr| attr.key == "contract_base_name" && attr.value == "test.pio"),
            "expected the contract_base_name attribute to be added to the response payload",
        );
        // The contract state should be created when instantiation occurs.  This example uses the
        // deps value created by mock_dependencies to interact with the state() function created
        // in the state.rs file to pull the State struct out of internal contract storage.
        let state = state_read(deps.as_ref().storage)
            .load()
            .expect("expected the contract state to be created by instantiation");
        assert_eq!(
            "test.pio", state.contract_base_name,
            "expected the contract base name to be properly set in the state after instantiation",
        );
        assert_eq!(
            150,
            state.contract_counter.u128(),
            "expected the contract counter to be properly set when provided",
        );
        assert!(
            state.increment_counter_fee.is_none(),
            "omitting a counter fee should result in no value being stored to state"
        );
        // Instantiation should also establish the default version info. This is used by deriving the
        // declared contract name and version in the Cargo.toml file
        let version_info = get_version_info(deps.as_ref().storage)
            .expect("version info should be set after instantiation");
        assert_eq!(
            env!("CARGO_CRATE_NAME"),
            version_info.contract,
            "expected the contract name to be set with the default cargo crate name declaration"
        );
        assert_eq!(
            env!("CARGO_PKG_VERSION"),
            version_info.version,
            "expected the contract version to be set with the default cargo package version declaration",
        );
    }

    // This test ensures that when the user omits the starting counter value during instantiation that
    // they will still receive the default value of zero.  It's important to test alternate paths!
    #[test]
    fn test_instantiation_default_counter_value() {
        let mut deps = mock_dependencies(&[]);
        instantiate(
            deps.as_mut(),
            mock_env(),
            mock_info("admin", &[]),
            InitMsg {
                contract_base_name: "test.pio".to_string(),
                starting_counter: None,
                increment_counter_fee: None,
            },
        )
        .expect("instantiation should succeed when arguments are properly supplied, even without a starting_counter value");
        let state = state_read(deps.as_ref().storage)
            .load()
            .expect("expected the contract state to be created by instantiation");
        assert_eq!(
            0,
            state.contract_counter.u128(),
            "the counter value should be set to the default value of zero when no value is provided",
        );
    }

    #[test]
    fn test_instantiation_provided_fee_detail() {
        let mut deps = mock_dependencies(&[]);
        instantiate(
            deps.as_mut(),
            mock_env(),
            mock_info("admin", &[]),
            InitMsg {
                contract_base_name: "test.pio".to_string(),
                starting_counter: None,
                increment_counter_fee: Some(FeeCollectionDetail {
                    fee_collector_address: "fee-collector".to_string(),
                    fee_collection_amount: coin(100, "nhash"),
                }),
            },
        )
        .expect("expected instantiation to succeed");
        let fee_detail = state_read(deps.as_ref().storage)
            .load()
            .expect("expected the contract state to be created by instantiation")
            .increment_counter_fee
            .expect("a provided fee collection detail to instantiation should result in a saved value in state");
        assert_eq!(
            "fee-collector", fee_detail.fee_collector_address,
            "expected the fee_collector_address of the fee detail to match the provided argument"
        );
        assert_eq!(
            100,
            fee_detail.fee_collection_amount.amount.u128(),
            "expected the fee_collection_amount's amount value to match the provided argument",
        );
        assert_eq!(
            "nhash", fee_detail.fee_collection_amount.denom,
            "expected the fee_collection_amount's denom value to match the provided argument",
        );
    }

    // Gotta test that errors occur when they should, as well!
    #[test]
    fn test_instantiation_failures() {
        let mut deps = mock_dependencies(&[]);
        // Verify that provided funds causes instantiation to be rejected
        let error = instantiate(
            deps.as_mut(),
            mock_env(),
            mock_info("admin", &[coin(150, "nhash")]),
            InitMsg {
                contract_base_name: "test.pio".to_string(),
                starting_counter: None,
                increment_counter_fee: None,
            },
        )
        .unwrap_err();
        assert!(
            matches!(error, ContractError::InvalidFunds { .. }),
            "expected provided nhash funds to cause an InvalidFunds ContractError, but got error: {:?}",
            error,
        );
        // Verify that an invalid fee collector detail triggers an error
        let error = instantiate(
            deps.as_mut(),
            mock_env(),
            mock_info("admin", &[]),
            InitMsg {
                contract_base_name: "test.pio".to_string(),
                starting_counter: None,
                increment_counter_fee: Some(FeeCollectionDetail {
                    fee_collector_address: "fee-collector".to_string(),
                    fee_collection_amount: Coin {
                        // A fee collection amount with zero as its chosen amount should be rejected with an error
                        amount: Uint128::zero(),
                        denom: "nhash".to_string(),
                    },
                }),
            },
        )
        .unwrap_err();
        match error {
            ContractError::GenericError(message) => {
                assert_eq!(
                    "fee collection amount must be greater than zero",
                    message,
                    "unexpected generic error message encountered during invalid fee detail check",
                );
            },
            _ => panic!("unexpected error encountered when passing invalid amount value to fee detail: {:?}", error),
        }
    }

    // This test showcases a round-trip through the contract.  It runs an execution route to
    // increment the contract's internal state's counter, and then runs a query to acquire the
    // value as Binary.
    #[test]
    fn test_increment_counter_and_query_flow() {
        let mut deps = mock_dependencies(&[]);
        // Instantiate to create the State and set the intial counter value to 1
        instantiate(
            deps.as_mut(),
            mock_env(),
            mock_info("admin", &[]),
            InitMsg {
                contract_base_name: "test.pio".to_string(),
                starting_counter: Some(1),
                increment_counter_fee: None,
            },
        )
        .expect("instantiation should complete successfully");
        let response = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("example_sender", &[]),
            ExecuteMsg::IncrementCounter {
                increment_amount: Some(5),
            },
        )
        .expect("expected the increment counter response to complete successfully");
        assert!(
            response.messages.is_empty(),
            "the increment counter entry_point should not generate messages"
        );
        assert_eq!(
            2,
            response.attributes.len(),
            "expected two attributes to be generated when increment counter is executed",
        );
        assert!(
            response
                .attributes
                .iter()
                .any(|attr| attr.key == "action" && attr.value == "execute_increment_counter"),
            "expected the action attribute to use the correct value",
        );
        assert!(
            response.attributes.iter().any(|attr| attr.key == "new_counter_value" && attr.value == "6"),
            "expected the new_counter_value attribute to properly indicate that the counter was incremented by 5",
        );
        let state_binary = query(
            // When using the query entry_point, the expected value is a Deps reference, not a DepsMut.
            // The mock dependencies can easily represent this with its .as_ref() function
            deps.as_ref(),
            mock_env(),
            QueryMsg::QueryState {},
        )
        .expect("expected the state query to respond with a binary");
        let state = from_binary::<State>(&state_binary)
            .expect("expected the resulting binary to deserialize to a State without issue");
        assert_eq!(
            6, state.contract_counter.u128(),
            "expected the counter in the contract state to correctly equate to 6, the result of the initial value of 1 + the input value of 5",
        );
    }

    // This test showcases excluding an Option parameter from an execute functionality.  When executing a smart contract
    // that has been deployed to Provenance, the JSON payload sent to the contract at that point would omit the field entirely.
    #[test]
    fn test_increment_counter_without_increment_amount() {
        let mut deps = mock_dependencies(&[]);
        instantiate(
            deps.as_mut(),
            mock_env(),
            mock_info("admin", &[]),
            InitMsg {
                contract_base_name: "test.pio".to_string(),
                starting_counter: None,
                increment_counter_fee: None,
            },
        )
        .expect("instantiation should complete successfully");
        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("example_sender", &[]),
            ExecuteMsg::IncrementCounter {
                increment_amount: None,
            },
        )
        .expect("expected the increment counter response to complete successfully");
        let state = state_read(deps.as_ref().storage)
            .load()
            .expect("expected the state to load correctly");
        assert_eq!(
            1, state.contract_counter.u128(),
            "expected the contract counter to be incremented to 1 from its initial default value of 0",
        );
    }

    #[test]
    fn test_increment_counter_with_fee_charge() {
        let mut deps = mock_dependencies(&[]);
        instantiate(
            deps.as_mut(),
            mock_env(),
            mock_info("admin", &[]),
            InitMsg {
                contract_base_name: "test.pio".to_string(),
                starting_counter: None,
                increment_counter_fee: Some(FeeCollectionDetail {
                    fee_collector_address: "fee-collector".to_string(),
                    fee_collection_amount: coin(100, "nhash"),
                }),
            },
        )
        .expect("expected instantiation to succeed");
        let response = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("example_sender", &[coin(100, "nhash")]),
            ExecuteMsg::IncrementCounter {
                increment_amount: None,
            },
        )
        .expect(
            "expected the correct fee amount provided to increment counter to execute successfully",
        );
        assert_eq!(
            1,
            response.messages.len(),
            "expected a single message to be returned when a fee is required for incrementing the counter",
        );
        match &response.messages.first().unwrap().msg {
            CosmosMsg::Bank(BankMsg::Send { to_address, amount }) => {
                assert_eq!(
                    "fee-collector",
                    to_address,
                    "expected the to_address value to equate to the amount provided during instantiation",
                );
                assert_eq!(
                    1,
                    amount.len(),
                    "expected only a single coin to be sent in the bank send",
                );
                let coin = amount.first().unwrap();
                assert_eq!(
                    100,
                    coin.amount.u128(),
                    "expected the amount sent to equate to the amount provided during instantiation",
                );
                assert_eq!(
                    "nhash", coin.denom,
                    "expected the denom sent to equate to the amount provided during instantiation",
                );
            }
            msg => panic!(
                "unexpected message sent after fee charge for increment: {:?}",
                msg
            ),
        };
    }

    #[test]
    fn test_increment_counter_failures_no_fee_charge() {
        let mut deps = mock_dependencies(&[]);
        instantiate(
            deps.as_mut(),
            mock_env(),
            mock_info("admin", &[]),
            InitMsg {
                contract_base_name: "test.pio".to_string(),
                starting_counter: None,
                increment_counter_fee: None,
            },
        )
        .expect("instantiation should complete successfully");
        let error = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("example_sender", &[coin(10, "fakecoin")]),
            ExecuteMsg::IncrementCounter {
                increment_amount: None,
            },
        )
        .unwrap_err();
        assert!(
            matches!(error, ContractError::InvalidFunds { .. }),
            "expected provided fakecoin funds to cause an InvalidFunds ContractError, but got error: {:?}",
            error,
        );
    }

    #[test]
    fn test_increment_counter_failures_with_fee_charge() {
        let mut deps = mock_dependencies(&[]);
        instantiate(
            deps.as_mut(),
            mock_env(),
            mock_info("admin", &[]),
            InitMsg {
                contract_base_name: "test.pio".to_string(),
                starting_counter: None,
                increment_counter_fee: Some(FeeCollectionDetail {
                    fee_collector_address: "fee-collector".to_string(),
                    fee_collection_amount: coin(100, "nhash"),
                }),
            },
        )
        .expect("expected instantiation to succeed");
        // Declare a re-usable closure to test the same scenario with multiple different inputs
        let mut test_invalid_funds =
            |funds: &[Coin], test_reason: &str, expected_error_text: &str| {
                let error = execute(
                    deps.as_mut(),
                    mock_env(),
                    mock_info("admin", funds),
                    ExecuteMsg::IncrementCounter {
                        increment_amount: None,
                    },
                )
                .unwrap_err();
                match error {
                    ContractError::InvalidFunds { explanation } => {
                        // Ensure the proper error text is being produced to ensure the end user will receive an understandable
                        // message upon executing the contract incorrectly when fees are needed
                        assert_eq!(
                            expected_error_text, explanation,
                            "{}: unexpected InvalidFunds error message encountered",
                            test_reason,
                        );
                    }
                    _ => panic!("{}: unexpected error encountered: {:?}", test_reason, error),
                }
            };
        test_invalid_funds(
            &[],
            "no funds provided",
            "the charge to increment the counter is [100nhash]. found funds: []",
        );
        test_invalid_funds(
            &[coin(99, "nhash")],
            "too few funds provided",
            "the charge to increment the counter is [100nhash]. found funds: [\"99nhash\"]",
        );
        test_invalid_funds(
            &[coin(101, "nhash")],
            "too many funds provided",
            "the charge to increment the counter is [100nhash]. found funds: [\"101nhash\"]",
        );
        test_invalid_funds(
            &[coin(100, "fakecoin")],
            "incorrect funds type provided",
            "the charge to increment the counter is [100nhash]. found funds: [\"100fakecoin\"]",
        );
        test_invalid_funds(
            &[coin(100, "nhash"), coin(1, "otherthing")],
            "too many coin types provided",
            "the charge to increment the counter is [100nhash]. found funds: [\"100nhash\", \"1otherthing\"]",
        );
    }

    // This test showcases how to use provwasm's MockQuerier (encapsulated within the response from mock_dependencies())
    // to mock out responses from the Provenance Attribute module.  Although this test only uses the attribute mock functionality,
    // there are also mocks for the other modules that provwasm covers (like the name module).
    #[test]
    fn test_add_attribute_and_query_flow() {
        let mut deps = mock_dependencies(&[]);
        instantiate(
            deps.as_mut(),
            mock_env(),
            mock_info("admin", &[]),
            InitMsg {
                contract_base_name: "test.pio".to_string(),
                starting_counter: None,
                increment_counter_fee: None,
            },
        )
        .expect("instantiation should complete successfully");
        let response = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("example_sender", &[]),
            ExecuteMsg::AddAttribute {
                attribute_prefix: "example".to_string(),
                attribute_text: "my amazing text".to_string(),
            },
        )
        .expect("expected the add attribute execution route to complete successfully");
        // A bind name message, and an add attribute message should be included in the response
        assert_eq!(
            2,
            response.messages.len(),
            "expected two messages to be included in the response"
        );
        response.messages.iter().for_each(|msg| match &msg.msg {
            CosmosMsg::Custom(ProvenanceMsg {
                params:
                    ProvenanceMsgParams::Name(NameMsgParams::BindName {
                        name,
                        address,
                        restrict,
                    }),
                ..
            }) => {
                assert_eq!("example.test.pio", name, "the name bound should be the concatenation of the contract_base_name and the provided attribute_prefix");
                assert_eq!(MOCK_CONTRACT_ADDR, address.to_string(), "the address used should be that of the contract itself to bind the name");
                assert!(restrict, "expected the bound name to be restricted as was defined in the name bind message");
            }
            CosmosMsg::Custom(ProvenanceMsg {
                params:
                    ProvenanceMsgParams::Attribute(AttributeMsgParams::AddAttribute {
                        address,
                        name,
                        value,
                        value_type,
                    }),
                ..
            }) => {
                assert_eq!(
                    MOCK_CONTRACT_ADDR,
                    address.to_string(),
                    "expected the address the attribute is added to to be the contract's address",
                );
                assert_eq!(
                    "example.test.pio",
                    name,
                    "expected the name the attribute is bound to to be the concatenation of the contract_base_name and the provided attribute_prefix",
                );
                assert_eq!(
                    &AttributeValueType::String,
                    value_type,
                    "expected the used value type to be a String, as was defined in the add_attribute message",
                );
                let value_string = from_binary::<String>(value).expect("expected the value in the add_attribute message to deserialize to a string");
                assert_eq!(
                    "my amazing text",
                    value_string,
                    "expected the deserialized value string to be the value provided to the execute functionality in the attribute_text parameter",
                );
            }
            msg => panic!(
                "unexpected msg encountered after executing add_attribute: {:?}",
                msg
            ),
        });
        assert_eq!(
            2,
            response.attributes.len(),
            "expected two attributes to be included in the add_attribute response",
        );
        assert!(
            response
                .attributes
                .iter()
                .any(|attr| attr.key == "action" && attr.value == "execute_add_attribute"),
            "expected the action attribute to have the proper value",
        );
        assert!(
            response
                .attributes
                .iter()
                .any(|attr| attr.key == "new_attribute_name" && attr.value == "example.test.pio"),
            "expected the new_attribute_name attribute to have the proper value",
        );
        // This is an example of using provwasm's mock_dependencies() to simulate an attribute's existence.
        // When these tests run, the Response values are never consumed by the underlying blockchain code,
        // so any messages emitted in the responses must be manually simulated in order for other processes
        // to acknowledge their existence (like a query).  The previous test assertions proved that the attribute
        // was requested and well-formed, so this mock can simulate its existence after the fact.  A more thoroughly
        // intricate test framework would perhaps add an interceptor function around the execute call to automatically
        // mock attributes in the querier when they are intercepted.
        deps.querier.with_attributes(
            MOCK_CONTRACT_ADDR,
            // This input accepts a slice of Tuples of (&str, &str, &str), which
            // themselves map to: name, value, type
            &[(
                "example.test.pio",
                // The input value, even with plain text, should be encoded using cosmwasm's to_string() serde encoder.
                // This usage can be expanded further for complex scenarios that also utilize structs, as long as they
                // derive Serialize and Deserialize.
                &to_string("my amazing text")
                    .expect("the attribute text should be properly serialized"),
                "string",
            )],
        );
        // With a successful mock established to equate to the derived values from the add_attribute message,
        // a query to the QueryAttribute route should produce the desired output string.
        let query_binary = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::QueryAttribute {
                attribute_prefix: "example".to_string(),
            },
        )
        .expect("the query should execute successfully and find the mocked attribute");
        let attribute_value = from_binary::<String>(&query_binary)
            .expect("the binary should deserialize to a String successfully");
        assert_eq!(
            "my amazing text",
            attribute_value,
            "expected the query to correctly locate the attribute in the attribute module after mocks were created",
        );
    }

    // This test is an example of using provwasm's MockQuerier to mock out a name module response
    // in order to demonstrate a potential error that can be encountered during contract execution.
    #[test]
    fn test_add_attribute_failures() {
        let mut deps = mock_dependencies(&[]);
        instantiate(
            deps.as_mut(),
            mock_env(),
            mock_info("admin", &[]),
            InitMsg {
                contract_base_name: "test.pio".to_string(),
                starting_counter: None,
                increment_counter_fee: None,
            },
        )
        .expect("instantiation should complete successfully");
        // Verify that including funds results in an error
        let error = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("example_sender", &[coin(15, "othercoin")]),
            ExecuteMsg::AddAttribute {
                attribute_prefix: "example".to_string(),
                attribute_text: "my amazing text".to_string(),
            },
        )
        .unwrap_err();
        assert!(
            matches!(error, ContractError::InvalidFunds { .. }),
            "expected provided othercoin funds to cause an InvalidFunds ContractError, but got error: {:?}",
            error,
        );
        // Verify that an existing name results in an error.  This utilizes provwasm's MockQuerier
        // to mock out responses from the name module.
        // This function takes a Tuple of &str that equates to:
        // 0: The fully-qualified name
        // 1: The address that the name is bound to
        // 2: Whether or not the name is restricted
        deps.querier
            .with_names(&[("example.test.pio", MOCK_CONTRACT_ADDR, true)]);
        let error = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("example_sender", &[]),
            ExecuteMsg::AddAttribute {
                attribute_prefix: "example".to_string(),
                attribute_text: "my amazing text".to_string(),
            },
        )
        .unwrap_err();
        assert!(
            matches!(error, ContractError::NameAlreadyExists { .. }),
            "expected an attempt to add an attribute with a name that already exists to be rejected with a NameAlreadyExists error, but got error: {:?}",
            error,
        );
    }

    #[test]
    fn test_send_funds() {
        let mut deps = mock_dependencies(&[]);
        instantiate(
            deps.as_mut(),
            mock_env(),
            mock_info("admin", &[]),
            InitMsg {
                contract_base_name: "test.pio".to_string(),
                starting_counter: None,
                increment_counter_fee: None,
            },
        )
        .expect("instantiation should complete successfully");
        let response = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("example_sender", &[coin(150, "nhash")]),
            ExecuteMsg::SendFunds {
                recipient_address: "recipient".to_string(),
            },
        )
        .expect("the send_funds execution route should complete successfully with proper input");
        assert_eq!(
            1,
            response.messages.len(),
            "expected one message to be included in the response"
        );
        match response.messages.first().unwrap().to_owned().msg {
            CosmosMsg::Bank(BankMsg::Send { to_address, amount }) => {
                assert_eq!(
                    "recipient",
                    to_address,
                    "expected the to_address value to equate to the recipient denoted in the SendFunds ExecuteMsg",
                );
                assert_eq!(
                    1, amount.len(),
                    "expected only a single coin to be sent because only one was specified in the mock_info",
                );
                let coin = amount.first().unwrap();
                assert_eq!(
                    "nhash", coin.denom,
                    "expected the coin's denom to be the specified value in the input"
                );
                assert_eq!(
                    150,
                    coin.amount.u128(),
                    "expected the coin's amount to be the specified amount in the input"
                );
            }
            msg => panic!(
                "unexpected message encountered when executing send_funds: {:?}",
                msg
            ),
        };
        assert_eq!(
            2,
            response.attributes.len(),
            "expected two attributes to be included in the response",
        );
        assert!(
            response
                .attributes
                .iter()
                .any(|attr| attr.key == "action" && attr.value == "execute_send_funds"),
            "expected the action attribute to include the proper value",
        );
        assert!(
            response
                .attributes
                .iter()
                .any(|attr| attr.key == "recipient_address" && attr.value == "recipient"),
            "expected the recipient_address attribute to include the proper value",
        );
    }

    #[test]
    fn test_migrate_all_changes_no_version_info() {
        let mut deps = mock_dependencies(&[]);
        // This test skips instantiation because it is simulating a previous version of the contract never had version info
        // A proper instantiation will bind a name (which is ignored in the test suite, so we can skip that), and will
        // create a state.  To simulate this situation, let's create a state value by itself
        state(deps.as_mut().storage)
            .save(&State {
                contract_base_name: "test.pio".to_string(),
                // Simulate a counter that has been incremented a few times
                contract_counter: Uint128::new(10),
                // A previous contract would not have this Option value, so set it to None to start with
                increment_counter_fee: None,
            })
            .expect("state save should succeed");
        let migration_fee_detail = FeeCollectionDetail {
            fee_collector_address: "fee-collector".to_string(),
            fee_collection_amount: coin(10, "nhash"),
        };
        let response = migrate(
            deps.as_mut(),
            mock_env(),
            MigrateMsg {
                new_counter_value: Some(3),
                increment_counter_fee: Some(migration_fee_detail.clone()),
            },
        )
        .expect("migration should execute successfully");
        assert!(
            response.messages.is_empty(),
            "a migration response should never contain messages"
        );
        assert_eq!(
            5,
            response.attributes.len(),
            "all five of the possible attributes should be added to the response",
        );
        assert!(
            response
                .attributes
                .iter()
                .any(|attr| attr.key == "action" && attr.value == "migrate"),
            "the action attribute should have the correct value",
        );
        assert!(
            response
                .attributes
                .iter()
                .any(|attr| attr.key == "new_version" && attr.value == CONTRACT_VERSION),
            "the new_version attribute should have the correct value",
        );
        assert!(
            response
                .attributes
                .iter()
                .any(|attr| attr.key == "modified_counter_value" && attr.value == "3"),
            "the modified_counter_value attribute should have the correct value",
        );
        assert!(
            response
                .attributes
                .iter()
                .any(|attr| attr.key == "modified_increment_counter_fee_address"
                    && attr.value == "fee-collector"),
            "the modfied_increment_counter_fee_address attribute should have the correct value",
        );
        assert!(
            response
                .attributes
                .iter()
                .any(|attr| attr.key == "modified_increment_counter_fee_amount"
                    && attr.value == "10nhash"),
            "the modified_increment_counter_fee_amount attribute should have the correct value",
        );
        let version_info = get_version_info(deps.as_ref().storage)
            .expect("version info should load after the migration creates it");
        assert_eq!(
            CONTRACT_NAME, version_info.contract,
            "the migration should set the version info's contract name to the contract env value",
        );
        assert_eq!(
            CONTRACT_VERSION, version_info.version,
            "the migration should set the version info's contract name to the contract env value",
        );
        let state = state_read(deps.as_ref().storage)
            .load()
            .expect("state should load after a migration");
        assert_eq!(
            3,
            state.contract_counter.u128(),
            "the contract counter value should be correctly reset to 3 after the migration",
        );
        let fee_detail = state
            .increment_counter_fee
            .expect("the counter fee should be set in the state after the migration");
        assert_eq!(
            migration_fee_detail, fee_detail,
            "the state's fee detail should equate directly to the value used in the migration"
        );
    }

    #[test]
    fn test_migration_from_older_version() {
        let mut deps = mock_dependencies(&[]);
        instantiate(
            deps.as_mut(),
            mock_env(),
            mock_info("admin", &[]),
            InitMsg {
                contract_base_name: "test.pio".to_string(),
                starting_counter: None,
                increment_counter_fee: None,
            },
        )
        .expect("instantiation should succeed");
        // The contract's name and version are only changed by modifications to the Cargo.toml file.
        // This manual adjustment ensures that whatever version the contract currently is coded to be
        // in that file will be larger than this value, ensuring that the migration will not be rejected
        // for having too low a version
        set_version_info(
            deps.as_mut().storage,
            &VersionInfo {
                contract: CONTRACT_NAME.to_string(),
                version: "0.0.0".to_string(),
            },
        )
        .expect("version info change should succeed");
        let response = migrate(
            deps.as_mut(),
            mock_env(),
            MigrateMsg {
                new_counter_value: Some(150),
                increment_counter_fee: Some(FeeCollectionDetail {
                    fee_collector_address: "fee-collector".to_string(),
                    fee_collection_amount: coin(1234, "bitcoin"),
                }),
            },
        )
        .expect("the migration should succeed");
        assert!(
            response.messages.is_empty(),
            "a migration response should never contain messages"
        );
        assert_eq!(
            5,
            response.attributes.len(),
            "all five attributes should be contained in the migration"
        );
        let version_info = get_version_info(deps.as_ref().storage)
            .expect("version info should be fetched successfully");
        assert_eq!(CONTRACT_VERSION, version_info.version, "the contract version should be successfully reset to the proper value after the migration");
    }

    #[test]
    fn test_migration_with_no_optional_values() {
        let mut deps = mock_dependencies(&[]);
        instantiate(
            deps.as_mut(),
            mock_env(),
            mock_info("admin", &[]),
            InitMsg {
                contract_base_name: "test.pio".to_string(),
                starting_counter: None,
                increment_counter_fee: None,
            },
        )
        .expect("instantiation should succeed");
        // The contract's name and version are only changed by modifications to the Cargo.toml file.
        // This manual adjustment ensures that whatever version the contract currently is coded to be
        // in that file will be larger than this value, ensuring that the migration will not be rejected
        // for having too low a version
        set_version_info(
            deps.as_mut().storage,
            &VersionInfo {
                contract: CONTRACT_NAME.to_string(),
                version: "0.0.0".to_string(),
            },
        )
        .expect("version info change should succeed");
        let response = migrate(
            deps.as_mut(),
            mock_env(),
            MigrateMsg {
                new_counter_value: None,
                increment_counter_fee: None,
            },
        )
        .expect("a migration with no optional values should succeed");
        assert!(
            response.messages.is_empty(),
            "a migration response should never contain messages"
        );
        assert_eq!(
            2,
            response.attributes.len(),
            "only the two standard attributes should be included in the response"
        );
        assert!(
            response.attributes.iter().any(|attr| attr.key == "action"),
            "the action attribute should be present in the response",
        );
        assert!(
            response
                .attributes
                .iter()
                .any(|attr| attr.key == "new_version"),
            "the new_version attribute should be present in the respons",
        );
    }

    #[test]
    fn test_migration_failures() {
        let mut deps = mock_dependencies(&[]);
        instantiate(
            deps.as_mut(),
            mock_env(),
            mock_info("admin", &[]),
            InitMsg {
                contract_base_name: "test.pio".to_string(),
                starting_counter: None,
                increment_counter_fee: None,
            },
        )
        .expect("instantiation should succeed");
        // Test that a migration to the same version fails.
        // Simply instantiating the contract to set the version info and then trying to migrate should achieve this
        let error = migrate(
            deps.as_mut(),
            mock_env(),
            MigrateMsg {
                new_counter_value: None,
                increment_counter_fee: None,
            },
        )
        .unwrap_err();
        match error {
            ContractError::InvalidVersion { explanation } => {
                assert_eq!(
                    format!("stored contract version {} is greater than or equal to the attempted migration version {}. no migration necessary", CONTRACT_VERSION, CONTRACT_VERSION),
                    explanation,
                    "expected the correct InvalidVersion explanation",
                );
            }
            _ => panic!(
                "unexpected error encountered when migrating to a matching version: {:?}",
                error
            ),
        }
        // Test that a migration to a lower version fails.
        // Set the contract version to an absurdly-high value to achieve this beforehand
        set_version_info(
            deps.as_mut().storage,
            &VersionInfo {
                contract: CONTRACT_NAME.to_string(),
                version: "999.9.9".to_string(),
            },
        )
        .expect("setting the contract version should succeed");
        let error = migrate(
            deps.as_mut(),
            mock_env(),
            MigrateMsg {
                new_counter_value: None,
                increment_counter_fee: None,
            },
        )
        .unwrap_err();
        match error {
            ContractError::InvalidVersion { explanation } => {
                assert_eq!(
                    format!("stored contract version 999.9.9 is greater than or equal to the attempted migration version {}. no migration necessary", CONTRACT_VERSION),
                    explanation,
                    "expected the correct InvalidVersion explanation",
                );
            }
            _ => panic!(
                "unexpected error encountered when migrating to a matching version: {:?}",
                error
            ),
        }
    }

    #[test]
    fn test_query_version() {
        let mut deps = mock_dependencies(&[]);
        instantiate(
            deps.as_mut(),
            mock_env(),
            mock_info("admin", &[]),
            InitMsg {
                contract_base_name: "test.pio".to_string(),
                starting_counter: None,
                increment_counter_fee: None,
            },
        )
        .expect("instantiation should succeed");
        let version_info_binary = query(deps.as_ref(), mock_env(), QueryMsg::QueryVersion {})
            .expect("version info query should succeed after an instantiation");
        let version_info = from_binary::<VersionInfo>(&version_info_binary)
            .expect("the query result should deserialize to a VersionInfo struct");
        assert_eq!(
            CONTRACT_NAME, version_info.contract,
            "instantiation should set the correct contract name",
        );
        assert_eq!(
            CONTRACT_VERSION, version_info.version,
            "instantiation should set the correct contract version",
        );
    }
}
