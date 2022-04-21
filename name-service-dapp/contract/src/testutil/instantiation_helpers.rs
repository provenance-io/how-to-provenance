use crate::core::error::ContractError;
use crate::core::msg::InitMsg;
use crate::instantiate::instantiate_contract::instantiate_contract;
use crate::testutil::test_constants::{
    DEFAULT_CONTRACT_NAME, DEFAULT_FEE_AMOUNT, DEFAULT_FEE_COLLECTION_ADDRESS, DEFAULT_INFO_NAME,
};
use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};
use provwasm_std::{ProvenanceMsg, ProvenanceQuery};

/// Holds all instantiation arguments for a test environment, and provides a default implementation
/// for all values, ensuring instantiation is a quick and painless process in testing.
pub struct InstArgs<'a> {
    pub env: Env,
    pub info: MessageInfo,
    pub name: &'a str,
    pub fee_amount: u128,
    pub fee_collection_address: &'a str,
}
impl Default for InstArgs<'_> {
    fn default() -> Self {
        InstArgs {
            env: mock_env(),
            info: mock_info(DEFAULT_INFO_NAME, &[]),
            name: DEFAULT_CONTRACT_NAME,
            fee_amount: DEFAULT_FEE_AMOUNT,
            fee_collection_address: DEFAULT_FEE_COLLECTION_ADDRESS,
        }
    }
}

/// Helper to instantiate the contract without being forced to pass all params, are most are
/// generally unneeded.
pub fn test_instantiate(
    deps: DepsMut<ProvenanceQuery>,
    args: InstArgs,
) -> Result<Response<ProvenanceMsg>, ContractError> {
    instantiate_contract(
        deps,
        args.env,
        args.info,
        InitMsg {
            name: args.name.into(),
            fee_amount: args.fee_amount.to_string(),
            fee_collection_address: args.fee_collection_address.into(),
        },
    )
}
