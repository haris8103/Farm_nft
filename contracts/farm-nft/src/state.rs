use cw_storage_plus::{Map, Item};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
/// maps token_id to its level
pub const LEVEL_DATA: Map<&str, u16> = Map::new("level_data");


#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct Config {
    pub minter: String,
}
pub const CONFIG: Item<Config> = Item::new("Config");