use cosmwasm_std::{coin as cosm_coin, Addr, Coin, Storage, Timestamp};
use cosmwasm_storage::{bucket, bucket_read, Bucket, ReadonlyBucket};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub static NAMESPACE_ORDER_ASK: &[u8] = b"ask";
pub static NAMESPACE_ORDER_BID: &[u8] = b"bid";

// V1 Storage
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AskOrder {
    pub base: Vec<Coin>,
    pub id: String,
    pub owner: Addr,
    pub quote: Vec<Coin>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct BidOrder {
    pub base: Vec<Coin>,
    pub effective_time: Option<Timestamp>,
    pub id: String,
    pub owner: Addr,
    pub quote: Vec<Coin>,
}

pub fn get_ask_storage(storage: &mut dyn Storage) -> Bucket<AskOrder> {
    bucket(storage, NAMESPACE_ORDER_ASK)
}
pub fn get_ask_storage_read(storage: &dyn Storage) -> ReadonlyBucket<AskOrder> {
    bucket_read(storage, NAMESPACE_ORDER_ASK)
}
pub fn get_bid_storage(storage: &mut dyn Storage) -> Bucket<BidOrder> {
    bucket(storage, NAMESPACE_ORDER_BID)
}
pub fn get_bid_storage_read(storage: &dyn Storage) -> ReadonlyBucket<BidOrder> {
    bucket_read(storage, NAMESPACE_ORDER_BID)
}

// V2 Storage
pub static NAMESPACE_ORDER_ASK_V2: &[u8] = b"ask_v2";
pub static NAMESPACE_ORDER_BID_V2: &[u8] = b"bid_v2";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BaseType {
    Coin { coins: Vec<Coin> },
    Scope { scope_address: String },
}
impl BaseType {
    pub fn coin(amount: u128, denom: impl Into<String>) -> BaseType {
        BaseType::Coin {
            coins: vec![cosm_coin(amount, denom)],
        }
    }
    pub fn coins(coins: Vec<Coin>) -> BaseType {
        BaseType::Coin { coins }
    }
    pub fn scope(scope_address: impl Into<String>) -> BaseType {
        BaseType::Scope {
            scope_address: scope_address.into(),
        }
    }

    pub fn sorted(&mut self) -> BaseType {
        match self {
            BaseType::Coin { coins } => {
                let coin_sorter = |a: &Coin, b: &Coin| {
                    a.denom.cmp(&b.denom).then_with(|| a.amount.cmp(&b.amount))
                };

                coins.sort_by(coin_sorter);

                BaseType::Coin {
                    coins: coins.to_vec(),
                }
            }
            BaseType::Scope { .. } => self.to_owned(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AskOrderV2 {
    pub base: BaseType,
    pub id: String,
    pub owner: Addr,
    pub quote: Vec<Coin>,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct BidOrderV2 {
    pub base: BaseType,
    pub effective_time: Option<Timestamp>,
    pub id: String,
    pub owner: Addr,
    pub quote: Vec<Coin>,
}

pub fn get_ask_storage_v2(storage: &mut dyn Storage) -> Bucket<AskOrderV2> {
    bucket(storage, NAMESPACE_ORDER_ASK_V2)
}
pub fn get_ask_storage_read_v2(storage: &dyn Storage) -> ReadonlyBucket<AskOrderV2> {
    bucket_read(storage, NAMESPACE_ORDER_ASK_V2)
}
pub fn get_bid_storage_v2(storage: &mut dyn Storage) -> Bucket<BidOrderV2> {
    bucket(storage, NAMESPACE_ORDER_BID_V2)
}
pub fn get_bid_storage_read_v2(storage: &dyn Storage) -> ReadonlyBucket<BidOrderV2> {
    bucket_read(storage, NAMESPACE_ORDER_BID_V2)
}
