use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized,
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
    #[error("Name [{name:?}] is already registered")]
    // The msg param is strictly for internal testing
    NameRegistered { name: String },

    #[error("Name serialization failed due to {cause:?}")]
    NameSerializationFailure { cause: StdError },

    #[error("Name not found")]
    NameNotFound,

    #[error("No nhash amount provided during name registration")]
    NoFundsProvidedForRegistration,

    #[error("Current contract name [{current_contract}] does not match provided migration name [{migration_contract}]")]
    InvalidContractName {
        current_contract: String,
        migration_contract: String,
    },

    #[error("Current contract version [{current_version}] is higher than provided migration version [{migration_version}]")]
    InvalidContractVersion {
        current_version: String,
        migration_version: String,
    },

    #[error("Non nhash coin provided for transaction {types:?}")]
    InvalidFundsProvided { types: Vec<String> },

    #[error("Name has invalid format. Names should be all lowercase with no spaces or special characters. Name used: [{name}]")]
    InvalidNameFormat { name: String },

    #[error("Insufficient funds provided for name registration. Provided {amount_provided:?} but required {amount_required:?}")]
    InsufficientFundsProvided {
        amount_provided: u128,
        amount_required: u128,
    },

    #[error("Invalid fields: {fields:?}")]
    InvalidFields { fields: Vec<String> },

    #[error("Semver parsing error: {0}")]
    SemVer(String),

    #[error("Query failed: {0}")]
    QueryError(String),
}
impl ContractError {
    /// Allows ContractError instances to be generically returned as a Response in a fluent manner
    /// instead of wrapping in an Err() call, improving readability.
    /// Ex: return ContractError::NameNotFound.to_result();
    /// vs
    ///     return Err(ContractError::NameNotFound);
    pub fn to_result<T>(self) -> Result<T, ContractError> {
        Err(self)
    }
    /// A simple abstraction to wrap an error response just by passing the message
    pub fn std_err<T>(msg: impl Into<String>) -> Result<T, ContractError> {
        Err(ContractError::Std(StdError::generic_err(msg)))
    }
}
impl From<semver::Error> for ContractError {
    /// Enables SemVer issues to cast convert implicitly to contract error
    fn from(err: semver::Error) -> Self {
        Self::SemVer(err.to_string())
    }
}
