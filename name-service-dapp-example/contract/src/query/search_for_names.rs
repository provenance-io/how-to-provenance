use crate::core::error::ContractError;
use crate::core::msg::NameSearchResponse;
use crate::core::state::meta_read;
use crate::util::constants::MAX_NAME_SEARCH_RESULTS;
use cosmwasm_std::{to_binary, Binary, Deps, Order};
use provwasm_std::ProvenanceQuery;

/// Scans the entire storage for the target name string by doing substring matches.
/// Will only ever return a maximum of MAX_NAME_SEARCH_RESULTS.
/// This allows for some basic pseudo fuzzy-search style results of names
pub fn search_for_names(
    deps: Deps<ProvenanceQuery>,
    search: String,
) -> Result<Binary, ContractError> {
    let meta_storage = meta_read(deps.storage);
    let search_str = search.as_str();
    let names = meta_storage
        .range(None, None, Order::Ascending)
        .into_iter()
        .filter(|element| element.is_ok())
        .map(|element| element.unwrap().1)
        .filter(|name_meta| name_meta.name.contains(search_str))
        .take(MAX_NAME_SEARCH_RESULTS)
        .collect();
    to_binary(&NameSearchResponse {
        search: search.clone(),
        names,
    })
    .map_err(ContractError::Std)
}

#[cfg(test)]
mod tests {
    use crate::core::msg::NameSearchResponse;
    use crate::execute::register_name::register_name;
    use crate::query::search_for_names::search_for_names;
    use crate::testutil::instantiation_helpers::{test_instantiate, InstArgs};
    use crate::testutil::test_constants::DEFAULT_FEE_AMOUNT;
    use crate::util::constants::{FEE_DENOMINATION, MAX_NAME_SEARCH_RESULTS};
    use cosmwasm_std::testing::mock_info;
    use cosmwasm_std::{coin, from_binary};
    use provwasm_mocks::mock_dependencies;

    #[test]
    fn test_search_for_names() {
        let mut deps = mock_dependencies(&[]);
        test_instantiate(deps.as_mut(), InstArgs::default()).unwrap();
        let mut names: Vec<String> = vec![
            "a".into(),
            "aa".into(),
            "ab".into(),
            "ac".into(),
            "test".into(),
        ];
        // Add a ton of stuff prefixed with b to the array to simulate a fully-used name backend
        for i in 0..1000 {
            names.push(format!("b{}", i));
        }
        // Register all names and fail if anything doesn't result in a success
        names.into_iter().for_each(|name| {
            register_name(
                deps.as_mut(),
                mock_info(
                    "fake_address",
                    &vec![coin(DEFAULT_FEE_AMOUNT, FEE_DENOMINATION)],
                ),
                name.into(),
            )
            .unwrap();
        });
        // Make the search functionality easy to re-use as a closure
        let search = |search_param: &str| {
            let result_bin = search_for_names(deps.as_ref(), search_param.to_string())
                .expect(format!("expected the name search to properly respond with binary for search input \"{}\"", search_param).as_str());
            from_binary::<NameSearchResponse>(&result_bin)
                .expect("expected binary deserialization to a NameSearchResposne to succeed")
        };
        // Verify that all the things added with "a" in them can be found
        let name_result = search("a");
        assert_eq!(
            "a",
            name_result.search.as_str(),
            "expected the search value to reflect the input"
        );
        assert_eq!(
            4,
            name_result.names.len(),
            "all four results containing the letter \"a\" should be returned"
        );
        name_result
            .names
            .iter()
            .find(|meta| meta.name == "a")
            .expect("the value \"a\" should be amongst the results");
        name_result
            .names
            .iter()
            .find(|meta| meta.name == "aa")
            .expect("the value \"aa\" should be amongst the results");
        name_result
            .names
            .iter()
            .find(|meta| meta.name == "ab")
            .expect("the value \"ab\" should be amongst the results");
        name_result
            .names
            .iter()
            .find(|meta| meta.name == "ac")
            .expect("the value \"ac\" should be amongst the results");
        assert!(
            name_result.names.iter().find(|meta| meta.name == "test").is_none(),
            "the value \"test\" should not be included in results because it does not contain the search string",
        );
        // Verify that the only result when using a direct search will be found
        let test_search_result = search("test");
        assert_eq!(
            1,
            test_search_result.names.len(),
            "expected only a single result to match for input \"test\""
        );
        test_search_result
            .names
            .iter()
            .find(|meta| meta.name == "test")
            .expect("the value \"test\" should be amongst the results");
        // Verify that all of the "b" names added in the loop were added
        let end_of_additions_result = search("b999");
        assert_eq!(
            1,
            end_of_additions_result.names.len(),
            "expected the final b name to be added"
        );
        end_of_additions_result
            .names
            .iter()
            .find(|meta| meta.name == "b999")
            .expect("the value \"b999\" should be amongst the results");
        // Verify that searches that find more than MAX_NAME_SEARCH_RESULTS will only find those results
        let large_search_result = search("b");
        assert_eq!(
            MAX_NAME_SEARCH_RESULTS,
            large_search_result.names.len(),
            "expected only the max search results to be returned when a query would find more results",
        );
        assert!(
            large_search_result
                .names
                .iter()
                .all(|meta| meta.name.contains("b")),
            "all results found should contain the letter \"b\" as indicated by the query",
        );
        // Verify that a search that hits nothing will return an empty array
        let empty_search_result = search("test0");
        assert_eq!(
            0,
            empty_search_result.names.len(),
            "a search that finds nothing should return an empty vector of names"
        );
    }
}
