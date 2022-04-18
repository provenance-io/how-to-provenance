use crate::error::ContractError;
use cosmwasm_std::Storage;
use cw_storage_plus::Item;
use schemars::JsonSchema;
use semver::Version;
use serde::{Deserialize, Serialize};

/// When cargo is building this project, it automatically adds this env var for the code to infer.
/// See Cargo.toml's name and version fields in the [package] section for the values.
pub const CONTRACT_NAME: &str = env!("CARGO_CRATE_NAME");
/// When cargo is building this project, it automatically adds this env var for the code to infer.
/// See Cargo.toml's name and version fields in the [package] section for the values.
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
/// cw_storage_plus's Item requires a namespace to be used when creating it
const VERSION_INFO_NAMESPACE: &str = "version_info";
/// This example is similar to this contract's implementation of the State struct (in state.rs).
/// The benefit of this usage is that these Item structs can be created as consts, which can
/// be appealing for numerous reasons. This syntax can be much clearer than the state() implementation, as well.
const VERSION_INFO: Item<VersionInfo> = Item::new(VERSION_INFO_NAMESPACE);

/// It is important when migrating to establish boundaries for when a migration is appropriate.
/// One incredibly important feature is ensuring that a migration does not downgrade the version
/// of the contract with an older version.  Using this VersionInfo struct correctly will allow
/// this codebase to only ever move forward on the Provenance blockchain, and prevent accidental
/// overwrites to older veresions.
///
/// A home-grown version-storage struct, to be added to the VERSION_INFO const.
/// Contains information about the contract's name and currently migrated version.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct VersionInfo {
    /// The name of the contract. Should always be a direct reflection of the name property in the package section
    /// of the Cargo.toml file of the project
    pub contract: String,
    /// The version of the contract.  Should always be a direct reflection of the version property in the package
    /// section of the Cargo.toml file of the project.
    pub version: String,
}
impl VersionInfo {
    /// A struct-level helper function to get the current contract name and version from the consts CONTRACT_NAME
    /// and CONTRACT_VERSION. A rust standard might dictate that this function just be the Default implementation
    /// of the struct, but this name is clearer to its purpose.
    pub fn current_version() -> Self {
        Self {
            contract: CONTRACT_NAME.to_string(),
            version: CONTRACT_VERSION.to_string(),
        }
    }

    /// Leverages semver's parse() function to attempt to get a Version from the version property
    /// of this struct.
    pub fn parse_sem_ver(&self) -> Result<Version, ContractError> {
        Ok(self.version.parse()?)
    }
}

/// Leverages the contract's Storage from the DepsMut struct to establish a new VersionInfo
/// struct, stored in the VERSION_INFO const. This will overwrite any existing value.
pub fn set_version_info(
    storage: &mut dyn Storage,
    version_info: &VersionInfo,
) -> Result<(), ContractError> {
    Ok(VERSION_INFO.save(storage, version_info)?)
}

/// Leverages the contract's Storage from the Deps or DepsMut structs to fetch the current
/// VersionInfo struct stored in the VERSION_INFO Item.  If none exists, an error will be returned.
pub fn get_version_info(storage: &dyn Storage) -> Result<VersionInfo, ContractError> {
    Ok(VERSION_INFO.load(storage)?)
}

/// Generates a VersionInfo struct using the current_version function and stores it directly in
/// storage.  A shortcut for manually accomplishing this via set_version_info.
pub fn migrate_version_info(storage: &mut dyn Storage) -> Result<VersionInfo, ContractError> {
    let version_info = VersionInfo::current_version();
    set_version_info(storage, &version_info)?;
    Ok(version_info)
}

#[cfg(test)]
mod tests {
    use super::*;
    use provwasm_mocks::mock_dependencies;

    #[test]
    fn test_set_and_get_version_info() {
        let mut deps = mock_dependencies(&[]);
        set_version_info(
            deps.as_mut().storage,
            &VersionInfo {
                contract: "some-contract".to_string(),
                version: "1.2.3".to_string(),
            },
        )
        .expect("setting version info should succeed");
        let version_info =
            get_version_info(deps.as_ref().storage).expect("fetching version info should succeed");
        assert_eq!(
            "some-contract", version_info.contract,
            "contract name value should be correctly established",
        );
        assert_eq!(
            "1.2.3", version_info.version,
            "contract version value should be correctly established",
        );
    }

    #[test]
    fn test_migrate_version_info() {
        let mut deps = mock_dependencies(&[]);
        let version_info = migrate_version_info(deps.as_mut().storage)
            .expect("migration request should work correctly");
        assert_eq!(
            CONTRACT_NAME, version_info.contract,
            "the env contract name should be stored in the version info",
        );
        assert_eq!(
            CONTRACT_VERSION, version_info.version,
            "the env contract version should be stored in the version info",
        );
        let version_info_from_get = get_version_info(deps.as_ref().storage)
            .expect("version info should be available after using migrate_version_info");
        assert_eq!(
            version_info,
            version_info_from_get,
            "expected the version info fetched by get_version_info to equate to the latest result from migrate_version_info",
        );
    }
}
