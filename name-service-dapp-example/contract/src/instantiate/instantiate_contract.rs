use crate::core::error::ContractError;
use crate::core::msg::InitMsg;
use crate::core::state::{config, State};
use crate::migrate::version_info::migrate_version_info;
use crate::util::helper_functions::fee_amount_from_string;
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};
use provwasm_std::{bind_name, NameBinding, ProvenanceMsg, ProvenanceQuery};

pub fn instantiate_contract(
    deps: DepsMut<ProvenanceQuery>,
    env: Env,
    info: MessageInfo,
    msg: InitMsg,
) -> Result<Response<ProvenanceMsg>, ContractError> {
    // Ensure no funds were sent with the message
    if !info.funds.is_empty() {
        return ContractError::std_err("purchase funds are not allowed to be sent during init");
    }
    // Verify the fee amount can be converted from string successfully
    fee_amount_from_string(&msg.fee_amount)?;
    // Create and save contract config state. The name is used for setting attributes on user accounts
    match config(deps.storage).save(&State {
        name: msg.name.clone(),
        fee_amount: msg.fee_amount.clone(),
        fee_collection_address: msg.fee_collection_address.clone(),
    }) {
        Ok(_) => {}
        Err(e) => {
            return ContractError::std_err(format!("failed to init state: {:?}", e));
        }
    };

    // Create a message that will bind a restricted name to the contract address.
    // this name being restricted means that only this smart contract will have the authority to issue any sub-names below this name
    // because the name is bound to this contract's address, only this particular contract instance will be able to attach attributes
    // to accounts (or scopes, or anything else one could attach a name to in Provenance Blockchain). This allows one to verify that
    // any attribute under this name was written by this contract and conformed to the rules defined within this contract.
    let bind_name_msg = match bind_name(&msg.name, env.contract.address, NameBinding::Restricted) {
        Ok(result) => result,
        Err(e) => {
            return ContractError::std_err(format!(
                "failed to construct bind name message: {:?}",
                e
            ));
        }
    };

    // Set the version info to the default contract values on instantiation
    migrate_version_info(deps.storage)?;

    // Dispatch messages and emit event attributes
    Ok(Response::new()
        .add_message(bind_name_msg)
        .add_attribute("action", "init"))
}

#[cfg(test)]
pub mod tests {
    use crate::migrate::version_info::{get_version_info, CONTRACT_NAME, CONTRACT_VERSION};
    use crate::testutil::instantiation_helpers::{test_instantiate, InstArgs};
    use cosmwasm_std::CosmosMsg;
    use provwasm_mocks::mock_dependencies;
    use provwasm_std::{NameMsgParams, ProvenanceMsgParams};

    #[test]
    fn valid_init() {
        // Create mocks
        let mut deps = mock_dependencies(&[]);

        // Create valid config state
        let res = test_instantiate(deps.as_mut(), InstArgs::default()).unwrap();

        // Ensure a message was created to bind the name to the contract address.
        assert_eq!(res.messages.len(), 1);
        match &res.messages[0].msg {
            CosmosMsg::Custom(msg) => match &msg.params {
                ProvenanceMsgParams::Name(p) => match &p {
                    NameMsgParams::BindName { name, .. } => assert_eq!(name, "wallet.pb"),
                    _ => panic!("unexpected name params"),
                },
                _ => panic!("unexpected provenance params"),
            },
            _ => panic!("unexpected cosmos message"),
        }
        let version_info = get_version_info(deps.as_ref().storage).unwrap();
        assert_eq!(
            CONTRACT_NAME,
            version_info.contract.as_str(),
            "the contract name should be stored in version info on a successful instantiation",
        );
        assert_eq!(
            CONTRACT_VERSION,
            version_info.version.as_str(),
            "the contract version should be stored in version info on a successful instantiation",
        );
    }
}
