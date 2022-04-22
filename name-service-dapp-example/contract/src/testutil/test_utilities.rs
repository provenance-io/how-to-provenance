use cosmwasm_std::to_binary;
use provwasm_std::{Attribute, AttributeValueType};

/// Helper to build an Attribute without having to do all the un-fun stuff repeatedly
pub fn create_fake_name_attribute<S: Into<String>>(name: S) -> Attribute {
    Attribute {
        name: "wallet.pb".into(),
        value: to_binary(&name.into()).expect("attribute name should serialize correctly"),
        value_type: AttributeValueType::String,
    }
}
