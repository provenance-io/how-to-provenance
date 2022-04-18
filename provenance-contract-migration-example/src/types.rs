use cosmwasm_std::{Api, Coin};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::error::ContractError;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct FeeCollectionDetail {
    pub fee_collector_address: String,
    pub fee_collection_amount: Coin,
}
impl FeeCollectionDetail {
    pub fn get_fee_amount_msg(&self) -> String {
        format!(
            "{}{}",
            self.fee_collection_amount.amount.u128(),
            self.fee_collection_amount.denom
        )
    }

    pub fn self_validate(&self, api: &dyn Api) -> Result<(), ContractError> {
        // Ensure that the provided address is in valid form
        api.addr_validate(&self.fee_collector_address)?;
        if self.fee_collection_amount.amount.is_zero() {
            return Err(ContractError::generic_err(
                "fee collection amount must be greater than zero",
            ));
        }
        if self.fee_collection_amount.denom.is_empty() {
            return Err(ContractError::generic_err(
                "fee collection denom must be defined",
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::{coin, Uint128};
    use provwasm_mocks::mock_dependencies;

    #[test]
    fn test_fee_amount_msg_prints_correct_value() {
        assert_eq!(
            "1500nhash",
            FeeCollectionDetail {
                fee_collector_address: "address".to_string(),
                fee_collection_amount: coin(1500, "nhash")
            }
            .get_fee_amount_msg(),
            "expected the correct amount to be printed when using the coin helper",
        );
    }

    #[test]
    fn test_self_validate_success() {
        let deps = mock_dependencies(&[]);
        FeeCollectionDetail {
            fee_collector_address: "address".to_string(),
            fee_collection_amount: coin(1, "coins"),
        }
        .self_validate(deps.as_ref().api)
        .expect("expected proper input to result in a passing validation");
    }

    #[test]
    fn test_self_validate_failures() {
        let deps = mock_dependencies(&[]);
        // Test bad address
        let error = FeeCollectionDetail {
            fee_collector_address: String::new(),
            fee_collection_amount: coin(1, "bitcoin"),
        }
        .self_validate(deps.as_ref().api)
        .unwrap_err();
        assert!(
            matches!(error, ContractError::Std(..)),
            "a Std error should be emitted when a blank address is attempted",
        );
        // Test zero coin amount
        let error = FeeCollectionDetail {
            fee_collector_address: "address".to_string(),
            fee_collection_amount: Coin {
                denom: "bitcoin".to_string(),
                amount: Uint128::zero(),
            },
        }
        .self_validate(deps.as_ref().api)
        .unwrap_err();
        match error {
            ContractError::GenericError(message) => {
                assert_eq!(
                    "fee collection amount must be greater than zero", message,
                    "unexpected GenericError encountered when bad fee amount supplied",
                );
            }
            _ => panic!(
                "unexpected error encountered when bad fee amount supplied: {:?}",
                error
            ),
        }
        // Test empty denom
        let error = FeeCollectionDetail {
            fee_collector_address: "address".to_string(),
            fee_collection_amount: Coin {
                denom: String::new(),
                amount: Uint128::new(10),
            },
        }
        .self_validate(deps.as_ref().api)
        .unwrap_err();
        match error {
            ContractError::GenericError(message) => {
                assert_eq!(
                    "fee collection denom must be defined", message,
                    "unexpected GenericError encountered when bad denom supplied",
                );
            }
            _ => panic!(
                "unexpected error encountered when bad denom supplied: {:?}",
                error
            ),
        };
    }
}
