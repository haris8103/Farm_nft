use cosmwasm_std::{Addr, Uint128};
/// maps token_id to its level
use cw_storage_plus::{Item, Map, U8Key};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct Config {
    pub admin: String,
    pub ust_address: String,
    pub reserve_addr: String, //reserve address for contract pool funds
    pub pack_rate: Uint128,
    pub nft_contract_address: String,
}

pub const LATEST_STAGE_KEY: &str = "stage";
pub const LATEST_STAGE: Item<u8> = Item::new(LATEST_STAGE_KEY);

pub const MERKLE_ROOT_PREFIX: &str = "merkle_root";
pub const MERKLE_ROOT: Map<U8Key, String> = Map::new(MERKLE_ROOT_PREFIX);

pub const CLAIM_PREFIX: &str = "claim";
pub const CLAIM: Map<(&Addr, U8Key), bool> = Map::new(CLAIM_PREFIX);

pub const CONFIG: Item<Config> = Item::new("Config");
