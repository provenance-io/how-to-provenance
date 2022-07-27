use cosmwasm_std::{Addr, StdResult, Storage, Uint128};
use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::error::ContractError;

const NAMESPACE_CONTRACT_INFO: &str = "contract_info";
pub const CONTRACT_TYPE: &str = env!("CARGO_CRATE_NAME");
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub const CONTRACT_INFO: Item<ContractInfo> = Item::new(NAMESPACE_CONTRACT_INFO);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ContractInfo {
    pub admin: Addr,
    pub bind_name: String,
    pub contract_name: String,
    pub contract_type: String,
    pub contract_version: String,
    pub ask_fee: Option<Uint128>,
    pub bid_fee: Option<Uint128>,
}

impl ContractInfo {
    pub fn new(
        admin: Addr,
        bind_name: String,
        contract_name: String,
        ask_fee: Option<Uint128>,
        bid_fee: Option<Uint128>,
    ) -> ContractInfo {
        ContractInfo {
            admin,
            bind_name,
            contract_name,
            contract_type: CONTRACT_TYPE.into(),
            contract_version: CONTRACT_VERSION.into(),
            ask_fee,
            bid_fee,
        }
    }
}

pub fn set_contract_info(
    store: &mut dyn Storage,
    contract_info: &ContractInfo,
) -> Result<(), ContractError> {
    let result = CONTRACT_INFO.save(store, contract_info);
    result.map_err(ContractError::Std)
}

pub fn get_contract_info(store: &dyn Storage) -> StdResult<ContractInfo> {
    CONTRACT_INFO.load(store)
}

#[cfg(test)]
mod tests {
    use provwasm_mocks::mock_dependencies;

    use crate::contract_info::{
        get_contract_info, set_contract_info, ContractInfo, CONTRACT_TYPE, CONTRACT_VERSION,
    };
    use cosmwasm_std::{Addr, Uint128};

    #[test]
    pub fn set_contract_info_with_valid_data() {
        let mut deps = mock_dependencies(&[]);
        let result = set_contract_info(
            &mut deps.storage,
            &ContractInfo::new(
                Addr::unchecked("contract_admin"),
                "contract_bind_name".into(),
                "contract_name".into(),
                Some(Uint128::new(50)),
                Some(Uint128::new(100)),
            ),
        );
        match result {
            Ok(()) => {}
            result => panic!("unexpected error: {:?}", result),
        }

        let contract_info = get_contract_info(&deps.storage);
        match contract_info {
            Ok(contract_info) => {
                assert_eq!(contract_info.admin, Addr::unchecked("contract_admin"));
                assert_eq!(contract_info.bind_name, "contract_bind_name");
                assert_eq!(contract_info.contract_name, "contract_name");
                assert_eq!(contract_info.contract_type, CONTRACT_TYPE);
                assert_eq!(contract_info.contract_version, CONTRACT_VERSION);
            }
            result => panic!("unexpected error: {:?}", result),
        }
    }
}
