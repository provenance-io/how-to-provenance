use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    // Ensure that the ContractError can be derived directly from a cosmwasm_std StdError.
    // This will allow the ? operator to magically up-shift cosmwasm errors into ContractError.
    #[error("{0}")]
    Std(#[from] StdError),

    /// This allows any message to be passed into the ContractError enum as a simple error.
    /// This should be used for one-off issues, where creating a ContractError variant would be
    /// overkill.
    #[error("{0}")]
    GenericError(String),

    #[error("Expected the name {name} to not exist, but it was already bound to address {owner_address}")]
    NameAlreadyExists { name: String, owner_address: String },

    #[error("Invalid funds were provided: {explanation}")]
    InvalidFunds { explanation: String },
}
impl ContractError {
    pub fn generic_err<S: Into<String>>(msg: S) -> Self {
        Self::GenericError(msg.into())
    }
}
