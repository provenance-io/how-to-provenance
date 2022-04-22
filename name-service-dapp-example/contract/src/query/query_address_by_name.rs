use crate::core::error::ContractError;
use crate::core::state::meta_read;
use cosmwasm_std::{to_binary, Binary, Deps};
use provwasm_std::ProvenanceQuery;

/// This is the main query functionality provided by this contract, name resolution.
/// Similar to the DNS system that allows one to resolve a server's ip address by
/// a website url, this allows other smart contracts or off-chain processes to resolve
/// an account address via a human-readable name.
///
/// Within the context of a smart contract utilizing this query functionality, one can be assured
/// that the address resolved by this query is still bound to the name provided in the atomic
/// context provided by contract execution
pub fn query_address_by_name(
    deps: Deps<ProvenanceQuery>,
    name: String,
) -> Result<Binary, ContractError> {
    let meta_storage = meta_read(deps.storage);
    let name_meta = meta_storage.load(name.as_bytes())?;
    Ok(to_binary(&name_meta)?)
}
