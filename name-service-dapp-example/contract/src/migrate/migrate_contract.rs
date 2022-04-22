use crate::core::error::ContractError;
use crate::core::msg::MigrateMsg;
use crate::core::state::config;
use crate::migrate::version_info::{
    get_version_info, migrate_version_info, CONTRACT_NAME, CONTRACT_VERSION,
};
use crate::util::helper_functions::fee_amount_from_string;
use cosmwasm_std::{DepsMut, Response};
use provwasm_std::ProvenanceQuery;
use semver::Version;

// A rather basic migration implementation that allows for updates to the fee collection amount/destination,
// as well as standard migrations between contract versions
pub fn migrate_contract(
    deps: DepsMut<ProvenanceQuery>,
    msg: MigrateMsg,
) -> Result<Response, ContractError> {
    let stored_version_info = get_version_info(deps.storage)?;
    // If the contract name has changed or another contract attempts to overwrite this one, this
    // check will reject the change
    if CONTRACT_NAME != stored_version_info.contract {
        return ContractError::InvalidContractName {
            current_contract: stored_version_info.contract,
            migration_contract: CONTRACT_NAME.to_string(),
        }
        .to_result();
    }
    let contract_version = CONTRACT_VERSION.parse::<Version>()?;
    // If the stored version in the contract is greater than the derived version from the package,
    // then this migration is effectively a downgrade and should not be committed
    if stored_version_info.parse_sem_ver()? > contract_version {
        return ContractError::InvalidContractVersion {
            current_version: stored_version_info.version,
            migration_version: CONTRACT_VERSION.to_string(),
        }
        .to_result();
    }
    let mut attributes: Vec<cosmwasm_std::Attribute> = vec![];
    // If any optional migration values were provided, swap them over during the migration
    if msg.has_fee_changes() {
        let mut config = config(deps.storage);
        let mut state = config.load()?;
        state.fee_amount = match msg.new_fee_amount {
            Some(amount) => {
                fee_amount_from_string(amount.as_str())?;
                attributes.push(cosmwasm_std::Attribute::new(
                    "fee_amount_updated",
                    amount.clone(),
                ));
                amount
            }
            None => state.fee_amount,
        };
        state.fee_collection_address = match msg.new_fee_collection_address {
            Some(addr_str) => {
                deps.api.addr_validate(addr_str.as_str())?;
                attributes.push(cosmwasm_std::Attribute::new(
                    "fee_collection_address_updated",
                    addr_str.clone(),
                ));
                addr_str
            }
            None => state.fee_collection_address,
        };
        config.save(&state)?;
    }
    // Ensure that the new contract version is stored for future migrations to reference
    migrate_version_info(deps.storage)?;
    Ok(Response::new().add_attributes(attributes))
}

#[cfg(test)]
mod tests {
    use crate::core::error::ContractError;
    use crate::core::msg::MigrateMsg;
    use crate::migrate::migrate_contract::migrate_contract;
    use crate::migrate::version_info::{
        get_version_info, set_version_info, VersionInfoV1, CONTRACT_NAME, CONTRACT_VERSION,
    };
    use crate::testutil::instantiation_helpers::{test_instantiate, InstArgs};
    use provwasm_mocks::mock_dependencies;

    #[test]
    fn test_migration_with_no_state_changes() {
        let mut deps = mock_dependencies(&[]);
        test_instantiate(deps.as_mut(), InstArgs::default()).unwrap();
        let migrate_response = migrate_contract(deps.as_mut(), MigrateMsg::empty()).unwrap();
        assert!(
            migrate_response.attributes.is_empty(),
            "no attributes should be added, indicating that the migration made no changes"
        );
    }

    #[test]
    fn test_migration_to_new_version_sets_version_info() {
        let mut deps = mock_dependencies(&[]);
        test_instantiate(deps.as_mut(), InstArgs::default()).unwrap();
        // Downgrade the contract internally to verify that the values are reset after migration
        set_version_info(
            deps.as_mut().storage,
            &VersionInfoV1 {
                contract: CONTRACT_NAME.to_string(),
                version: "0.0.1".to_string(),
            },
        )
        .unwrap();
        migrate_contract(deps.as_mut(), MigrateMsg::empty()).unwrap();
        let version_info = get_version_info(deps.as_ref().storage).unwrap();
        assert_eq!(
            CONTRACT_NAME,
            version_info.contract.as_str(),
            "the proper contract name should be set after the migration completes",
        );
        assert_eq!(
            CONTRACT_VERSION,
            version_info.version.as_str(),
            "the proper contract version should be set after the migration completes",
        );
    }

    #[test]
    fn test_migration_with_only_fee_changed() {
        let mut deps = mock_dependencies(&[]);
        test_instantiate(
            deps.as_mut(),
            InstArgs {
                fee_amount: 100,
                fee_collection_address: "fake_address",
                ..Default::default()
            },
        )
        .unwrap();
        let migrate_response = migrate_contract(
            deps.as_mut(),
            MigrateMsg {
                new_fee_amount: Some("150".to_string()),
                new_fee_collection_address: None,
            },
        )
        .unwrap();
        assert_eq!(
            1,
            migrate_response.attributes.len(),
            "only one attribute should be added, indicating that a single state value was changed"
        );
        let attribute = migrate_response
            .attributes
            .first()
            .expect("The first element should be available within the migration values");
        assert_eq!(
            "fee_amount_updated",
            attribute.key.as_str(),
            "Expected the key to show that the fee was changed",
        );
        assert_eq!(
            "150",
            attribute.value.as_str(),
            "Expected the value to show the new value that the fee amount was updated to",
        );
    }

    #[test]
    fn test_migration_with_only_fee_address_changed() {
        let mut deps = mock_dependencies(&[]);
        test_instantiate(
            deps.as_mut(),
            InstArgs {
                fee_amount: 100,
                fee_collection_address: "fake_address",
                ..Default::default()
            },
        )
        .unwrap();
        let migrate_response = migrate_contract(
            deps.as_mut(),
            MigrateMsg {
                new_fee_amount: None,
                new_fee_collection_address: Some("new_address".to_string()),
            },
        )
        .unwrap();
        assert_eq!(
            1,
            migrate_response.attributes.len(),
            "only one attribute should be added, indicating that a single state value was changed"
        );
        let attribute = migrate_response
            .attributes
            .first()
            .expect("The first element should be available within the migration values");
        assert_eq!(
            "fee_collection_address_updated",
            attribute.key.as_str(),
            "Expected the key to show that the fee address was changed",
        );
        assert_eq!(
            "new_address",
            attribute.value.as_str(),
            "Expected the value to show the new value that the fee address was updated to",
        );
    }

    #[test]
    fn test_migration_with_invalid_new_fee_amount() {
        let mut deps = mock_dependencies(&[]);
        test_instantiate(deps.as_mut(), InstArgs::default()).unwrap();
        migrate_contract(
            deps.as_mut(),
            MigrateMsg {
                new_fee_amount: Some("not a number".to_string()),
                new_fee_collection_address: None,
            },
        )
        .unwrap_err();
    }

    #[test]
    fn test_migration_with_invalid_new_fee_collection_address() {
        let mut deps = mock_dependencies(&[]);
        test_instantiate(deps.as_mut(), InstArgs::default()).unwrap();
        migrate_contract(
            deps.as_mut(),
            MigrateMsg {
                new_fee_amount: None,
                new_fee_collection_address: Some("".to_string()),
            },
        )
        .unwrap_err();
    }

    #[test]
    fn test_migration_with_invalid_contract_name() {
        let mut deps = mock_dependencies(&[]);
        test_instantiate(deps.as_mut(), InstArgs::default()).unwrap();
        // Override the internal contract name to a new, different name
        set_version_info(
            deps.as_mut().storage,
            &VersionInfoV1 {
                contract: "Fake Name".to_string(),
                version: CONTRACT_VERSION.to_string(),
            },
        )
        .unwrap();
        let error = migrate_contract(deps.as_mut(), MigrateMsg::empty()).unwrap_err();
        match error {
            ContractError::InvalidContractName {
                current_contract,
                migration_contract,
            } => {
                assert_eq!(
                    "Fake Name", current_contract,
                    "the current contract name should be the value in storage",
                );
                assert_eq!(
                    CONTRACT_NAME,
                    migration_contract.as_str(),
                    "the migration contract name should be the cargo package name",
                );
            }
            _ => panic!("unexpected error encountered when bad contract name provided"),
        };
    }

    #[test]
    fn test_migration_with_invalid_version() {
        let mut deps = mock_dependencies(&[]);
        test_instantiate(deps.as_mut(), InstArgs::default()).unwrap();
        // Override the internal contract version to a version largely above the current version
        set_version_info(
            deps.as_mut().storage,
            &VersionInfoV1 {
                contract: CONTRACT_NAME.to_string(),
                version: "9.9.9".to_string(),
            },
        )
        .unwrap();
        let error = migrate_contract(deps.as_mut(), MigrateMsg::empty()).unwrap_err();
        match error {
            ContractError::InvalidContractVersion {
                current_version,
                migration_version,
            } => {
                assert_eq!(
                    "9.9.9", current_version,
                    "the current contract version should be the value in storage",
                );
                assert_eq!(
                    CONTRACT_VERSION,
                    migration_version.as_str(),
                    "the migration contract version should be the cargo package version",
                );
            }
            _ => panic!("unexpected error encountered when bad contract version provided"),
        }
    }
}
