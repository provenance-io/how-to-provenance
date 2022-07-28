use cosmwasm_std::{
    attr, coin, entry_point, to_binary, Addr, BankMsg, Binary, Coin, CosmosMsg, Deps, DepsMut, Env,
    MessageInfo, Response, StdResult, Timestamp, Uint128,
};
use provwasm_std::{
    assess_custom_fee, bind_name, write_scope, NameBinding, Party, PartyType, ProvenanceMsg,
    ProvenanceQuerier, ProvenanceQuery, Scope,
};
use thiserror::private::DisplayAsDisplay;

use crate::contract_info::{get_contract_info, set_contract_info, ContractInfo, CONTRACT_VERSION};
use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::state::{
    get_ask_storage_read_v2, get_ask_storage_v2, get_bid_storage_read_v2, get_bid_storage_v2,
    AskOrderV2, BaseType, BidOrderV2,
};

// smart contract initialization entrypoint
// This will set up a specific instance of this contract on the blockchain that has a unique address (generated upon instantiation)
// the storage containing ask/bid info will be unique to this instance of the smart contract, so only asks/bid_storage
// within the same instance can be matched
#[entry_point]
pub fn instantiate(
    deps: DepsMut<ProvenanceQuery>,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response<ProvenanceMsg>, ContractError> {
    // this is the name to be bound to this contract's address via the Provenance name module
    if msg.bind_name.is_empty() {
        return Err(ContractError::MissingField {
            field: "bind_name".into(),
        });
    }

    // this is simply a human-readable name for the contract
    if msg.contract_name.is_empty() {
        return Err(ContractError::MissingField {
            field: "contract_name".into(),
        });
    }

    // An ask fee of zero is invalid, but omitting the ask fee (None) is valid because it indicates
    // no fee is to be charged
    if msg.ask_fee.is_some() && msg.ask_fee.unwrap().is_zero() {
        return Err(ContractError::InvalidFee {
            fee_type: "ask".to_string(),
        });
    }

    // A bid fee of zero is invalid, but omitting the bid fee (None) is valid because it indicates
    // no fee is to be charged
    if msg.bid_fee.is_some() && msg.bid_fee.unwrap().is_zero() {
        return Err(ContractError::InvalidFee {
            fee_type: "bid".to_string(),
        });
    }

    // set contract info
    let contract_info = ContractInfo::new(
        info.sender,
        msg.bind_name,
        msg.contract_name,
        msg.ask_fee,
        msg.bid_fee,
    );
    set_contract_info(deps.storage, &contract_info)?;

    // create name binding provenance message
    let bind_name_msg = bind_name(
        contract_info.bind_name,
        env.contract.address,
        NameBinding::Restricted,
    )?;

    // build response
    Ok(Response::new()
        .add_messages(vec![bind_name_msg]) // this message will be executed in the same transaction once this function returns
        .add_attributes(vec![
            // these are attributes that will be included in the event resulting from this contract instantiation
            attr(
                "contract_info",
                format!("{:?}", get_contract_info(deps.storage)?),
            ),
            attr("action", "init"),
        ]))
}

// smart contract execute entrypoint
// This is effectively a router for handling requests based on an incoming execute message's type
#[entry_point]
pub fn execute(
    deps: DepsMut<ProvenanceQuery>,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response<ProvenanceMsg>, ContractError> {
    match msg {
        ExecuteMsg::CreateAsk {
            id,
            quote,
            scope_address,
        } => create_ask(deps, env, info, id, quote, scope_address),
        ExecuteMsg::CreateBid {
            id,
            base,
            effective_time,
        } => create_bid(deps, env, info, id, base, effective_time),
        ExecuteMsg::CancelAsk { id } => cancel_ask(deps, env, info, id),
        ExecuteMsg::CancelBid { id } => cancel_bid(deps, env, info, id),
        ExecuteMsg::UpdateFees { ask_fee, bid_fee } => update_fees(deps, info, ask_fee, bid_fee),
        ExecuteMsg::ExecuteMatch { ask_id, bid_id } => {
            execute_match(deps, env, info, ask_id, bid_id)
        }
    }
}

// create ask entrypoint
// This entrypoint will add an entry into ask storage indicating that the owner of some coin/a scope will accept
// some amount of tokens (the quote Vec<Coin>) in exchange for the coin/scope. Upon executing this contract entrypoint,
// the coin up for sale is transferred to the smart contract's control (or in the case of a scope, the scope's ownership has
// to have already been transferred to the contract before executing this method), but will not be transferred to a buyer
// until there is a bid created that matches the quote provided here, and the contract admin executes the match. In the case of
// a base of Coin, the coins have to be provided to the contract via info.funds. The contract will hold onto the provided coins/scope
// until the ask is either matched with a bid by the admin or is cancelled by its owner.
fn create_ask(
    deps: DepsMut<ProvenanceQuery>,
    env: Env,
    info: MessageInfo,
    id: String,
    quote: Vec<Coin>,
    scope_address: Option<String>,
) -> Result<Response<ProvenanceMsg>, ContractError> {
    // the id has to be provided in the message, not generated randomly in the contract as contracts have to be deterministic
    if id.is_empty() {
        return Err(ContractError::MissingField { field: "id".into() });
    }
    if quote.is_empty() {
        return Err(ContractError::MissingField {
            field: "quote".into(),
        });
    }
    let base = if let Some(address) = scope_address {
        // can't provide funds when putting in an ask for a scope
        if !info.funds.is_empty() {
            return Err(ContractError::ScopeAskBaseWithFunds);
        }
        // verify that the scope is owned by the contract prior to consuming via the ask route
        // due to restrictions on permissioning, the scope owner and value owner address must be the contract's address prior to invoking this execute path
        // this should really be done in as a previous message in the same transaction that is invoking this contract endpoint so as to
        // eliminate the risk of the scope being owned by the contract but not yet registered as an ask within the contract
        // (otherwise anyone could subsequently create an ask for someone else's scope and end up with the funds upon a match)
        // ... unfortunately from the perspective of the contract we have no way to enforce that behavior
        check_scope_owners(
            &ProvenanceQuerier::new(&deps.querier).get_scope(&address)?,
            Some(&env.contract.address),
            Some(&env.contract.address),
        )?;
        BaseType::scope(&address)
    } else {
        if info.funds.is_empty() {
            return Err(ContractError::MissingAskBase);
        }
        BaseType::coins(info.funds)
    };

    let mut ask_storage = get_ask_storage_v2(deps.storage);

    // create/store the ask order, mapping the provided base with the quote the seller is willing to accept
    let ask_order = AskOrderV2 {
        base,
        id,
        owner: info.sender,
        quote,
    };
    // key the ask by id to allow for lookup by id later
    ask_storage.save(ask_order.id.as_bytes(), &ask_order)?;

    let mut response = Response::new()
        // anything watching the event stream could see an event from this contract with this attribute, and then act on it if desired
        .add_attribute("action", "create_ask")
        .set_data(to_binary(&ask_order)?);

    let contract_info = get_contract_info(deps.storage)?;

    // Only generate an ask fee message if it is configured within the contract info
    if let Some(ref ask_fee) = &contract_info.ask_fee {
        response = response
            .add_attribute("fee_charged", format!("{}nhash", ask_fee.as_display()))
            .add_message(generate_creation_fee(
                ask_fee.u128(),
                "Ask",
                env.contract.address,
                contract_info.admin,
            )?);
    }

    Ok(response)
}

// create bid entrypoint
// this entrypoint will add an entry into bid storage indicating that this potential buyer will provide
// some amount of tokens (the funds provided to the contract in the info argument) in exchange for the provided base
// (some other set of coins/a scope). Note that in order to create a bid, the bidder has to send funds into the contract
// that will be held/managed by the contract until either this bid is matched with an appropriate ask by the admin, or
// this bid is cancelled.
fn create_bid(
    deps: DepsMut<ProvenanceQuery>,
    env: Env,
    info: MessageInfo,
    id: String,
    base: BaseType,
    effective_time: Option<Timestamp>,
) -> Result<Response<ProvenanceMsg>, ContractError> {
    // you have to provide information on what you are wanting to buy
    // the case of a scope base isn't checked, as the scope doesn't necessarily even have to exist yet,
    // it is just an address that could be created in the future (i.e. maybe there is some off-chain agreement in progress)
    if let BaseType::Coin { coins } = &base {
        if coins.is_empty() {
            return Err(ContractError::MissingField {
                field: "base".into(),
            });
        }
    }

    // the id has to be provided in the message, not generated randomly in the contract as contracts have to be deterministic
    if id.is_empty() {
        return Err(ContractError::MissingField { field: "id".into() });
    }
    // the bidder has to send funds into the contract in order to buy something/determine the quote amount
    if info.funds.is_empty() {
        return Err(ContractError::MissingBidQuote);
    }

    let mut bid_storage = get_bid_storage_v2(deps.storage);

    // create/store the bid details
    let bid_order = BidOrderV2 {
        base,
        effective_time,
        id,
        owner: info.sender,
        quote: info.funds,
    };
    // key the bid by id so it can be retrieved as such later
    bid_storage.save(bid_order.id.as_bytes(), &bid_order)?;

    let mut response = Response::new()
        // anything watching the event stream could see an event from this contract with this attribute, and then act on it if desired
        .add_attributes(vec![attr("action", "create_bid")])
        .set_data(to_binary(&bid_order)?);

    let contract_info = get_contract_info(deps.storage)?;

    // Only generate a bid fee message if it is configured within the contract info
    if let Some(ref bid_fee) = &contract_info.bid_fee {
        response = response
            .add_attribute("fee_charged", format!("{}nhash", bid_fee.as_display()))
            .add_message(generate_creation_fee(
                bid_fee.u128(),
                "Bid",
                env.contract.address,
                contract_info.admin,
            )?);
    }

    Ok(response)
}

fn generate_creation_fee<S: Into<String>>(
    fee_amount: u128,
    fee_type: S,
    contract_address: Addr,
    admin_address: Addr,
) -> Result<CosmosMsg<ProvenanceMsg>, ContractError> {
    Ok(assess_custom_fee(
        // Custom fees must either use nhash or usd as the denom.  This example uses nhash,
        // because the fee is intended to ensure the admin has enough funds to execute matches
        // in a sustainable manner (aka no account reloading to keep the admin functional)
        coin(fee_amount, "nhash"),
        // The custom fee name is free-form text and displays in the Provenance Blockchain Wallet
        // when the fee is added to the contract dispatch
        Some(format!("{} creation fee", fee_type.into())),
        // The contract address must always be specified in the "from" field.  This is due to
        // the fact that smart contracts can only ever execute messages that they sign for.
        // Even though the "from" field is the contract, the signer of the message to the contract
        // will be required to pay the fee.  This message is essentially an intermediary that tells
        // the Provenance Blockchain to charge the fee from the sender of the contract execute
        // message.
        contract_address,
        // If this recipient field is not provided, then the entirety of the fee is sent to the
        // Provenance Blockchain fee module.  In this case, though, the admin requires a cut of
        // the charged fee, so it is specified here.
        Some(admin_address),
    )?)
}

// cancel ask entrypoint
// this entrypoint allows the account that created an ask to cancel the ask, transferring the base back to them and
// effectively taking it off the market and preventing any match from happening in the future
fn cancel_ask(
    deps: DepsMut<ProvenanceQuery>,
    env: Env,
    info: MessageInfo,
    id: String,
) -> Result<Response<ProvenanceMsg>, ContractError> {
    // return error if id is empty, we need to know which ask to cancel
    if id.is_empty() {
        return Err(ContractError::Unauthorized {});
    }

    // return error if funds sent (this entrypoint is only to return funds to the owner, not accept new funds)
    // note that this has nothing to do with the gas fees incurred by executing this contract
    if !info.funds.is_empty() {
        return Err(ContractError::CancelWithFunds {});
    }

    // try and find the ask being cancelled
    let ask_storage = get_ask_storage_read_v2(deps.storage);
    let stored_ask_order = ask_storage.load(id.as_bytes());
    match stored_ask_order {
        Err(_) => Err(ContractError::Unauthorized {}),
        Ok(stored_ask_order) => {
            if !info.sender.eq(&stored_ask_order.owner) {
                return Err(ContractError::Unauthorized {});
            }

            // remove the ask order from storage
            let mut ask_storage = get_ask_storage_v2(deps.storage);
            ask_storage.remove(id.as_bytes());

            let mut messages: Vec<CosmosMsg<ProvenanceMsg>> = vec![];

            // determine which type of base this ask was for (a scope or coins) and return as appropriate by
            // either transferring the coin to the ask owner's account, or setting the ask owner as the scope's owner
            match stored_ask_order.base {
                BaseType::Coin { coins } => {
                    messages.push(cosmwasm_std::CosmosMsg::Bank(BankMsg::Send {
                        to_address: stored_ask_order.owner.to_string(),
                        amount: coins,
                    }));
                }
                BaseType::Scope { scope_address } => {
                    // fetch scope
                    let scope = ProvenanceQuerier::new(&deps.querier).get_scope(scope_address)?;

                    // Set the original asker's address back to being the owner and value owner address
                    messages.push(write_scope(
                        replace_scope_owner(scope, stored_ask_order.owner)?,
                        vec![env.contract.address],
                    )?);
                }
            };

            // 'send base back to owner' message
            Ok(Response::new()
                // whatever messages were produced (in order to return the base to the owner) have to be added to the
                // response so they can be executed after this function returns in the same transaction
                .add_messages(messages)
                // anything watching the event stream could see an event from this contract with this attribute, and then act on it if desired
                .add_attributes(vec![attr("action", "cancel_ask")]))
        }
    }
}

// cancel bid entrypoint
// this entrypoint allows the account that created an bid to cancel the bid, transferring the quote (provided funds) back to them and
// preventing any match from happening in the future using those funds
fn cancel_bid(
    deps: DepsMut<ProvenanceQuery>,
    _env: Env,
    info: MessageInfo,
    id: String,
) -> Result<Response<ProvenanceMsg>, ContractError> {
    // return error if id is empty, we need to know which bid to cancel
    if id.is_empty() {
        return Err(ContractError::Unauthorized {});
    }

    // return error if funds sent
    if !info.funds.is_empty() {
        return Err(ContractError::CancelWithFunds {});
    }

    // try and find the bid being cancelled
    let bid_storage = get_bid_storage_read_v2(deps.storage);
    let stored_bid_order = bid_storage.load(id.as_bytes());
    match stored_bid_order {
        Ok(stored_bid_order) => {
            if !info.sender.eq(&stored_bid_order.owner) {
                return Err(ContractError::Unauthorized {});
            }

            // remove the ask order from storage
            let mut bid_storage = get_bid_storage_v2(deps.storage);
            bid_storage.remove(id.as_bytes());

            // 'send quote back to owner' message
            Ok(Response::new()
                // whatever messages were produced (in order to return the base to the owner) have to be added to the
                // response so they can be executed after this function returns in the same transaction
                .add_message(BankMsg::Send {
                    to_address: stored_bid_order.owner.to_string(),
                    amount: stored_bid_order.quote,
                })
                // anything watching the event stream could see an event from this contract with this attribute, and then act on it if desired
                .add_attributes(vec![attr("action", "cancel_bid")]))
        }
        Err(_) => Err(ContractError::Unauthorized {}),
    }
}

fn update_fees(
    deps: DepsMut<ProvenanceQuery>,
    info: MessageInfo,
    ask_fee: Option<Uint128>,
    bid_fee: Option<Uint128>,
) -> Result<Response<ProvenanceMsg>, ContractError> {
    let mut contract_info = get_contract_info(deps.storage)?;
    // Prevent any users beside the admin from executing this route
    if info.sender != contract_info.admin {
        return Err(ContractError::Unauthorized {});
    }
    // Prevent funds from accidentally being escrowed in the contract
    if !info.funds.is_empty() {
        return Err(ContractError::UpdateFeesWithFunds {});
    }
    // An ask fee of zero is invalid, but omitting the ask fee (None) is valid because it indicates
    // no fee is to be charged
    if ask_fee.is_some() && ask_fee.unwrap().is_zero() {
        return Err(ContractError::InvalidFee {
            fee_type: "ask".to_string(),
        });
    }
    // A bid fee of zero is invalid, but omitting the bid fee (None) is valid because it indicates
    // no fee is to be charged
    if bid_fee.is_some() && bid_fee.unwrap().is_zero() {
        return Err(ContractError::InvalidFee {
            fee_type: "bid".to_string(),
        });
    }
    // Format the ask and bid fee output messages up front before declaring the response, allowing
    // for the ask fee and bid fee to be moved into the contract_info and avoiding errors
    let ask_fee_message = if let Some(ref ask_fee) = &ask_fee {
        format!("{}nhash", ask_fee.u128())
    } else {
        "cleared".to_string()
    };
    let bid_fee_message = if let Some(ref bid_fee) = &bid_fee {
        format!("{}nhash", bid_fee.u128())
    } else {
        "cleared".to_string()
    };
    // Overwrite the ask and bid fees in the contract info and save the new result
    contract_info.ask_fee = ask_fee;
    contract_info.bid_fee = bid_fee;
    set_contract_info(deps.storage, &contract_info)?;
    Ok(Response::new()
        .add_attribute("action", "update_fees")
        .add_attribute("new_ask_fee", ask_fee_message)
        .add_attribute("new_bid_fee", bid_fee_message))
}

// match and execute an ask and bid order
// this allows for the atomic transfer of the bid funds to the seller and the quote asset (coin/scope) to the bidder,
// ensuring neither party has chance to back out of the deal after a partial transfer
fn execute_match(
    deps: DepsMut<ProvenanceQuery>,
    env: Env,
    info: MessageInfo,
    ask_id: String,
    bid_id: String,
) -> Result<Response<ProvenanceMsg>, ContractError> {
    // only the admin may execute matches
    if info.sender != get_contract_info(deps.storage)?.admin {
        return Err(ContractError::Unauthorized {});
    }

    // return error if id is empty
    if ask_id.is_empty() | bid_id.is_empty() {
        return Err(ContractError::Unauthorized {});
    }

    // return error if funds sent
    if !info.funds.is_empty() {
        return Err(ContractError::ExecuteWithFunds {});
    }

    let ask_storage_read = get_ask_storage_read_v2(deps.storage);
    let ask_order_result = ask_storage_read.load(ask_id.as_bytes());
    if ask_order_result.is_err() {
        return Err(ContractError::AskBidMismatch {});
    }

    let bid_storage_read = get_bid_storage_read_v2(deps.storage);
    let bid_order_result = bid_storage_read.load(bid_id.as_bytes());
    if bid_order_result.is_err() {
        return Err(ContractError::AskBidMismatch {});
    }

    let ask_order = ask_order_result.unwrap();
    let bid_order = bid_order_result.unwrap();

    // this is possibly the most critical piece of this entrypoint, in that it ensures the price the bidder is paying is
    // the same as what the seller listed their asset for sale at
    if !is_executable(&ask_order, &bid_order) {
        return Err(ContractError::AskBidMismatch {});
    }

    // 'send quote to asker' and 'send base to bidder' messages
    let response = Response::new().add_message(BankMsg::Send {
        to_address: ask_order.owner.to_string(),
        amount: ask_order.quote,
    });
    let mut messages: Vec<CosmosMsg<ProvenanceMsg>> = vec![];

    match bid_order.base {
        BaseType::Coin { coins } => messages.push(cosmwasm_std::CosmosMsg::Bank(BankMsg::Send {
            to_address: bid_order.owner.to_string(),
            amount: coins,
        })),
        BaseType::Scope { scope_address } => {
            // fetch scope
            let scope = ProvenanceQuerier::new(&deps.querier).get_scope(scope_address)?;

            messages.push(write_scope(
                replace_scope_owner(scope, bid_order.owner)?,
                vec![env.contract.address],
            )?)
        }
    };

    // finally remove the orders from storage
    get_ask_storage_v2(deps.storage).remove(ask_id.as_bytes());
    get_bid_storage_v2(deps.storage).remove(bid_id.as_bytes());

    Ok(response
        // whatever messages were produced (in order to return the base to the owner) have to be added to the
        // response so they can be executed after this function returns in the same transaction
        .add_messages(messages)
        // anything watching the event stream could see an event from this contract with this attribute, and then act on it if desired
        .add_attributes(vec![attr("action", "execute")]))
}

// the logic determining if an ask/bid are actually a legitinate match
fn is_executable(ask_order: &AskOrderV2, bid_order: &BidOrderV2) -> bool {
    // sort the base and quote vectors by the order chain: denom, amount
    // this ensures that the ask/bid can be repeatably compared with the same result
    let coin_sorter =
        |a: &Coin, b: &Coin| a.denom.cmp(&b.denom).then_with(|| a.amount.cmp(&b.amount));

    let ask_base = ask_order.base.to_owned().sorted();
    let bid_base = bid_order.base.to_owned().sorted();

    let mut ask_quote = ask_order.quote.to_owned();
    ask_quote.sort_by(coin_sorter);
    let mut bid_quote = bid_order.quote.to_owned();
    bid_quote.sort_by(coin_sorter);

    ask_base == bid_base && ask_quote == bid_quote
}

/// Verifies that the scope is properly owned.  At minimum, checks that the scope has only a singular owner.
/// If expected_owner is provided, the single owner with party type Owner must match this address.
/// If expected_value_owner is provided, the value_owner_address value must match this.
fn check_scope_owners(
    scope: &Scope,
    expected_owner: Option<&Addr>,
    expected_value_owner: Option<&Addr>,
) -> Result<(), ContractError> {
    let owners = scope
        .owners
        .iter()
        .filter(|owner| owner.role == PartyType::Owner)
        .collect::<Vec<&Party>>();
    // if more than one owner is specified, removing all of them can potentially cause data loss
    if owners.len() != 1 {
        return Err(ContractError::InvalidScopeOwner {
            scope_address: scope.scope_id.clone(),
            explanation: format!(
                "the scope should only include a single owner, but found: {}",
                owners.len(),
            ),
        });
    }
    if let Some(expected) = expected_owner {
        let owner = owners.first().unwrap();
        if &owner.address != expected {
            return Err(ContractError::InvalidScopeOwner {
                scope_address: scope.scope_id.clone(),
                explanation: format!(
                    "the scope owner was expected to be [{}], not [{}]",
                    expected, owner.address,
                ),
            });
        }
    }
    if let Some(expected) = expected_value_owner {
        if &scope.value_owner_address != expected {
            return Err(ContractError::InvalidScopeOwner {
                scope_address: scope.scope_id.clone(),
                explanation: format!(
                    "the scope's value owner was expected to be [{}], not [{}]",
                    expected, scope.value_owner_address,
                ),
            });
        }
    }
    Ok(())
}

/// Switches the scope's current owner value to the given owner value.
fn replace_scope_owner(mut scope: Scope, new_owner: Addr) -> Result<Scope, ContractError> {
    // Empty out all owners from the scope now that it's verified safe to do
    scope.owners = scope
        .owners
        .into_iter()
        .filter(|owner| owner.role != PartyType::Owner)
        .collect();
    // Append the target value as the new sole owner
    scope.owners.push(Party {
        address: new_owner.clone(),
        role: PartyType::Owner,
    });
    // Swap over the value owner, ensuring that the target owner not only is listed as an owner,
    // but has full access control over the scope
    scope.value_owner_address = new_owner;
    Ok(scope)
}

// smart contract query entrypoint
// this allows anyone to inspect the details of the contract or specific asks/bids
// note that the raw underlying contract storage is also available to anyone who so desires to look
// this can be queried via code off-chain or other smart contracts. Potentially the details of an ask/bid
// may be of interest to an application/individual that wants to buy/sell something listed as a base
#[entry_point]
pub fn query(deps: Deps<ProvenanceQuery>, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetAsk { id } => {
            let ask_storage_read = get_ask_storage_read_v2(deps.storage);
            return to_binary(&ask_storage_read.load(id.as_bytes())?);
        }
        QueryMsg::GetBid { id } => {
            let bid_storage_read = get_bid_storage_read_v2(deps.storage);
            return to_binary(&bid_storage_read.load(id.as_bytes())?);
        }
        QueryMsg::GetContractInfo {} => to_binary(&get_contract_info(deps.storage)?),
    }
}

// the router for handling the raw migrate message. In this case there is only one type of migration
#[entry_point]
pub fn migrate(
    deps: DepsMut<ProvenanceQuery>,
    _env: Env,
    msg: MigrateMsg,
) -> Result<Response, ContractError> {
    match msg {
        MigrateMsg::NewVersion {} => migrate_new_version(deps),
    }
}

// just set the new version in the contract storage.
// If the structure of ask/bid storage were to change between versions, you might need to iterate through all entries
// and modify each from the old format to the new
fn migrate_new_version(deps: DepsMut<ProvenanceQuery>) -> Result<Response, ContractError> {
    let mut contract_info = get_contract_info(deps.storage)?;
    // Bump version in contract info the version stored in the wasm
    contract_info.contract_version = CONTRACT_VERSION.into();
    set_contract_info(deps.storage, &contract_info)?;
    Ok(Response::new().add_attribute("action", "migrate"))
}

// unit tests
#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::{mock_env, mock_info, MOCK_CONTRACT_ADDR};
    use cosmwasm_std::{coin, coins, Addr, BankMsg, StdError};
    use cosmwasm_std::{CosmosMsg, Uint128};
    use provwasm_mocks::mock_dependencies;
    use provwasm_std::{
        MetadataMsgParams, MsgFeesMsgParams, NameMsgParams, ProvenanceMsg, ProvenanceMsgParams,
        ProvenanceRoute,
    };

    use crate::contract_info::{ContractInfo, CONTRACT_TYPE, CONTRACT_VERSION};
    use crate::state::{get_bid_storage_read_v2, BaseType};

    use super::*;
    use crate::msg::ExecuteMsg;

    #[test]
    fn test_is_executable() {
        assert!(is_executable(
            &AskOrderV2 {
                base: BaseType::coin(100, "base_1"),
                id: "ask_id".to_string(),
                owner: Addr::unchecked("asker"),
                quote: coins(100, "quote_1"),
            },
            &BidOrderV2 {
                base: BaseType::coin(100, "base_1"),
                effective_time: Some(Timestamp::default()),
                id: "bid_id".to_string(),
                owner: Addr::unchecked("bidder"),
                quote: coins(100, "quote_1"),
            }
        ));
        assert!(is_executable(
            &AskOrderV2 {
                base: BaseType::coins(vec![coin(100, "base_1"), coin(200, "base_2")]),
                id: "ask_id".to_string(),
                owner: Addr::unchecked("asker"),
                quote: coins(100, "quote_1"),
            },
            &BidOrderV2 {
                base: BaseType::coins(vec![coin(200, "base_2"), coin(100, "base_1")]),
                effective_time: Some(Timestamp::default()),
                id: "bid_id".to_string(),
                owner: Addr::unchecked("bidder"),
                quote: coins(100, "quote_1"),
            }
        ));
        assert!(is_executable(
            &AskOrderV2 {
                base: BaseType::scope("scope1234"),
                id: "ask_id".to_string(),
                owner: Addr::unchecked("asker"),
                quote: coins(100, "quote_1"),
            },
            &BidOrderV2 {
                base: BaseType::scope("scope1234"),
                effective_time: Some(Timestamp::default()),
                id: "bid_id".to_string(),
                owner: Addr::unchecked("bidder"),
                quote: coins(100, "quote_1"),
            }
        ));
        assert!(!is_executable(
            &AskOrderV2 {
                base: BaseType::coin(100, "base_1"),
                id: "ask_id".to_string(),
                owner: Addr::unchecked("asker"),
                quote: coins(100, "quote_1"),
            },
            &BidOrderV2 {
                base: BaseType::coin(100, "base_2"),
                effective_time: Some(Timestamp::default()),
                id: "bid_id".to_string(),
                owner: Addr::unchecked("bidder"),
                quote: coins(100, "quote_1"),
            }
        ));
        assert!(!is_executable(
            &AskOrderV2 {
                base: BaseType::coin(100, "base_1"),
                id: "ask_id".to_string(),
                owner: Addr::unchecked("asker"),
                quote: coins(100, "quote_1"),
            },
            &BidOrderV2 {
                base: BaseType::coin(100, "base_1"),
                effective_time: Some(Timestamp::default()),
                id: "bid_id".to_string(),
                owner: Addr::unchecked("bidder"),
                quote: coins(100, "quote_2"),
            }
        ));
        assert!(!is_executable(
            &AskOrderV2 {
                base: BaseType::scope("scope1234"),
                id: "ask_id".to_string(),
                owner: Addr::unchecked("asker"),
                quote: coins(100, "quote_1"),
            },
            &BidOrderV2 {
                base: BaseType::coin(100, "base_1"),
                effective_time: Some(Timestamp::default()),
                id: "bid_id".to_string(),
                owner: Addr::unchecked("bidder"),
                quote: coins(100, "quote_1"),
            }
        ));
        assert!(!is_executable(
            &AskOrderV2 {
                base: BaseType::scope("scope1234"),
                id: "ask_id".to_string(),
                owner: Addr::unchecked("asker"),
                quote: coins(100, "quote_1"),
            },
            &BidOrderV2 {
                base: BaseType::scope("scope4321"),
                effective_time: Some(Timestamp::default()),
                id: "bid_id".to_string(),
                owner: Addr::unchecked("bidder"),
                quote: coins(100, "quote_1"),
            }
        ));
    }

    #[test]
    fn instantiate_with_valid_data() {
        // create valid init data
        let mut deps = mock_dependencies(&[]);
        let info = mock_info("contract_admin", &[]);
        let init_msg = InstantiateMsg {
            bind_name: "contract_bind_name".to_string(),
            contract_name: "contract_name".to_string(),
            ask_fee: None,
            bid_fee: None,
        };

        // initialize
        let init_response = instantiate(deps.as_mut(), mock_env(), info, init_msg.clone());

        // verify initialize response
        match init_response {
            Ok(init_response) => {
                assert_eq!(init_response.messages.len(), 1);
                assert_eq!(
                    init_response.messages[0].msg,
                    CosmosMsg::Custom(ProvenanceMsg {
                        route: ProvenanceRoute::Name,
                        params: ProvenanceMsgParams::Name(NameMsgParams::BindName {
                            name: init_msg.bind_name,
                            address: Addr::unchecked(MOCK_CONTRACT_ADDR),
                            restrict: true
                        }),
                        version: "2.0.0".to_string(),
                    })
                );
                let expected_contract_info = ContractInfo {
                    admin: Addr::unchecked("contract_admin"),
                    bind_name: "contract_bind_name".to_string(),
                    contract_name: "contract_name".to_string(),
                    contract_type: CONTRACT_TYPE.into(),
                    contract_version: CONTRACT_VERSION.into(),
                    ask_fee: None,
                    bid_fee: None,
                };

                assert_eq!(init_response.attributes.len(), 2);
                assert_eq!(
                    init_response.attributes[0],
                    attr("contract_info", format!("{:?}", expected_contract_info))
                );
                assert_eq!(init_response.attributes[1], attr("action", "init"));
            }
            error => panic!("failed to initialize: {:?}", error),
        }
    }

    #[test]
    fn instantiate_with_invalid_data() {
        // create invalid init data
        let mut deps = mock_dependencies(&[]);
        let info = mock_info("contract_owner", &[]);
        let init_msg = InstantiateMsg {
            bind_name: "".to_string(),
            contract_name: "contract_name".to_string(),
            ask_fee: None,
            bid_fee: None,
        };

        // initialize
        let init_response = instantiate(deps.as_mut(), mock_env(), info.to_owned(), init_msg);

        // verify initialize response
        match init_response {
            Ok(_) => panic!("expected error, but init_response ok"),
            Err(error) => match error {
                ContractError::MissingField { field } => {
                    assert_eq!(field, "bind_name")
                }
                error => panic!("unexpected error: {:?}", error),
            },
        };

        let init_msg = InstantiateMsg {
            bind_name: "bind_name".to_string(),
            contract_name: "".to_string(),
            ask_fee: None,
            bid_fee: None,
        };

        // initialize
        let init_response = instantiate(deps.as_mut(), mock_env(), info.to_owned(), init_msg);

        // verify initialize response
        match init_response {
            Ok(_) => panic!("expected error, but init_response ok"),
            Err(error) => match error {
                ContractError::MissingField { field } => {
                    assert_eq!(field, "contract_name")
                }
                error => panic!("unexpected error: {:?}", error),
            },
        };

        let init_msg = InstantiateMsg {
            bind_name: "bind_name".to_string(),
            contract_name: "contract_name".to_string(),
            ask_fee: Some(Uint128::zero()),
            bid_fee: None,
        };

        let init_response = instantiate(deps.as_mut(), mock_env(), info.to_owned(), init_msg);

        match init_response {
            Ok(_) => panic!("expected error, but init_response ok"),
            Err(error) => match error {
                ContractError::InvalidFee { fee_type } => {
                    assert_eq!(fee_type, "ask");
                }
                error => panic!("unexpected error: {:?}", error),
            },
        };

        let init_msg = InstantiateMsg {
            bind_name: "bind_name".to_string(),
            contract_name: "contract_name".to_string(),
            ask_fee: Some(Uint128::new(100)),
            bid_fee: Some(Uint128::zero()),
        };

        let init_response = instantiate(deps.as_mut(), mock_env(), info, init_msg);

        match init_response {
            Ok(_) => panic!("expected error, but init_response ok"),
            Err(error) => match error {
                ContractError::InvalidFee { fee_type } => {
                    assert_eq!(fee_type, "bid");
                }
                error => panic!("unexpected error: {:?}", error),
            },
        };
    }

    #[test]
    fn create_ask_for_coin_with_valid_data_no_fee() {
        test_valid_coin_ask(None);
    }

    #[test]
    fn create_ask_for_coin_with_valid_data_and_fee() {
        test_valid_coin_ask(Some(500));
    }

    #[test]
    fn test_create_ask_for_scope_with_valid_data_no_fee() {
        test_valid_scope_ask(None);
    }

    #[test]
    fn test_create_ask_for_scope_with_valid_data_and_fee() {
        test_valid_scope_ask(Some(1000000));
    }

    #[test]
    fn create_ask_with_invalid_data() {
        let mut deps = mock_dependencies(&[]);
        if let Err(error) = set_contract_info(
            &mut deps.storage,
            &ContractInfo::new(
                Addr::unchecked("contract_admin"),
                "contract_bind_name".into(),
                "contract_name".into(),
                None,
                None,
            ),
        ) {
            panic!("unexpected error: {:?}", error)
        }

        // create ask invalid data
        let create_ask_msg = ExecuteMsg::CreateAsk {
            id: "".into(),
            quote: vec![],
            scope_address: None,
        };

        // handle create ask
        let create_ask_response = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("asker", &[]),
            create_ask_msg,
        );

        // verify handle create ask response returns ContractError::MissingField { id }
        match create_ask_response {
            Ok(_) => panic!("expected error, but handle_create_ask_response ok"),
            Err(error) => match error {
                ContractError::MissingField { field } => {
                    assert_eq!(field, "id")
                }
                error => panic!("unexpected error: {:?}", error),
            },
        }

        // create ask missing id
        let create_ask_msg = ExecuteMsg::CreateAsk {
            id: "".into(),
            quote: coins(100, "quote_1"),
            scope_address: None,
        };

        // handle create ask
        let create_ask_response = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("asker", &coins(100, "base_1")),
            create_ask_msg,
        );

        // verify execute create ask response returns ContractError::MissingField { id }
        match create_ask_response {
            Ok(_) => panic!("expected error, but execute_create_ask_response ok"),
            Err(error) => match error {
                ContractError::MissingField { field } => {
                    assert_eq!(field, "id")
                }
                error => panic!("unexpected error: {:?}", error),
            },
        }

        // create ask missing quote
        let create_ask_msg = ExecuteMsg::CreateAsk {
            id: "id".into(),
            quote: vec![],
            scope_address: None,
        };

        // execute create ask
        let create_ask_response = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("asker", &coins(100, "base_1")),
            create_ask_msg,
        );

        // verify execute create ask response returns ContractError::MissingField { quote }
        match create_ask_response {
            Ok(_) => panic!("expected error, but execute_create_ask_response ok"),
            Err(error) => match error {
                ContractError::MissingField { field } => {
                    assert_eq!(field, "quote")
                }
                error => panic!("unexpected error: {:?}", error),
            },
        }

        // create ask missing base
        let create_ask_msg = ExecuteMsg::CreateAsk {
            id: "id".into(),
            quote: coins(100, "quote_1"),
            scope_address: None,
        };

        // execute create ask
        let create_ask_response = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("asker", &[]),
            create_ask_msg,
        );

        // verify execute create ask response returns ContractError::AskMissingBase
        match create_ask_response {
            Ok(_) => panic!("expected error, but execute_create_ask_response ok"),
            Err(error) => match error {
                ContractError::MissingAskBase {} => {}
                error => panic!("unexpected error: {:?}", error),
            },
        };

        // create ask with scope and funds provided
        let create_ask_msg = ExecuteMsg::CreateAsk {
            id: "id".into(),
            quote: coins(100, "quote_1"),
            scope_address: Some("scope-address".to_string()),
        };

        let create_ask_response = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("asker", &coins(150, "fakecoin")),
            create_ask_msg,
        );

        match create_ask_response {
            Ok(_) => panic!("expected error, but execute_create_ask_response ok"),
            Err(error) => match error {
                ContractError::ScopeAskBaseWithFunds => {}
                error => panic!("unexpected error: {:?}", error),
            },
        };

        // create ask with scope provided with incorrect value owner address
        let create_ask_msg = ExecuteMsg::CreateAsk {
            id: "id".into(),
            quote: coins(100, "quote_1"),
            scope_address: Some("scope_address".to_string()),
        };

        deps.querier.with_scope(Scope {
            scope_id: "scope_address".to_string(),
            specification_id: "spec_address".to_string(),
            owners: vec![Party {
                address: Addr::unchecked(MOCK_CONTRACT_ADDR),
                role: PartyType::Owner,
            }],
            data_access: vec![],
            value_owner_address: Addr::unchecked("not_contract_address"),
        });

        let create_ask_response = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("asker", &[]),
            create_ask_msg.clone(),
        );

        match create_ask_response {
            Ok(_) => panic!("expected error, but execute_create_ask_response ok"),
            Err(error) => match error {
                ContractError::InvalidScopeOwner {
                    scope_address,
                    explanation,
                } => {
                    assert_eq!(
                        "scope_address", scope_address,
                        "the proper scope address should be found",
                    );
                    assert_eq!(
                        "the scope's value owner was expected to be [cosmos2contract], not [not_contract_address]", explanation,
                        "the proper explanation must be used in the InvalidScopeOwner error",
                    );
                }
                error => panic!("unexpected error: {:?}", error),
            },
        };

        // create ask with scope provided with multiple owners specified - re-using previous ask msg
        deps.querier.with_scope(Scope {
            scope_id: "scope_address".to_string(),
            specification_id: "spec_address".to_string(),
            owners: vec![
                Party {
                    address: Addr::unchecked("asker"),
                    role: PartyType::Owner,
                },
                Party {
                    address: Addr::unchecked("other-guy"),
                    role: PartyType::Owner,
                },
            ],
            data_access: vec![],
            value_owner_address: Addr::unchecked(MOCK_CONTRACT_ADDR),
        });

        let create_ask_response = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("asker", &[]),
            create_ask_msg.clone(),
        );

        match create_ask_response {
            Ok(_) => panic!("expected error, but execute_create_ask_response ok"),
            Err(error) => match error {
                ContractError::InvalidScopeOwner {
                    scope_address,
                    explanation,
                } => {
                    assert_eq!(
                        "scope_address", scope_address,
                        "the proper scope address should be found",
                    );
                    assert_eq!(
                        "the scope should only include a single owner, but found: 2", explanation,
                        "the proper explanation must be used in the InvalidScopeOwner error",
                    );
                }
                error => panic!("unexpected error: {:?}", error),
            },
        };

        // create ask with scope provided with incorrect contract owner specified - re-using previous ask msg
        deps.querier.with_scope(Scope {
            scope_id: "scope_address".to_string(),
            specification_id: "spec_address".to_string(),
            owners: vec![Party {
                address: Addr::unchecked("not-contract-address"),
                role: PartyType::Owner,
            }],
            data_access: vec![],
            value_owner_address: Addr::unchecked(MOCK_CONTRACT_ADDR),
        });

        let create_ask_response = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("asker", &[]),
            create_ask_msg,
        );

        match create_ask_response {
            Ok(_) => panic!("expected error, but execute_create_ask_response ok"),
            Err(error) => match error {
                ContractError::InvalidScopeOwner {
                    scope_address,
                    explanation,
                } => {
                    assert_eq!(
                        "scope_address", scope_address,
                        "the proper scope address should be found",
                    );
                    assert_eq!(
                        "the scope owner was expected to be [cosmos2contract], not [not-contract-address]", explanation,
                        "the proper explanation must be used in the InvalidScopeOwner error",
                    );
                }
                error => panic!("unexpected error: {:?}", error),
            },
        };
    }

    #[test]
    fn test_create_coin_bid_with_valid_data_no_fee() {
        test_valid_coin_bid(None);
    }

    #[test]
    fn test_create_coin_bid_with_valid_data_and_fee() {
        test_valid_coin_bid(Some(12345));
    }

    #[test]
    fn create_bid_with_invalid_data() {
        let mut deps = mock_dependencies(&[]);
        if let Err(error) = set_contract_info(
            &mut deps.storage,
            &ContractInfo::new(
                Addr::unchecked("contract_admin"),
                "contract_bind_name".into(),
                "contract_name".into(),
                None,
                None,
            ),
        ) {
            panic!("unexpected error: {:?}", error)
        }

        // create bid missing id
        let create_bid_msg = ExecuteMsg::CreateBid {
            id: "".into(),
            base: BaseType::coin(100, "base_1"),
            effective_time: Some(Timestamp::default()),
        };

        // execute create bid
        let create_bid_response = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("bidder", &coins(100, "quote_1")),
            create_bid_msg,
        );

        // verify execute create bid response returns ContractError::MissingField { id }
        match create_bid_response {
            Ok(_) => panic!("expected error, but create_bid_response ok"),
            Err(error) => match error {
                ContractError::MissingField { field } => {
                    assert_eq!(field, "id")
                }
                error => panic!("unexpected error: {:?}", error),
            },
        }

        // create bid missing base
        let create_bid_msg = ExecuteMsg::CreateBid {
            id: "id".into(),
            base: BaseType::coins(vec![]),
            effective_time: Some(Timestamp::default()),
        };

        // execute create bid
        let create_bid_response = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("bidder", &coins(100, "quote_1")),
            create_bid_msg,
        );

        // verify execute create bid response returns ContractError::MissingField { base }
        match create_bid_response {
            Ok(_) => panic!("expected error, but create_bid_response ok"),
            Err(error) => match error {
                ContractError::MissingField { field } => {
                    assert_eq!(field, "base")
                }
                error => panic!("unexpected error: {:?}", error),
            },
        }

        // create bid missing quote
        let create_bid_msg = ExecuteMsg::CreateBid {
            id: "id".into(),
            base: BaseType::coin(100, "base_1"),
            effective_time: Some(Timestamp::default()),
        };

        // execute create bid
        let create_bid_response = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("bidder", &[]),
            create_bid_msg,
        );

        // verify execute create bid response returns ContractError::BidMissingQuote
        match create_bid_response {
            Ok(_) => panic!("expected error, but create_bid_response ok"),
            Err(error) => match error {
                ContractError::MissingBidQuote {} => {}
                error => panic!("unexpected error: {:?}", error),
            },
        }
    }

    #[test]
    fn test_create_scope_bid_with_valid_data_no_fee() {
        test_valid_scope_bid(None);
    }

    #[test]
    fn test_create_scope_bid_with_valid_data_and_fee() {
        test_valid_scope_bid(Some(3434234));
    }

    #[test]
    fn cancel_coin_with_valid_data() {
        let mut deps = mock_dependencies(&[]);
        if let Err(error) = set_contract_info(
            &mut deps.storage,
            &ContractInfo::new(
                Addr::unchecked("contract_admin"),
                "contract_bind_name".into(),
                "contract_name".into(),
                None,
                None,
            ),
        ) {
            panic!("unexpected error: {:?}", error)
        }

        // create ask data
        let asker_info = mock_info("asker", &coins(200, "base_1"));

        let create_ask_msg = ExecuteMsg::CreateAsk {
            id: "ask_id".into(),
            quote: coins(100, "quote_1"),
            scope_address: None,
        };

        // execute create ask
        if let Err(error) = execute(deps.as_mut(), mock_env(), asker_info, create_ask_msg) {
            panic!("unexpected error: {:?}", error)
        }

        // verify ask order stored
        let ask_storage = get_ask_storage_read_v2(&deps.storage);
        assert!(ask_storage.load("ask_id".to_string().as_bytes()).is_ok());

        // cancel ask order
        let asker_info = mock_info("asker", &[]);

        let cancel_ask_msg = ExecuteMsg::CancelAsk {
            id: "ask_id".to_string(),
        };
        let cancel_ask_response = execute(
            deps.as_mut(),
            mock_env(),
            asker_info.clone(),
            cancel_ask_msg,
        );

        match cancel_ask_response {
            Ok(cancel_ask_response) => {
                assert_eq!(cancel_ask_response.attributes.len(), 1);
                assert_eq!(
                    cancel_ask_response.attributes[0],
                    attr("action", "cancel_ask")
                );
                assert_eq!(cancel_ask_response.messages.len(), 1);
                assert_eq!(
                    cancel_ask_response.messages[0].msg,
                    CosmosMsg::Bank(BankMsg::Send {
                        to_address: asker_info.sender.to_string(),
                        amount: coins(200, "base_1"),
                    })
                );
            }
            Err(error) => panic!("unexpected error: {:?}", error),
        }

        // verify ask order removed from storage
        let ask_storage = get_ask_storage_read_v2(&deps.storage);
        assert!(ask_storage.load("ask_id".to_string().as_bytes()).is_err());

        // create bid data
        let bidder_info = mock_info("bidder", &coins(100, "quote_1"));
        let create_bid_msg = ExecuteMsg::CreateBid {
            id: "bid_id".into(),
            base: BaseType::coins(vec![Coin {
                denom: "base_1".into(),
                amount: Uint128::new(200),
            }]),
            effective_time: Some(Timestamp::default()),
        };

        // execute create bid
        if let Err(error) = execute(deps.as_mut(), mock_env(), bidder_info, create_bid_msg) {
            panic!("unexpected error: {:?}", error)
        }

        // verify bid order stored
        let bid_storage = get_bid_storage_read_v2(&deps.storage);
        assert!(bid_storage.load("bid_id".to_string().as_bytes()).is_ok(),);

        // cancel bid order
        let bidder_info = mock_info("bidder", &[]);

        let cancel_bid_msg = ExecuteMsg::CancelBid {
            id: "bid_id".to_string(),
        };

        let cancel_bid_response = execute(
            deps.as_mut(),
            mock_env(),
            bidder_info.clone(),
            cancel_bid_msg,
        );

        match cancel_bid_response {
            Ok(cancel_bid_response) => {
                assert_eq!(cancel_bid_response.attributes.len(), 1);
                assert_eq!(
                    cancel_bid_response.attributes[0],
                    attr("action", "cancel_bid")
                );
                assert_eq!(cancel_bid_response.messages.len(), 1);
                assert_eq!(
                    cancel_bid_response.messages[0].msg,
                    CosmosMsg::Bank(BankMsg::Send {
                        to_address: bidder_info.sender.to_string(),
                        amount: coins(100, "quote_1"),
                    })
                );
            }
            Err(error) => panic!("unexpected error: {:?}", error),
        }

        // verify bid order removed from storage
        let bid_storage = get_bid_storage_read_v2(&deps.storage);
        assert!(bid_storage.load("bid_id".to_string().as_bytes()).is_err());
    }

    #[test]
    fn cancel_scope_with_valid_data() {
        let mut deps = mock_dependencies(&[]);
        if let Err(error) = set_contract_info(
            &mut deps.storage,
            &ContractInfo::new(
                Addr::unchecked("contract_admin"),
                "contract_bind_name".into(),
                "contract_name".into(),
                None,
                None,
            ),
        ) {
            panic!("unexpected error: {:?}", error)
        }

        // create ask data - omit funds because a scope is being provided
        let asker_info = mock_info("asker", &[]);

        let create_ask_msg = ExecuteMsg::CreateAsk {
            id: "ask_id".into(),
            quote: coins(100, "quote_1"),
            scope_address: Some("scope_address".to_string()),
        };

        deps.querier.with_scope(Scope {
            scope_id: "scope_address".to_string(),
            specification_id: "spec_address".to_string(),
            owners: vec![Party {
                address: Addr::unchecked(MOCK_CONTRACT_ADDR),
                role: PartyType::Owner,
            }],
            data_access: vec![],
            value_owner_address: Addr::unchecked(MOCK_CONTRACT_ADDR),
        });

        // execute create ask
        if let Err(error) = execute(deps.as_mut(), mock_env(), asker_info, create_ask_msg) {
            panic!("unexpected error: {:?}", error)
        }

        // verify ask order stored
        let ask_storage = get_ask_storage_read_v2(&deps.storage);
        assert!(ask_storage.load("ask_id".to_string().as_bytes()).is_ok());

        // cancel ask order
        let asker_info = mock_info("asker", &[]);

        let cancel_ask_msg = ExecuteMsg::CancelAsk {
            id: "ask_id".to_string(),
        };
        let cancel_ask_response = execute(
            deps.as_mut(),
            mock_env(),
            asker_info.clone(),
            cancel_ask_msg,
        );

        match cancel_ask_response {
            Ok(cancel_ask_response) => {
                assert_eq!(cancel_ask_response.attributes.len(), 1);
                assert_eq!(
                    cancel_ask_response.attributes[0],
                    attr("action", "cancel_ask")
                );
                assert_eq!(cancel_ask_response.messages.len(), 1);
                match &cancel_ask_response.messages.first().unwrap().msg {
                    CosmosMsg::Custom(ProvenanceMsg {
                        params:
                            ProvenanceMsgParams::Metadata(MetadataMsgParams::WriteScope {
                                scope,
                                signers,
                            }),
                        ..
                    }) => {
                        assert_eq!(
                            1,
                            scope.owners.len(),
                            "expected the scope to only include one owner after the owner is swapped back to the original value",
                        );
                        let scope_owner = scope.owners.first().unwrap();
                        assert_eq!(
                            "asker",
                            scope_owner.address.as_str(),
                            "expected the asker address to be set as the scope owner",
                        );
                        assert_eq!(
                            PartyType::Owner,
                            scope_owner.role,
                            "expected the asker's role to be that of owner",
                        );
                        assert_eq!(
                            "asker",
                            scope.value_owner_address.as_str(),
                            "expected the asker to be set as the value owner after a cancellation",
                        );
                        assert_eq!(
                            1,
                            signers.len(),
                            "expected only a single signer to be used on the write scope request",
                        );
                        assert_eq!(
                            MOCK_CONTRACT_ADDR,
                            signers.first().unwrap().as_str(),
                            "expected the signer for the write scope request to be the contract",
                        );
                    }
                    msg => panic!("unexpected message emitted by cancel ask: {:?}", msg),
                };
            }
            Err(error) => panic!("unexpected error: {:?}", error),
        }

        // verify ask order removed from storage
        let ask_storage = get_ask_storage_read_v2(&deps.storage);
        assert!(ask_storage.load("ask_id".to_string().as_bytes()).is_err());

        // create bid data
        let bidder_info = mock_info("bidder", &coins(100, "quote_1"));
        let create_bid_msg = ExecuteMsg::CreateBid {
            id: "bid_id".into(),
            base: BaseType::scope("scope_address"),
            effective_time: Some(Timestamp::default()),
        };

        // execute create bid
        if let Err(error) = execute(deps.as_mut(), mock_env(), bidder_info, create_bid_msg) {
            panic!("unexpected error: {:?}", error)
        }

        // verify bid order stored
        let bid_storage = get_bid_storage_read_v2(&deps.storage);
        assert!(bid_storage.load("bid_id".to_string().as_bytes()).is_ok(),);

        // cancel bid order
        let bidder_info = mock_info("bidder", &[]);

        let cancel_bid_msg = ExecuteMsg::CancelBid {
            id: "bid_id".to_string(),
        };

        let cancel_bid_response = execute(
            deps.as_mut(),
            mock_env(),
            bidder_info.clone(),
            cancel_bid_msg,
        );

        match cancel_bid_response {
            Ok(cancel_bid_response) => {
                assert_eq!(cancel_bid_response.attributes.len(), 1);
                assert_eq!(
                    cancel_bid_response.attributes[0],
                    attr("action", "cancel_bid")
                );
                assert_eq!(cancel_bid_response.messages.len(), 1);
                assert_eq!(
                    cancel_bid_response.messages[0].msg,
                    CosmosMsg::Bank(BankMsg::Send {
                        to_address: bidder_info.sender.to_string(),
                        amount: coins(100, "quote_1"),
                    })
                );
            }
            Err(error) => panic!("unexpected error: {:?}", error),
        }

        // verify bid order removed from storage
        let bid_storage = get_bid_storage_read_v2(&deps.storage);
        assert!(bid_storage.load("bid_id".to_string().as_bytes()).is_err());
    }

    #[test]
    fn cancel_with_invalid_data() {
        let mut deps = mock_dependencies(&[]);
        if let Err(error) = set_contract_info(
            &mut deps.storage,
            &ContractInfo::new(
                Addr::unchecked("contract_admin"),
                "contract_bind_name".into(),
                "contract_name".into(),
                None,
                None,
            ),
        ) {
            panic!("unexpected error: {:?}", error)
        }

        let asker_info = mock_info("asker", &[]);

        // cancel ask order with missing id returns ContractError::Unauthorized
        let cancel_ask_msg = ExecuteMsg::CancelAsk { id: "".to_string() };
        let cancel_response = execute(
            deps.as_mut(),
            mock_env(),
            asker_info.clone(),
            cancel_ask_msg,
        );

        match cancel_response {
            Err(error) => match error {
                ContractError::Unauthorized {} => {}
                _ => {
                    panic!("unexpected error: {:?}", error)
                }
            },
            Ok(_) => panic!("expected error, but cancel_response ok"),
        }

        // cancel non-existent ask order returns ContractError::Unauthorized
        let cancel_ask_msg = ExecuteMsg::CancelAsk {
            id: "unknown_id".to_string(),
        };

        let cancel_response = execute(
            deps.as_mut(),
            mock_env(),
            asker_info.clone(),
            cancel_ask_msg,
        );

        match cancel_response {
            Err(error) => match error {
                ContractError::Unauthorized {} => {}
                _ => {
                    panic!("unexpected error: {:?}", error)
                }
            },
            Ok(_) => panic!("expected error, but cancel_response ok"),
        }

        // cancel ask order with sender not equal to owner returns ContractError::Unauthorized
        if let Err(error) = get_ask_storage_v2(&mut deps.storage).save(
            "ask_id".to_string().as_bytes(),
            &AskOrderV2 {
                base: BaseType::coin(200, "base_1"),
                id: "ask_id".into(),
                owner: Addr::unchecked(""),
                quote: coins(100, "quote_1"),
            },
        ) {
            panic!("unexpected error: {:?}", error)
        };
        let cancel_ask_msg = ExecuteMsg::CancelAsk {
            id: "ask_id".to_string(),
        };

        let cancel_response = execute(deps.as_mut(), mock_env(), asker_info, cancel_ask_msg);

        match cancel_response {
            Err(error) => match error {
                ContractError::Unauthorized {} => {}
                _ => {
                    panic!("unexpected error: {:?}", error)
                }
            },
            Ok(_) => panic!("expected error, but cancel_response ok"),
        }

        // cancel ask order with sent_funds returns ContractError::CancelWithFunds
        let asker_info = mock_info("asker", &coins(1, "sent_coin"));
        let cancel_ask_msg = ExecuteMsg::CancelAsk {
            id: "ask_id".to_string(),
        };

        let cancel_response = execute(deps.as_mut(), mock_env(), asker_info, cancel_ask_msg);

        match cancel_response {
            Err(error) => match error {
                ContractError::CancelWithFunds {} => {}
                _ => {
                    panic!("unexpected error: {:?}", error)
                }
            },
            Ok(_) => panic!("expected error, but cancel_response ok"),
        }
    }

    #[test]
    fn execute_match_with_valid_coin_data() {
        // setup
        let mut deps = mock_dependencies(&[]);
        if let Err(error) = set_contract_info(
            &mut deps.storage,
            &ContractInfo::new(
                Addr::unchecked("contract_admin"),
                "contract_bind_name".into(),
                "contract_name".into(),
                None,
                None,
            ),
        ) {
            panic!("unexpected error: {:?}", error)
        }

        // store valid ask order
        let ask_order = AskOrderV2 {
            base: BaseType::coins(vec![coin(100, "base_1"), coin(200, "base_2")]),
            id: "ask_id".into(),
            owner: Addr::unchecked("asker"),
            quote: coins(200, "quote_1"),
        };

        let mut ask_storage = get_ask_storage_v2(&mut deps.storage);
        if let Err(error) = ask_storage.save(ask_order.id.as_bytes(), &ask_order) {
            panic!("unexpected error: {:?}", error)
        };

        // store valid bid order
        let bid_order = BidOrderV2 {
            base: BaseType::coins(vec![coin(200, "base_2"), coin(100, "base_1")]),
            effective_time: Some(Timestamp::default()),
            id: "bid_id".to_string(),
            owner: Addr::unchecked("bidder"),
            quote: coins(200, "quote_1"),
        };

        let mut bid_storage = get_bid_storage_v2(&mut deps.storage);
        if let Err(error) = bid_storage.save(bid_order.id.as_bytes(), &bid_order) {
            panic!("unexpected error: {:?}", error);
        };

        // execute on matched ask order and bid order
        let execute_msg = ExecuteMsg::ExecuteMatch {
            ask_id: ask_order.id,
            bid_id: bid_order.id.clone(),
        };

        let execute_response = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("contract_admin", &[]),
            execute_msg,
        );

        // validate execute response
        match execute_response {
            Err(error) => panic!("unexpected error: {:?}", error),
            Ok(execute_response) => {
                assert_eq!(execute_response.attributes.len(), 1);
                assert_eq!(execute_response.attributes[0], attr("action", "execute"));
                assert_eq!(execute_response.messages.len(), 2);
                assert_eq!(
                    execute_response.messages[0].msg,
                    CosmosMsg::Bank(BankMsg::Send {
                        to_address: ask_order.owner.to_string(),
                        amount: ask_order.quote,
                    })
                );
                handle_expected_coin(&bid_order.base, |coins| {
                    assert_eq!(
                        execute_response.messages[1].msg,
                        CosmosMsg::Bank(BankMsg::Send {
                            to_address: bid_order.owner.to_string(),
                            amount: coins.to_vec(),
                        })
                    );
                });
            }
        }
    }

    #[test]
    fn execute_match_with_valid_scope_data() {
        // setup
        let mut deps = mock_dependencies(&[]);

        let scope_input = Scope {
            scope_id: "scope1234".to_string(),
            specification_id: "scopespec1".to_string(),
            owners: vec![Party {
                address: Addr::unchecked("asker"),
                role: PartyType::Owner,
            }],
            data_access: vec![],
            value_owner_address: Addr::unchecked("asker"), // todo: does this need to be the contract's address?
        };
        deps.querier.with_scope(scope_input.clone());

        if let Err(error) = set_contract_info(
            &mut deps.storage,
            &ContractInfo::new(
                Addr::unchecked("contract_admin"),
                "contract_bind_name".into(),
                "contract_name".into(),
                None,
                None,
            ),
        ) {
            panic!("unexpected error: {:?}", error)
        }

        // store valid ask order
        let ask_order = AskOrderV2 {
            base: BaseType::scope(&scope_input.scope_id),
            id: "ask_id".into(),
            owner: Addr::unchecked("asker"),
            quote: coins(200, "quote_1"),
        };

        let mut ask_storage = get_ask_storage_v2(&mut deps.storage);
        if let Err(error) = ask_storage.save(ask_order.id.as_bytes(), &ask_order) {
            panic!("unexpected error: {:?}", error)
        };

        // store valid bid order
        let bid_order = BidOrderV2 {
            base: BaseType::scope(&scope_input.scope_id),
            effective_time: Some(Timestamp::default()),
            id: "bid_id".to_string(),
            owner: Addr::unchecked("bidder"),
            quote: coins(200, "quote_1"),
        };

        let mut bid_storage = get_bid_storage_v2(&mut deps.storage);
        if let Err(error) = bid_storage.save(bid_order.id.as_bytes(), &bid_order) {
            panic!("unexpected error: {:?}", error);
        };

        // execute on matched ask order and bid order
        let execute_msg = ExecuteMsg::ExecuteMatch {
            ask_id: ask_order.id,
            bid_id: bid_order.id.clone(),
        };

        let execute_response = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("contract_admin", &[]),
            execute_msg,
        );

        // validate execute response
        match execute_response {
            Err(error) => panic!("unexpected error: {:?}", error),
            Ok(execute_response) => {
                assert_eq!(execute_response.attributes.len(), 1);
                assert_eq!(execute_response.attributes[0], attr("action", "execute"));
                assert_eq!(execute_response.messages.len(), 2);
                assert_eq!(
                    execute_response.messages[0].msg,
                    CosmosMsg::Bank(BankMsg::Send {
                        to_address: ask_order.owner.to_string(),
                        amount: ask_order.quote,
                    })
                );
                handle_expected_scope(&bid_order.base, |scope_id| {
                    if let CosmosMsg::Custom(ProvenanceMsg { params, .. }) =
                        &execute_response.messages[1].msg
                    {
                        assert_eq!(
                            params.to_owned(),
                            ProvenanceMsgParams::Metadata(MetadataMsgParams::WriteScope {
                                scope: Scope {
                                    scope_id: scope_id.to_string(),
                                    specification_id: scope_input.specification_id,
                                    owners: vec![Party {
                                        address: bid_order.owner.clone(),
                                        role: PartyType::Owner
                                    }],
                                    data_access: scope_input.data_access,
                                    value_owner_address: bid_order.owner.clone()
                                },
                                signers: vec![Addr::unchecked(MOCK_CONTRACT_ADDR)]
                            }),
                        );
                    } else {
                        panic!("Unexpected second message type for match, expected WriteScope, received {:?}", execute_response.messages[1].msg)
                    }
                });
            }
        }
    }

    #[test]
    fn execute_match_with_invalid_coin_data() {
        // setup
        let mut deps = mock_dependencies(&[]);
        if let Err(error) = set_contract_info(
            &mut deps.storage,
            &ContractInfo::new(
                Addr::unchecked("contract_admin"),
                "contract_bind_name".into(),
                "contract_name".into(),
                None,
                None,
            ),
        ) {
            panic!("unexpected error: {:?}", error)
        }

        // store valid ask order
        let ask_order = AskOrderV2 {
            base: BaseType::coin(200, "base_1"),
            id: "ask_id".into(),
            owner: Addr::unchecked("asker"),
            quote: coins(100, "quote_1"),
        };

        let mut ask_storage = get_ask_storage_v2(&mut deps.storage);
        if let Err(error) = ask_storage.save(ask_order.id.as_bytes(), &ask_order) {
            panic!("unexpected error: {:?}", error)
        };

        // store valid bid order
        let bid_order = BidOrderV2 {
            base: BaseType::coin(100, "base_1"),
            effective_time: Some(Timestamp::default()),
            id: "bid_id".into(),
            owner: Addr::unchecked("bidder"),
            quote: coins(100, "quote_1"),
        };

        let mut bid_storage = get_bid_storage_v2(&mut deps.storage);
        if let Err(error) = bid_storage.save(bid_order.id.as_bytes(), &bid_order) {
            panic!("unexpected error: {:?}", error);
        };

        // execute by non-admin ContractError::Unauthorized
        let execute_msg = ExecuteMsg::ExecuteMatch {
            ask_id: "ask_id".into(),
            bid_id: "bid_id".into(),
        };

        let execute_response = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("user", &[]),
            execute_msg,
        );

        match execute_response {
            Err(ContractError::Unauthorized {}) => {}
            Err(error) => panic!("unexpected error: {:?}", error),
            Ok(_) => panic!("expected error, but execute_response ok"),
        }

        // execute on mismatched ask order and bid order returns ContractError::AskBidMismatch
        let execute_msg = ExecuteMsg::ExecuteMatch {
            ask_id: "ask_id".into(),
            bid_id: "bid_id".into(),
        };

        let execute_response = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("contract_admin", &[]),
            execute_msg,
        );

        match execute_response {
            Err(ContractError::AskBidMismatch {}) => {}
            Err(error) => panic!("unexpected error: {:?}", error),
            Ok(_) => panic!("expected error, but execute_response ok"),
        }

        // execute on non-existent ask order and bid order returns ContractError::AskBidMismatch
        let execute_msg = ExecuteMsg::ExecuteMatch {
            ask_id: "no_ask_id".into(),
            bid_id: "bid_id".into(),
        };

        let execute_response = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("contract_admin", &[]),
            execute_msg,
        );

        match execute_response {
            Err(ContractError::AskBidMismatch {}) => {}
            Err(error) => panic!("unexpected error: {:?}", error),
            Ok(_) => panic!("expected error, but execute_response ok"),
        }

        // execute on non-existent ask order and bid order returns ContractError::AskBidMismatch
        let execute_msg = ExecuteMsg::ExecuteMatch {
            ask_id: "ask_id".into(),
            bid_id: "no_bid_id".into(),
        };

        let execute_response = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("contract_admin", &[]),
            execute_msg,
        );

        match execute_response {
            Err(ContractError::AskBidMismatch {}) => {}
            Err(error) => panic!("unexpected error: {:?}", error),
            Ok(_) => panic!("expected error, but execute_response ok"),
        }

        // execute with sent_funds returns ContractError::ExecuteWithFunds
        let execute_msg = ExecuteMsg::ExecuteMatch {
            ask_id: "ask_id".into(),
            bid_id: "bid_id".into(),
        };

        let execute_response = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("contract_admin", &coins(100, "funds")),
            execute_msg,
        );

        match execute_response {
            Err(ContractError::ExecuteWithFunds {}) => {}
            Err(error) => panic!("unexpected error: {:?}", error),
            Ok(_) => panic!("expected error, but execute_response ok"),
        }
    }

    #[test]
    fn execute_match_with_invalid_scope_data() {
        // setup
        let mut deps = mock_dependencies(&[]);

        let scope_input = Scope {
            scope_id: "scope1234".to_string(),
            specification_id: "scopespec1".to_string(),
            owners: vec![Party {
                address: Addr::unchecked("asker"),
                role: PartyType::Owner,
            }],
            data_access: vec![],
            value_owner_address: Addr::unchecked("asker"), // todo: does this need to be the contract's address?
        };
        deps.querier.with_scope(scope_input.clone());

        if let Err(error) = set_contract_info(
            &mut deps.storage,
            &ContractInfo::new(
                Addr::unchecked("contract_admin"),
                "contract_bind_name".into(),
                "contract_name".into(),
                None,
                None,
            ),
        ) {
            panic!("unexpected error: {:?}", error)
        }

        // store valid ask order
        let ask_order = AskOrderV2 {
            base: BaseType::scope(scope_input.scope_id),
            id: "ask_id".into(),
            owner: Addr::unchecked("asker"),
            quote: coins(100, "quote_1"),
        };

        let mut ask_storage = get_ask_storage_v2(&mut deps.storage);
        if let Err(error) = ask_storage.save(ask_order.id.as_bytes(), &ask_order) {
            panic!("unexpected error: {:?}", error)
        };

        // store invalid bid order
        let bid_order = BidOrderV2 {
            base: BaseType::coin(100, "base_1"),
            effective_time: Some(Timestamp::default()),
            id: "bid_id".into(),
            owner: Addr::unchecked("bidder"),
            quote: coins(100, "quote_1"),
        };

        let mut bid_storage = get_bid_storage_v2(&mut deps.storage);
        if let Err(error) = bid_storage.save(bid_order.id.as_bytes(), &bid_order) {
            panic!("unexpected error: {:?}", error);
        };

        // execute by non-admin ContractError::Unauthorized
        let execute_msg = ExecuteMsg::ExecuteMatch {
            ask_id: "ask_id".into(),
            bid_id: "bid_id".into(),
        };

        let execute_response = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("user", &[]),
            execute_msg,
        );

        match execute_response {
            Err(ContractError::Unauthorized {}) => {}
            Err(error) => panic!("unexpected error: {:?}", error),
            Ok(_) => panic!("expected error, but execute_response ok"),
        }

        // execute on mismatched ask order and bid order returns ContractError::AskBidMismatch
        let execute_msg = ExecuteMsg::ExecuteMatch {
            ask_id: "ask_id".into(),
            bid_id: "bid_id".into(),
        };

        let execute_response = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("contract_admin", &[]),
            execute_msg,
        );

        match execute_response {
            Err(ContractError::AskBidMismatch {}) => {}
            Err(error) => panic!("unexpected error: {:?}", error),
            Ok(_) => panic!("expected error, but execute_response ok"),
        }

        // execute on non-existent ask order and bid order returns ContractError::AskBidMismatch
        let execute_msg = ExecuteMsg::ExecuteMatch {
            ask_id: "no_ask_id".into(),
            bid_id: "bid_id".into(),
        };

        let execute_response = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("contract_admin", &[]),
            execute_msg,
        );

        match execute_response {
            Err(ContractError::AskBidMismatch {}) => {}
            Err(error) => panic!("unexpected error: {:?}", error),
            Ok(_) => panic!("expected error, but execute_response ok"),
        }

        // execute on non-existent ask order and bid order returns ContractError::AskBidMismatch
        let execute_msg = ExecuteMsg::ExecuteMatch {
            ask_id: "ask_id".into(),
            bid_id: "no_bid_id".into(),
        };

        let execute_response = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("contract_admin", &[]),
            execute_msg,
        );

        match execute_response {
            Err(ContractError::AskBidMismatch {}) => {}
            Err(error) => panic!("unexpected error: {:?}", error),
            Ok(_) => panic!("expected error, but execute_response ok"),
        }

        // execute with sent_funds returns ContractError::ExecuteWithFunds
        let execute_msg = ExecuteMsg::ExecuteMatch {
            ask_id: "ask_id".into(),
            bid_id: "bid_id".into(),
        };

        let execute_response = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("contract_admin", &coins(100, "funds")),
            execute_msg,
        );

        match execute_response {
            Err(ContractError::ExecuteWithFunds {}) => {}
            Err(error) => panic!("unexpected error: {:?}", error),
            Ok(_) => panic!("expected error, but execute_response ok"),
        }
    }

    #[test]
    pub fn query_with_valid_data() {
        // setup
        let mut deps = mock_dependencies(&[]);
        if let Err(error) = set_contract_info(
            &mut deps.storage,
            &ContractInfo::new(
                Addr::unchecked("contract_admin"),
                "contract_bind_name".into(),
                "contract_name".into(),
                None,
                None,
            ),
        ) {
            panic!("unexpected error: {:?}", error)
        }

        // store valid ask order
        let ask_order = AskOrderV2 {
            base: BaseType::coin(200, "base_1"),
            id: "ask_id".into(),
            owner: Addr::unchecked("asker"),
            quote: coins(100, "quote_1"),
        };

        let mut ask_storage = get_ask_storage_v2(&mut deps.storage);
        if let Err(error) = ask_storage.save(ask_order.id.as_bytes(), &ask_order) {
            panic!("unexpected error: {:?}", error)
        };

        // store valid bid order
        let bid_order = BidOrderV2 {
            base: BaseType::coin(100, "base_1"),
            effective_time: Some(Timestamp::default()),
            id: "bid_id".into(),
            owner: Addr::unchecked("bidder"),
            quote: coins(100, "quote_1"),
        };

        let mut bid_storage = get_bid_storage_v2(&mut deps.storage);
        if let Err(error) = bid_storage.save(bid_order.id.as_bytes(), &bid_order) {
            panic!("unexpected error: {:?}", error);
        };

        // query for contract_info
        let query_contract_info_response =
            query(deps.as_ref(), mock_env(), QueryMsg::GetContractInfo {});

        match query_contract_info_response {
            Ok(contract_info) => {
                assert_eq!(
                    contract_info,
                    to_binary(&get_contract_info(&deps.storage).unwrap()).unwrap()
                )
            }
            Err(error) => panic!("unexpected error: {:?}", error),
        }

        // query for ask order
        let query_ask_response = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetAsk {
                id: ask_order.id.clone(),
            },
        );

        assert_eq!(query_ask_response, to_binary(&ask_order));

        // query for bid order
        let query_bid_response = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetBid {
                id: bid_order.id.clone(),
            },
        );

        assert_eq!(query_bid_response, to_binary(&bid_order));
    }

    #[test]
    fn test_update_fees_with_valid_data() {
        let mut deps = mock_dependencies(&[]);
        instantiate(
            deps.as_mut(),
            mock_env(),
            mock_info("contract_admin", &[]),
            InstantiateMsg {
                bind_name: "examples.sc.pb".to_string(),
                contract_name: "contract_name".to_string(),
                ask_fee: None,
                bid_fee: None,
            },
        )
        .unwrap();
        let contract_info = get_contract_info(deps.as_ref().storage).unwrap();
        assert_eq!(None, contract_info.ask_fee);
        assert_eq!(None, contract_info.bid_fee);
        let response = update_fees(
            deps.as_mut(),
            mock_info("contract_admin", &[]),
            Some(Uint128::new(10)),
            Some(Uint128::new(15)),
        )
        .expect("updating fees should be successful");
        assert!(response.messages.is_empty());
        assert_eq!(3, response.attributes.len());
        assert_eq!(attr("action", "update_fees"), response.attributes[0]);
        assert_eq!(attr("new_ask_fee", "10nhash"), response.attributes[1]);
        assert_eq!(attr("new_bid_fee", "15nhash"), response.attributes[2]);
        let response = update_fees(deps.as_mut(), mock_info("contract_admin", &[]), None, None)
            .expect("clearing fees should be successful");
        assert!(response.messages.is_empty());
        assert_eq!(3, response.attributes.len());
        assert_eq!(attr("action", "update_fees"), response.attributes[0]);
        assert_eq!(attr("new_ask_fee", "cleared"), response.attributes[1]);
        assert_eq!(attr("new_bid_fee", "cleared"), response.attributes[2]);
    }

    #[test]
    fn test_update_fees_with_invalid_data() {
        let mut deps = mock_dependencies(&[]);
        let err = update_fees(deps.as_mut(), mock_info("contract_admin", &[]), None, None)
            .expect_err("an error should occur when no contract info exists");
        assert!(
            matches!(err, ContractError::Std(StdError::NotFound { .. })),
            "a not found error should occur when contract info does not exist, but got: {:?}",
            err,
        );
        instantiate(
            deps.as_mut(),
            mock_env(),
            mock_info("contract_admin", &[]),
            InstantiateMsg {
                bind_name: "examples.sc.pb".to_string(),
                contract_name: "contract_name".to_string(),
                ask_fee: None,
                bid_fee: None,
            },
        )
        .unwrap();
        let err = update_fees(deps.as_mut(), mock_info("not_admin", &[]), None, None)
            .expect_err("an error should occur when a non-admin attempts to update fees");
        assert!(
            matches!(err, ContractError::Unauthorized {}),
            "an unauthorized error should occur when a non-admin attempts to update fees, but got: {:?}",
            err,
        );
        let err = update_fees(
            deps.as_mut(),
            mock_info("contract_admin", &coins(1000, "nhash")),
            None,
            None,
        )
        .expect_err("an error should occur when the admin provides funds");
        assert!(
            matches!(err, ContractError::UpdateFeesWithFunds {}),
            "an update fees with funds error should occur, but got: {:?}",
            err,
        );
        let err = update_fees(
            deps.as_mut(),
            mock_info("contract_admin", &[]),
            Some(Uint128::zero()),
            None,
        )
        .expect_err("an error should occur when the ask fee is zero");
        match err {
            ContractError::InvalidFee { fee_type } => {
                assert_eq!("ask", fee_type);
            }
            e => panic!("unexpected error when ask fee is zero: {:?}", e),
        };
        let err = update_fees(
            deps.as_mut(),
            mock_info("contract_admin", &[]),
            None,
            Some(Uint128::zero()),
        )
        .expect_err("an error should occur when the bid fee is zero");
        match err {
            ContractError::InvalidFee { fee_type } => {
                assert_eq!("bid", fee_type);
            }
            e => panic!("unexpected error when the bid fee is zero: {:?}", e),
        };
    }

    fn handle_expected_coin<A: FnOnce(&Vec<Coin>) -> ()>(base_type: &BaseType, action: A) {
        match base_type {
            BaseType::Coin { coins } => action(coins),
            _ => panic!("Unexpected base type of scope"),
        }
    }

    fn handle_expected_scope<A: FnOnce(&String) -> ()>(base_type: &BaseType, action: A) {
        match base_type {
            BaseType::Scope { scope_address } => action(scope_address),
            _ => panic!("Unexpected base type of coin"),
        }
    }

    fn match_create_response(
        response: Result<Response<ProvenanceMsg>, ContractError>,
        expected_fee_type: &str,
        expected_action_attribute_value: &str,
        expected_fee: Option<u128>,
    ) {
        match response {
            Ok(response) => {
                assert_eq!(
                    response.attributes.len(),
                    1 + if expected_fee.is_some() { 1 } else { 0 }
                );
                assert_eq!(
                    response.attributes[0],
                    attr("action", expected_action_attribute_value)
                );
                if let Some(fee) = expected_fee {
                    assert_eq!(
                        response.attributes[1],
                        attr("fee_charged", format!("{}nhash", fee.to_string())),
                    );
                    assert_eq!(1, response.messages.len());
                    match &response.messages.first().unwrap().msg {
                        CosmosMsg::Custom(ProvenanceMsg {
                            params:
                                ProvenanceMsgParams::MsgFees(MsgFeesMsgParams::AssessCustomFee {
                                    amount,
                                    name,
                                    from,
                                    recipient,
                                }),
                            ..
                        }) => {
                            assert_eq!(fee, amount.amount.u128());
                            assert_eq!("nhash", amount.denom);
                            assert_eq!(
                                format!("{} creation fee", expected_fee_type),
                                name.to_owned().unwrap()
                            );
                            assert_eq!(MOCK_CONTRACT_ADDR, from.as_str());
                            assert_eq!("contract_admin", recipient.to_owned().unwrap().as_str());
                        }
                        msg => panic!("unexpected msg: {:?}", msg),
                    }
                } else {
                    assert!(response.messages.is_empty());
                }
            }
            Err(error) => {
                panic!("failed to create {}: {:?}", expected_fee_type, error)
            }
        }
    }

    fn test_valid_coin_ask(ask_fee: Option<u128>) {
        let mut deps = mock_dependencies(&[]);
        if let Err(error) = set_contract_info(
            &mut deps.storage,
            &ContractInfo::new(
                Addr::unchecked("contract_admin"),
                "contract_bind_name".into(),
                "contract_name".into(),
                ask_fee.to_owned().map(Uint128::new),
                None,
            ),
        ) {
            panic!("unexpected error: {:?}", error)
        }

        // create ask data
        let create_ask_msg = ExecuteMsg::CreateAsk {
            id: "ask_id".into(),
            quote: coins(100, "quote_1"),
            scope_address: None,
        };

        let asker_info = mock_info("asker", &coins(2, "base_1"));

        // handle create ask
        let create_ask_response = execute(
            deps.as_mut(),
            mock_env(),
            asker_info.clone(),
            create_ask_msg.clone(),
        );

        // verify handle create ask response
        match_create_response(create_ask_response, "Ask", "create_ask", ask_fee);

        // verify ask order stored
        let ask_storage = get_ask_storage_read_v2(&deps.storage);
        if let ExecuteMsg::CreateAsk {
            id,
            quote,
            scope_address: None,
        } = create_ask_msg
        {
            match ask_storage.load("ask_id".to_string().as_bytes()) {
                Ok(stored_order) => {
                    assert_eq!(
                        stored_order,
                        AskOrderV2 {
                            base: BaseType::coins(asker_info.funds),
                            id,
                            owner: asker_info.sender,
                            quote,
                        }
                    )
                }
                _ => {
                    panic!("ask order was not found in storage")
                }
            }
        } else {
            panic!("ask_message is not a CreateAsk type. this is bad.")
        }
    }

    fn test_valid_scope_ask(ask_fee: Option<u128>) {
        let mut deps = mock_dependencies(&[]);
        if let Err(error) = set_contract_info(
            &mut deps.storage,
            &ContractInfo::new(
                Addr::unchecked("contract_admin"),
                "contract_bind_name".into(),
                "contract_name".into(),
                ask_fee.to_owned().map(Uint128::new),
                None,
            ),
        ) {
            panic!("unexpected error: {:?}", error)
        }

        let scope_address = "scope1qraczfp249d3rmysdurne8cxrwmqamu8tk".to_string();

        // create ask data
        let create_ask_msg = ExecuteMsg::CreateAsk {
            id: "ask_id".into(),
            quote: coins(100, "quote_1"),
            scope_address: Some(scope_address.clone()),
        };

        let asker_info = mock_info("asker", &[]);

        deps.querier.with_scope(Scope {
            scope_id: scope_address.clone(),
            specification_id: "scopespec1qs0lctxj49wprm9xwxt5wk0paswqzkdaax".to_string(),
            owners: vec![Party {
                address: Addr::unchecked(MOCK_CONTRACT_ADDR),
                role: PartyType::Owner,
            }],
            data_access: vec![],
            value_owner_address: Addr::unchecked(MOCK_CONTRACT_ADDR),
        });

        // handle create ask
        let create_ask_response = execute(
            deps.as_mut(),
            mock_env(),
            asker_info.clone(),
            create_ask_msg.clone(),
        );

        // verify handle create ask response
        match_create_response(create_ask_response, "Ask", "create_ask", ask_fee);

        // verify ask order stored
        let ask_storage = get_ask_storage_read_v2(&deps.storage);
        if let ExecuteMsg::CreateAsk {
            id,
            quote,
            scope_address,
        } = create_ask_msg
        {
            match ask_storage.load("ask_id".to_string().as_bytes()) {
                Ok(stored_order) => {
                    assert_eq!(
                        stored_order,
                        AskOrderV2 {
                            base: BaseType::scope(scope_address.unwrap()),
                            id,
                            owner: asker_info.sender,
                            quote,
                        }
                    )
                }
                _ => {
                    panic!("ask order was not found in storage")
                }
            }
        } else {
            panic!("ask_message is not a CreateAsk type. this is bad.")
        }
    }

    fn test_valid_coin_bid(bid_fee: Option<u128>) {
        let mut deps = mock_dependencies(&[]);
        if let Err(error) = set_contract_info(
            &mut deps.storage,
            &ContractInfo::new(
                Addr::unchecked("contract_admin"),
                "contract_bind_name".into(),
                "contract_name".into(),
                None,
                bid_fee.to_owned().map(Uint128::new),
            ),
        ) {
            panic!("unexpected error: {:?}", error)
        }

        // create bid data
        let create_bid_msg = ExecuteMsg::CreateBid {
            id: "bid_id".into(),
            base: BaseType::coin(100, "base_1"),
            effective_time: Some(Timestamp::default()),
        };

        let bidder_info = mock_info("bidder", &coins(2, "mark_2"));

        // execute create bid
        let create_bid_response = execute(
            deps.as_mut(),
            mock_env(),
            bidder_info.clone(),
            create_bid_msg.clone(),
        );

        // verify execute create bid response
        match_create_response(create_bid_response, "Bid", "create_bid", bid_fee);

        // verify bid order stored
        let bid_storage = get_bid_storage_read_v2(&deps.storage);
        if let ExecuteMsg::CreateBid {
            id,
            base,
            effective_time,
        } = create_bid_msg
        {
            match bid_storage.load("bid_id".to_string().as_bytes()) {
                Ok(stored_order) => {
                    assert_eq!(
                        stored_order,
                        BidOrderV2 {
                            base,
                            effective_time,
                            id,
                            owner: bidder_info.sender,
                            quote: bidder_info.funds,
                        }
                    )
                }
                _ => {
                    panic!("bid order was not found in storage")
                }
            }
        } else {
            panic!("bid_message is not a CreateBid type. this is bad.")
        }
    }

    fn test_valid_scope_bid(bid_fee: Option<u128>) {
        let mut deps = mock_dependencies(&[]);
        if let Err(error) = set_contract_info(
            &mut deps.storage,
            &ContractInfo::new(
                Addr::unchecked("contract_admin"),
                "contract_bind_name".into(),
                "contract_name".into(),
                None,
                bid_fee.to_owned().map(Uint128::new),
            ),
        ) {
            panic!("unexpected error: {:?}", error)
        }

        // create bid data
        let create_bid_msg = ExecuteMsg::CreateBid {
            id: "bid_id".into(),
            base: BaseType::scope("scope1234"),
            effective_time: Some(Timestamp::default()),
        };

        let bidder_info = mock_info("bidder", &coins(2, "mark_2"));

        // execute create bid
        let create_bid_response = execute(
            deps.as_mut(),
            mock_env(),
            bidder_info.clone(),
            create_bid_msg.clone(),
        );

        // verify execute create bid response
        match_create_response(create_bid_response, "Bid", "create_bid", bid_fee);

        // verify bid order stored
        let bid_storage = get_bid_storage_read_v2(&deps.storage);
        if let ExecuteMsg::CreateBid {
            id,
            base,
            effective_time,
        } = create_bid_msg
        {
            match bid_storage.load("bid_id".to_string().as_bytes()) {
                Ok(stored_order) => {
                    assert_eq!(
                        stored_order,
                        BidOrderV2 {
                            base,
                            effective_time,
                            id,
                            owner: bidder_info.sender,
                            quote: bidder_info.funds,
                        }
                    )
                }
                _ => {
                    panic!("bid order was not found in storage")
                }
            }
        } else {
            panic!("bid_message is not a CreateBid type. this is bad.")
        }
    }
}
