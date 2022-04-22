use crate::core::error::ContractError;

pub fn fee_amount_from_string(fee_amount_string: &str) -> Result<u128, ContractError> {
    match fee_amount_string.parse::<u128>() {
        Ok(amount) => Ok(amount),
        Err(e) => ContractError::std_err(format!(
            "unable to parse input fee amount {} as numeric:\n{}",
            fee_amount_string, e
        )),
    }
}
