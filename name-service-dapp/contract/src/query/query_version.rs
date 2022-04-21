use crate::core::error::ContractError;
use crate::migrate::version_info::get_version_info;
use cosmwasm_std::{to_binary, Binary, Deps};
use provwasm_std::ProvenanceQuery;

// This allows for easy querying of the version details of a particular contract instance
pub fn query_version(deps: Deps<ProvenanceQuery>) -> Result<Binary, ContractError> {
    let version_info = get_version_info(deps.storage)?;
    Ok(to_binary(&version_info)?)
}

#[cfg(test)]
mod tests {
    use crate::migrate::version_info::{VersionInfoV1, CONTRACT_NAME, CONTRACT_VERSION};
    use crate::query::query_version::query_version;
    use crate::testutil::instantiation_helpers::{test_instantiate, InstArgs};
    use cosmwasm_std::from_binary;
    use provwasm_mocks::mock_dependencies;

    #[test]
    fn query_version_test() {
        let mut deps = mock_dependencies(&[]);
        test_instantiate(deps.as_mut(), InstArgs::default()).unwrap();
        let version_bin = query_version(deps.as_ref()).unwrap();
        let version_info = from_binary::<VersionInfoV1>(&version_bin).unwrap();
        assert_eq!(
            CONTRACT_NAME,
            version_info.contract.as_str(),
            "the contract name should properly be returned via the query",
        );
        assert_eq!(
            CONTRACT_VERSION,
            version_info.version.as_str(),
            "the contract version should properly be returned via the query",
        );
    }
}
