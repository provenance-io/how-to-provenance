use crate::core::error::ContractError;
use crate::core::state::{config, meta, NameMeta, State};
use crate::util::constants::FEE_DENOMINATION;
use crate::util::helper_functions::fee_amount_from_string;
use cosmwasm_std::{
    coin, to_binary, Api, BankMsg, CosmosMsg, DepsMut, MessageInfo, Response, Uint128,
};
use cosmwasm_storage::Bucket;
use provwasm_std::{add_attribute, ProvenanceMsg, ProvenanceQuery};

// register a name
// This will bind a name to the account that invoked this contract (self-registration)
// The fee collection address will receive a fee taken out of the funds provided by the
// account invoking this contract when they construct the message to do so.
// note that if something within this execution were to fail, no fee would be taken, and the funds
// would be returned to the invoker, though gas fees may still be paid by the invoker for work performed.
pub fn register_name(
    deps: DepsMut<ProvenanceQuery>,
    info: MessageInfo,
    name: String,
) -> Result<Response<ProvenanceMsg>, ContractError> {
    let config = config(deps.storage).load()?;

    // Fetch the name registry bucket from storage for use in dupe verification, as well as
    // storing the new name if validation passes
    let mut meta_storage = meta(deps.storage);

    // Ensure the provided name has not yet been registered. Bubble up the error if the lookup
    // succeeds in finding the value
    validate_name(name.clone(), &meta_storage)?;

    // Serialize the proposed name as binary, allowing it to be sent via the ProvenanceClient as
    // a new attribute under the registrar
    let name_bin = match to_binary(&name) {
        Ok(bin) => bin,
        Err(e) => {
            return ContractError::NameSerializationFailure { cause: e }.to_result();
        }
    };

    // Construct the new attribute message for dispatch
    // This is a Provenance Blockchain message that will attach an attribute containing the value of the name (as a String) to
    // the account invoking this contract entrypoint. This name is visible outside of this contract, via Provenance Blockchain Explorer, or queries,
    // or by inspecting the state of this contract itself.
    // Note that a user may register multiple names and they will all appear under the same name as an array of attributes on their account.
    let add_attribute_message = add_attribute(
        info.sender.clone(),
        config.clone().name,
        name_bin,
        provwasm_std::AttributeValueType::String,
    )?;

    // Validate that fees are payable and correctly constructed. Errors are properly packaged within
    // the target function, which makes this a perfect candidate for bubbling up via the ? operator
    let charge_response = validate_fee_params_get_messages(deps.api, &info, &config)?;

    // Construct and store a NameMeta to the internal bucket.  This is important, because this
    // registry ensures duplicates names cannot be added, as well as allow addresses to be looked
    // up by name
    let name_meta = NameMeta {
        name: name.clone(),
        address: info.sender.into_string(),
    };
    meta_storage.save(name.as_bytes(), &name_meta)?;

    // Return a response that will dispatch the marker messages and emit events.
    let mut response = Response::new()
        // adding this message to the response results in the blockchain executing this action as the contract in the same transaction after this message is processed.
        // if something were to fail in the processing of this message, all other messages in the same transaction, including the one that invoked this entrypoint
        // would be rolled back, and the contract's state store would not contain a name, and no name would be attached to the invoker's account via an attribute.
        .add_message(add_attribute_message)
        .add_attribute("action", "name_register")
        .add_attribute("name", name);

    // If a fee charge is requested, append it
    if let Some(fee_message) = charge_response.fee_charge_message {
        response = response.add_message(fee_message);
    }

    // If a fee refund must occur, append the constructed message as well as an attribute explicitly
    // detailing the amount of "denom" refunded
    // This functionality is more of a convenience to the invoker of this registration, so they can safely overpay
    // and not lose funds (or receive an error if the contract was configured to do so on provided vs. actual fee mismatch)
    if let Some(refund_message) = charge_response.fee_refund_message {
        response = response.add_message(refund_message).add_attribute(
            "fee_refund",
            format!("{}{}", charge_response.fee_refund_amount, FEE_DENOMINATION),
        );
    }
    Ok(response)
}
/// Validates that a name can be added.  Makes the following checks:
/// - The name is not already registered. Core validation to ensure duplicate registrations cannot occur
/// - The name is all lowercase and does not contain special characters. Ensures all names are easy to recognize.
fn validate_name(name: String, meta: &Bucket<NameMeta>) -> Result<String, ContractError> {
    // If the load doesn't error out, that means it found the input name
    if meta.load(name.as_bytes()).is_ok() {
        return ContractError::NameRegistered { name }.to_result();
    }
    // Ensures that the given name is all lowercase and has no special characters or spaces
    // Note: This would be a great place to have a regex, but the regex cargo itself adds 500K to
    // the file size after optimization, excluding it as an option
    if name.is_empty()
        || name
            .chars()
            .any(|char| !char.is_alphanumeric() || (!char.is_lowercase() && !char.is_numeric()))
    {
        return ContractError::InvalidNameFormat { name }.to_result();
    }
    Ok("successful validation".into())
}

/// Helper struct to make the validate fee params function response more readable
struct FeeChargeResponse {
    fee_charge_message: Option<CosmosMsg<ProvenanceMsg>>,
    fee_refund_message: Option<CosmosMsg<ProvenanceMsg>>,
    fee_refund_amount: u128,
}

/// Verifies that funds provided are correct and enough for a fee charge, and then constructs
/// provenance messages that will provide the correct output during the name registration process.
///
/// The validation performed is:
/// - Ensure no funds provided are of an incorrect denomination.
/// - Ensure that the provided funds sent are >= the fee charge for transactions
/// - Ensure that, if more funds are provided than are needed by for the fee, that the excess is caught and refunded
///
/// Returns:
/// - 1: The message to allocate provided funds to the fee collection account (None if the fee collection amount is instantiated as zero with the contract)
/// - 2: The message to refund the sender with any excess fees (None if the funds provided are exactly equal to the amount of fee required)
/// - 3: The amount refunded.  Will be zero if the perfect fund amount if sent.
/// - Various errors if funds provided are not enough or incorrectly formatted
fn validate_fee_params_get_messages(
    api: &dyn Api,
    info: &MessageInfo,
    config: &State,
) -> Result<FeeChargeResponse, ContractError> {
    // Determine if any funds sent are not of the correct denom
    let invalid_funds = info
        .funds
        .iter()
        .filter(|coin| coin.denom != FEE_DENOMINATION)
        .map(|coin| coin.denom.clone())
        .collect::<Vec<String>>();

    // If any funds are found that do not match the fee denom, exit prematurely to prevent
    // contract from siphoning random funds for no reason
    if !invalid_funds.is_empty() {
        return ContractError::InvalidFundsProvided {
            types: invalid_funds,
        }
        .to_result();
    }

    let nhash_fee_amount = fee_amount_from_string(&config.fee_amount)?;

    // Pull the nhash sent by verifying that only one fund sent is of the nhash variety
    let nhash_sent = match info
        .clone()
        .funds
        .into_iter()
        .find(|coin| coin.denom == FEE_DENOMINATION)
    {
        Some(coin) => coin.amount,
        None => {
            // If fees are required, then a coin of type FEE_DENOMINATION should be sent and the
            // absence of one is an error.  Otherwise, treat omission as purposeful definition of
            // zero money fronted for a fee
            if nhash_fee_amount > 0 {
                return ContractError::NoFundsProvidedForRegistration.to_result();
            } else {
                Uint128::zero()
            }
        }
    };

    // If the amount provided is too low, reject the request because the fee cannot be paid
    if nhash_sent.u128() < nhash_fee_amount {
        return ContractError::InsufficientFundsProvided {
            amount_provided: nhash_sent.u128(),
            amount_required: nhash_fee_amount,
        }
        .to_result();
    }

    // Pull the fee amount from the sender for name registration
    let fee_charge_message = if nhash_fee_amount > 0 {
        Some(CosmosMsg::Bank(BankMsg::Send {
            // The fee collection address is validated on contract instantiation, so there's no need to
            // define custom error messages here
            to_address: api.addr_validate(&config.fee_collection_address)?.into(),
            // The same goes for the fee_amount - it is guaranteed to pass this check
            amount: vec![coin(nhash_fee_amount, FEE_DENOMINATION)],
        }))
    } else {
        None
    };

    // The refund amount is == the total nhash sent - fee charged
    let fee_refund_amount = nhash_sent.u128() - nhash_fee_amount;

    // If more than the fee amount is sent, then respond with an additional message that sends the
    // excess back into the sender's account
    let fee_refund_message = if fee_refund_amount > 0 {
        Some(CosmosMsg::Bank(BankMsg::Send {
            to_address: info.sender.clone().into(),
            amount: vec![coin(fee_refund_amount, FEE_DENOMINATION)],
        }))
    } else {
        None
    };

    Ok(FeeChargeResponse {
        fee_charge_message,
        fee_refund_message,
        fee_refund_amount,
    })
}

#[cfg(test)]
pub mod tests {
    use crate::core::error::ContractError;
    use crate::core::state::meta;
    use crate::execute::register_name::{register_name, validate_name};
    use crate::testutil::instantiation_helpers::{test_instantiate, InstArgs};
    use crate::testutil::test_constants::DEFAULT_FEE_AMOUNT;
    use crate::util::constants::FEE_DENOMINATION;
    use crate::util::helper_functions::fee_amount_from_string;
    use cosmwasm_std::testing::mock_info;
    use cosmwasm_std::{coin, from_binary, BankMsg, Coin, CosmosMsg};
    use provwasm_mocks::mock_dependencies;
    use provwasm_std::{
        AttributeMsgParams, AttributeValueType, ProvenanceMsg, ProvenanceMsgParams,
    };

    #[test]
    fn handle_valid_register() {
        // Create mocks
        let mut deps = mock_dependencies(&[]);

        // Create config state
        test_instantiate(
            deps.as_mut(),
            InstArgs {
                fee_amount: 150,
                fee_collection_address: "no-u",
                ..Default::default()
            },
        )
        .unwrap();

        let res = register_name(
            deps.as_mut(),
            mock_info("somedude", &vec![coin(150, "nhash")]),
            "mycoolname".into(),
        )
        .unwrap();

        // Ensure we have the attribute message and the fee message
        assert_eq!(res.messages.len(), 2);
        res.messages.into_iter().for_each(|msg| match msg.msg {
            CosmosMsg::Custom(ProvenanceMsg { params, .. }) => {
                verify_add_attribute_result(params, "wallet.pb", "mycoolname");
            }
            CosmosMsg::Bank(BankMsg::Send { to_address, amount }) => {
                assert_eq!("no-u", to_address);
                assert_eq!(
                    1,
                    amount.len(),
                    "Only one coin should be specified in the fee transfer message"
                );
                let transferred_coin = amount
                    .first()
                    .expect("Expected the first element of the coin transfer to be accessible");
                assert_eq!(
                    transferred_coin.amount.u128(),
                    fee_amount_from_string("150").unwrap()
                );
                assert_eq!(transferred_coin.denom.as_str(), FEE_DENOMINATION);
            }
            _ => panic!("unexpected message type"),
        });

        // Ensure we got the name event attribute value
        let attribute = res
            .attributes
            .into_iter()
            .find(|attr| attr.key == "name")
            .unwrap();
        assert_eq!(attribute.value, "mycoolname");
    }

    #[test]
    fn test_fee_overage_is_refunded() {
        let mut deps = mock_dependencies(&[]);

        test_instantiate(
            deps.as_mut(),
            InstArgs {
                fee_amount: 150,
                fee_collection_address: "fee_bucket",
                ..Default::default()
            },
        )
        .unwrap();

        // Send 50 more than the required fee amount
        let response = register_name(
            deps.as_mut(),
            mock_info("sender_wallet", &vec![coin(200, FEE_DENOMINATION)]),
            "thebestnameever".into(),
        )
        .unwrap();

        assert_eq!(
            response.messages.len(),
            3,
            "three messages should be returned with an excess fee"
        );

        response.messages.into_iter().for_each(|msg| match msg.msg {
            CosmosMsg::Custom(ProvenanceMsg { params, .. }) => {
                verify_add_attribute_result(params, "wallet.pb", "thebestnameever");
            }
            CosmosMsg::Bank(BankMsg::Send { to_address, amount }) => {
                let coin_amount_sent = validate_and_get_nhash_sent(amount);
                match to_address.as_str() {
                    "fee_bucket" => {
                        assert_eq!(
                            coin_amount_sent, 150,
                            "expected the fee bucket to be sent the instantiated fee amount"
                        );
                    }
                    "sender_wallet" => {
                        assert_eq!(
                            coin_amount_sent, 50,
                            "expected the sender to be refunded the excess funds they added"
                        );
                    }
                    _ => panic!("unexpected to_address encountered"),
                };
            }
            _ => panic!("unexpected message type"),
        });

        assert_eq!(
            3,
            response.attributes.len(),
            "expected three attributes to be added when a refund occurs"
        );
        response
            .attributes
            .iter()
            .find(|attr| attr.key.as_str() == "action")
            .unwrap();
        let name_attr = response
            .attributes
            .iter()
            .find(|attr| attr.key.as_str() == "name")
            .unwrap();
        assert_eq!(name_attr.value.as_str(), "thebestnameever");
        let excess_funds_attr = response
            .attributes
            .iter()
            .find(|attr| attr.key.as_str() == "fee_refund")
            .unwrap();
        assert_eq!(excess_funds_attr.value.as_str(), "50nhash");
    }

    #[test]
    fn test_zero_fee_allows_no_amounts() {
        let mut deps = mock_dependencies(&[]);
        test_instantiate(
            deps.as_mut(),
            InstArgs {
                fee_amount: 0,
                fee_collection_address: "feebucket",
                ..Default::default()
            },
        )
        .unwrap();
        // Send no coin with the request under the assumption that zero fee should allow this
        let zero_fee_resp = register_name(
            deps.as_mut(),
            mock_info("senderwallet", &[]),
            "nameofmine".into(),
        )
        .unwrap();
        assert_eq!(1, zero_fee_resp.messages.len(), "only one message should be responded with because no fee occurred and no refund occurred");
        zero_fee_resp
            .messages
            .into_iter()
            .for_each(|msg| match msg.msg {
                CosmosMsg::Custom(ProvenanceMsg { params, .. }) => {
                    verify_add_attribute_result(params, "wallet.pb", "nameofmine");
                }
                _ => panic!("unexpected response message type"),
            });
        let refund_attr = zero_fee_resp
            .attributes
            .into_iter()
            .find(|attr| attr.key.as_str() == "fee_refund");
        assert!(
            refund_attr.is_none(),
            "no refund should occur with no amount passed in"
        );
    }

    #[test]
    fn test_zero_fee_and_fee_overage_provided_results_in_full_refund() {
        let mut deps = mock_dependencies(&[]);
        test_instantiate(
            deps.as_mut(),
            InstArgs {
                fee_amount: 0,
                fee_collection_address: "feebucket",
                ..Default::default()
            },
        )
        .unwrap();
        // Send a coin overage of nhash to ensure all of it gets returned as a refund
        let refund_resp = register_name(
            deps.as_mut(),
            mock_info("sender_wallet", &vec![coin(200, FEE_DENOMINATION)]),
            "nametouse".into(),
        )
        .unwrap();
        assert_eq!(
            2,
            refund_resp.messages.len(),
            "two messages should be responded with when a fee is not charged, but a refund is made"
        );
        refund_resp.messages.into_iter().for_each(|msg| match msg.msg {
            CosmosMsg::Custom(ProvenanceMsg { params, .. }) => {
                verify_add_attribute_result(params, "wallet.pb", "nametouse");
            }
            CosmosMsg::Bank(BankMsg::Send { to_address, amount }) => {
                assert_eq!(to_address.as_str(), "sender_wallet", "the recipient of the transaction should be the sender because all funds allocated were refunded");
                let coin_amount_sent = validate_and_get_nhash_sent(amount);
                assert_eq!(coin_amount_sent, 200, "all funds sent should be refunded");
            }
            _ => panic!("unexpected response message type"),
        });
        let fee_refund_attr = refund_resp
            .attributes
            .into_iter()
            .find(|attr| attr.key.as_str() == "fee_refund")
            .expect("the refunded fee amount should be added as an attribute");
        assert_eq!(
            fee_refund_attr.value.as_str(),
            "200nhash",
            "expected the refund amount to be indicated as nhash"
        );
    }

    #[test]
    fn test_duplicate_registrations_are_rejected() {
        // Create mocks
        let mut deps = mock_dependencies(&[]);

        // Create config state
        test_instantiate(deps.as_mut(), InstArgs::default()).unwrap();
        let m_info = mock_info("somedude", &vec![coin(DEFAULT_FEE_AMOUNT, "nhash")]);
        // Do first execution to ensure the new name is in there
        register_name(deps.as_mut(), m_info.clone(), "mycoolname".into()).unwrap();
        // Try a duplicate request
        let rejected = register_name(deps.as_mut(), m_info, "mycoolname".into()).unwrap_err();
        match rejected {
            ContractError::NameRegistered { name } => {
                assert_eq!("mycoolname".to_string(), name);
            }
            _ => panic!("unexpected error for proposed duplicate message"),
        };
    }

    #[test]
    fn test_missing_fee_amount_for_registration() {
        let mut deps = mock_dependencies(&[]);
        test_instantiate(deps.as_mut(), InstArgs::default()).unwrap();
        // No fees provided in mock info - this should cause a rejection
        let rejected_no_coin =
            register_name(deps.as_mut(), mock_info("theguy", &[]), "newname".into()).unwrap_err();
        assert!(matches!(
            rejected_no_coin,
            ContractError::NoFundsProvidedForRegistration
        ));
        let incorrect_denom_info = mock_info(
            "theotherguy",
            &vec![
                // Send 3 different types of currencies that the contract is not expected to handle
                coin(DEFAULT_FEE_AMOUNT, "nothash"),
                coin(DEFAULT_FEE_AMOUNT, "fakecoin"),
                coin(DEFAULT_FEE_AMOUNT, "dogecoin"),
                // Provide the a correct value as well to ensure that the validation will reject all
                // requests that include excess
                coin(DEFAULT_FEE_AMOUNT, "nhash"),
            ],
        );
        let rejected_incorrect_type_coin =
            register_name(deps.as_mut(), incorrect_denom_info, "newname".into()).unwrap_err();
        match rejected_incorrect_type_coin {
            ContractError::InvalidFundsProvided { types } => {
                assert_eq!(
                    3,
                    types.len(),
                    "expected the three invalid types to be returned in the rejection"
                );
                types
                    .iter()
                    .find(|coin_type| coin_type.as_str() == "nothash")
                    .unwrap();
                types
                    .iter()
                    .find(|coin_type| coin_type.as_str() == "fakecoin")
                    .unwrap();
                types
                    .iter()
                    .find(|coin_type| coin_type.as_str() == "dogecoin")
                    .unwrap();
            }
            _ => panic!("unexpected error encountered when providing invalid fund types"),
        }
    }

    #[test]
    fn test_invalid_name_format_scenarios() {
        let mut deps = mock_dependencies(&[]);
        let empty_bucket = meta(deps.as_mut().storage);
        // Establish a decent set of non-alphanumeric characters to test against
        let special_characters = vec![
            ".", ",", "<", ">", "/", "?", ";", ":", "'", "\"", "[", "]", "{", "}", "-", "_", "+",
            "=", "(", ")", "*", "&", "^", "%", "$", "#", "@", "!", " ", "\\", "|",
        ];
        special_characters.into_iter().for_each(|character| {
            let test_name = format!("name{}", character);
            let response = validate_name(test_name.clone(), &empty_bucket).unwrap_err();
            assert!(
                matches!(response, ContractError::InvalidNameFormat { .. }),
                "Expected the name {} to be rejected as an invalid name",
                test_name,
            );
        });
        let empty_name_response = validate_name("".into(), &empty_bucket).unwrap_err();
        assert!(
            matches!(empty_name_response, ContractError::InvalidNameFormat { .. }),
            "Expected an empty name to be rejected as invalid input",
        );
        let uppercase_name_response = validate_name("A".into(), &empty_bucket).unwrap_err();
        assert!(
            matches!(
                uppercase_name_response,
                ContractError::InvalidNameFormat { .. }
            ),
            "Expected an uppercase name to be rejected as invalid input",
        );
        validate_name("a1".into(), &empty_bucket)
            .expect("expected a name containing a number to be valid");
    }

    /// Helper to verify that a name was properly registered under the appropriate registrar.
    fn verify_add_attribute_result(
        params: ProvenanceMsgParams,
        expected_registrar: &str,
        expected_result_name: &str,
    ) {
        match params {
            ProvenanceMsgParams::Attribute(AttributeMsgParams::AddAttribute {
                name,
                value,
                value_type,
                ..
            }) => {
                assert_eq!(name, expected_registrar);
                assert_eq!(
                    from_binary::<String>(&value)
                        .expect("unable to deserialize name response binary"),
                    expected_result_name.to_string(),
                );
                assert_eq!(value_type, AttributeValueType::String)
            }
            _ => panic!("unexpected provenance message type"),
        }
    }

    /// Verifies that the amount vector received via a CosmosMsg::BankMsg::Send is the correct
    /// enclosure: One coin result indicating an amount of nhash sent.
    fn validate_and_get_nhash_sent(amount: Vec<Coin>) -> u128 {
        assert_eq!(
            1,
            amount.len(),
            "expected the amount sent to be a single value, indicating one nhash coin"
        );
        amount
            .into_iter()
            .find(|coin| coin.denom == FEE_DENOMINATION)
            .expect(
                format!(
                    "there should be a coin entry of type [{}]",
                    FEE_DENOMINATION
                )
                .as_str(),
            )
            .amount
            .u128()
    }
}
