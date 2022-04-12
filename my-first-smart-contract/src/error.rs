use cosmwasm_std::StdError;
use thiserror::Error;

/// This error enum is used to cover all errors that can be encountered during contract execution.
/// It utilizes the thiserror crate in order to automatically wrap errors that are produced by other
/// crates in a way that prevents any panics throughout contract execution.  It is highly recommended
/// that panics never be used during contract execution, as the error messages they will produce for
/// users and/or applications that interact with the contract tend to be cryptic and incredibly
/// difficult to debug.
#[derive(Error, Debug)]
pub enum ContractError {
    /// This allows any message to be passed into the ContractError enum as a simple error.
    /// This should be used for one-off issues, where creating a ContractError variant would be
    /// overkill.
    #[error("{0}")]
    GenericError(String),

    #[error("Invalid funds were provided: {explanation}")]
    InvalidFunds { explanation: String },

    #[error("Expected the name {name} to not exist, but it was already bound to address {owner_address}")]
    NameAlreadyExists { name: String, owner_address: String },

    // Ensure that the ContractError can be derived directly from a cosmwasm_std StdError.
    // This will allow the ? operator to magically up-shift cosmwasm errors into ContractError.
    #[error("{0}")]
    Std(#[from] StdError),
}
impl ContractError {
    /// A helper function to create a GenericError using any String-like input.
    /// Changes:
    /// ```
    /// ContractError::GenericError("My message".to_string());
    /// ```
    /// Into:
    /// ```
    /// ContractError::generic_err("My message");
    /// ```
    pub fn generic_err<S: Into<String>>(msg: S) -> Self {
        Self::GenericError(msg.into())
    }
}
