use crate::core::error::ContractError;
use crate::core::state::config_read;
use cosmwasm_std::{to_binary, Binary, Deps};
use provwasm_std::ProvenanceQuery;

// This allows an easy way to see the basic details about the contract (root name, fee amount/collection address, etc.)
pub fn query_state(deps: Deps<ProvenanceQuery>) -> Result<Binary, ContractError> {
    let state = config_read(deps.storage).load()?;
    Ok(to_binary(&state)?)
}

#[cfg(test)]
pub mod tests {
    use crate::core::msg::QueryResponse;
    use crate::query::query_state::query_state;
    use crate::testutil::instantiation_helpers::{test_instantiate, InstArgs};
    use cosmwasm_std::from_binary;
    use provwasm_mocks::mock_dependencies;

    #[test]
    fn query_test() {
        // Create mocks
        let mut deps = mock_dependencies(&[]);

        // Create config state
        test_instantiate(deps.as_mut(), InstArgs::default()).unwrap();

        // Call the smart contract query function to get stored state.
        let bin = query_state(deps.as_ref()).unwrap();
        let resp = from_binary::<QueryResponse>(&bin).unwrap();

        // Ensure the expected init fields were properly stored.
        assert_eq!(resp.name, "wallet.pb");
    }
}
