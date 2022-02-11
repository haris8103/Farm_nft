/// maps token_id to its level
use cw_storage_plus::{Item,};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{Uint128};



#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct Config {
    pub admin: String,
    pub ust_address: String,
    pub reserve_addr: String, //reserve address for contract pool funds
    pub pack_rate: Uint128,
    pub nft_contract_address: String,
}


pub const CONFIG: Item<Config> = Item::new("Config");
