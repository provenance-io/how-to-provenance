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
    check_funds_are_empty(
        info.funds,
        "funds should not be provided when instantiating the contract",
    )?;
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
    // Leverage the funds check to ensure that this free execution route does not receive funds at all
    check_funds_are_empty(
        info.funds,
        "funds should not be provided when incrementing the counter",
    )?;
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
            },
        )
        .unwrap_err();
        assert!(
            matches!(error, ContractError::InvalidFunds { .. }),
            "expected provided nhash funds to cause an InvalidFunds ContractError, but got error: {:?}",
            error,
        );
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
    fn test_increment_counter_failures() {
        let mut deps = mock_dependencies(&[]);
        instantiate(
            deps.as_mut(),
            mock_env(),
            mock_info("admin", &[]),
            InitMsg {
                contract_base_name: "test.pio".to_string(),
                starting_counter: None,
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
}
