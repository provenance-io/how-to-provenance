use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Storage;
use cosmwasm_storage::{
    bucket, bucket_read, singleton, singleton_read, Bucket, ReadonlyBucket, ReadonlySingleton,
    Singleton,
};

pub static CONFIG_KEY: &[u8] = b"config";
pub static NAME_META_KEY: &[u8] = b"name_meta";

/// Fields that comprise the smart contract state
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub name: String,
    pub fee_amount: String,
    pub fee_collection_address: String,
}

pub fn config(storage: &mut dyn Storage) -> Singleton<State> {
    singleton(storage, CONFIG_KEY)
}

pub fn config_read(storage: &dyn Storage) -> ReadonlySingleton<State> {
    singleton_read(storage, CONFIG_KEY)
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct NameMeta {
    pub name: String,
    pub address: String,
}

pub fn meta(storage: &mut dyn Storage) -> Bucket<NameMeta> {
    bucket(storage, NAME_META_KEY)
}

pub fn meta_read(storage: &dyn Storage) -> ReadonlyBucket<NameMeta> {
    bucket_read(storage, NAME_META_KEY)
}
