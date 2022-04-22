use crate::core::error::ContractError;
use crate::core::msg::NameResponse;
use crate::core::state::config_read;
use cosmwasm_std::{from_binary, to_binary, Binary, Deps, StdResult};
use provwasm_std::{Attribute, Attributes, ProvenanceQuerier, ProvenanceQuery};

// This is a convenient way to query by account address and determine
// all names bound to it within the context of this contract's root name (namespace, if you will)
pub fn query_names_by_address(
    deps: Deps<ProvenanceQuery>,
    address: String,
) -> Result<Binary, ContractError> {
    // Implicitly pull the root registrar name out of the state
    let registrar_name = match config_read(deps.storage).load() {
        Ok(config) => config.name,
        Err(e) => {
            return ContractError::QueryError(format!("failed to load registrar name: {:?}", e))
                .to_result();
        }
    };
    // Validate and convert the provided address into an Addr for the attribute query
    let validated_address = match deps.api.addr_validate(address.as_str()) {
        Ok(addr) => addr,
        Err(e) => {
            return ContractError::QueryError(format!(
                "invalid address provided [{}]: {:?}",
                address, e
            ))
            .to_result();
        }
    };
    // Check for the registered name inside the attributes of the target address
    let attribute_container: Attributes = match ProvenanceQuerier::new(&deps.querier)
        .get_attributes(validated_address, Some(registrar_name))
    {
        Ok(attributes) => attributes,
        Err(e) => {
            return ContractError::QueryError(format!(
                "failed to lookup account by address [{}]: {:?}",
                address, e
            ))
            .to_result();
        }
    };
    // Deserialize all names from their binary-encoded values to the source strings
    let response_bin = match pack_response_from_attributes(attribute_container) {
        Ok(binary) => binary,
        Err(e) => {
            return ContractError::QueryError(format!(
                "failed to pack attribute response to binary: {:?}",
                e
            ))
            .to_result();
        }
    };
    // After establishing a vector of all derived names, serialize the list itself to a binary response
    Ok(response_bin)
}

/// Creates a NameResponse from an Attribute module response. Isolated for unit testing.
fn pack_response_from_attributes(attributes: Attributes) -> StdResult<Binary> {
    let names = attributes
        .attributes
        .iter()
        .map(deserialize_name_from_attribute)
        .collect();
    to_binary(&NameResponse::new(attributes.address.into_string(), names))
}

/// Simple pass-through to convert a binary response from the Attribute module to a usable String.
/// Isolated for unit testing.
fn deserialize_name_from_attribute(attribute: &Attribute) -> String {
    from_binary::<String>(&attribute.value).expect("name deserialization failed")
}

#[cfg(test)]
pub mod tests {
    use crate::core::msg::NameResponse;
    use crate::execute::register_name::register_name;
    use crate::query::query_names_by_address::{
        deserialize_name_from_attribute, pack_response_from_attributes, query_names_by_address,
    };
    use crate::testutil::instantiation_helpers::{test_instantiate, InstArgs};
    use crate::testutil::test_constants::DEFAULT_FEE_AMOUNT;
    use crate::testutil::test_utilities::create_fake_name_attribute;
    use cosmwasm_std::testing::mock_info;
    use cosmwasm_std::{coin, from_binary, Addr};
    use provwasm_mocks::mock_dependencies;
    use provwasm_std::Attributes;

    #[test]
    fn test_deserialize_name_from_attribute() {
        let attribute = create_fake_name_attribute("test_name");
        assert_eq!("test_name", deserialize_name_from_attribute(&attribute));
    }

    #[test]
    fn test_pack_response_from_attributes() {
        let first_name = create_fake_name_attribute("name1");
        let second_name = create_fake_name_attribute("name2");
        let attribute_container = Attributes {
            address: Addr::unchecked("my_address"),
            attributes: vec![first_name, second_name],
        };
        let bin = pack_response_from_attributes(attribute_container)
            .expect("pack_response_from_attributes should create a valid binary");
        let name_response: NameResponse = from_binary(&bin)
            .expect("the generated binary should be resolvable to the source name response");
        assert_eq!(
            "my_address",
            name_response.address.as_str(),
            "the source address should be exposed in the query"
        );
        assert_eq!(
            2,
            name_response.names.len(),
            "the two names should be in the response"
        );
    }

    #[test]
    fn test_name_registration_and_lookup_by_address() {
        // Create mocks
        let mut deps = mock_dependencies(&[]);

        // Create config state
        test_instantiate(deps.as_mut(), InstArgs::default()).unwrap();
        // Drop the name into the system
        let name = "bestnameever".to_string();
        let sender = "registration_guy";
        register_name(
            deps.as_mut(),
            mock_info(sender, &vec![coin(DEFAULT_FEE_AMOUNT, "nhash")]),
            name.clone(),
        )
        .unwrap();
        let name_response_binary =
            query_names_by_address(deps.as_ref(), sender.to_string()).unwrap();
        // Ensure that the result can properly deserialize to a name response
        from_binary::<NameResponse>(&name_response_binary)
            .expect("Expected the response to correctly deserialize to a NameResp value");
    }
}
