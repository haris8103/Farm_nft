/// maps token_id to its level
use cosmwasm_std::{Addr, BlockInfo, Env, StdResult, Storage, Uint128};
use cw721::{ContractInfoResponse, Expiration};
use cw_storage_plus::{Index, IndexList, IndexedMap, Item, Map, MultiIndex};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{ HashSet};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TokenInfo {
    /// The owner of the newly minted NFT
    pub owner: Addr,
    /// Approvals are stored here, as we clear them all upon transfer and cannot accumulate much
    pub approvals: Vec<Approval>,

    /// Identifies the asset to which this NFT represents
    pub name: String,
    /// Describes the asset to which this NFT represents
    pub description: String,
    /// A URI pointing to an image representing the asset
    pub image: String,

    pub rarity: String,

    pub reward_start_time: u64,

    pub is_pack_token: bool,

    pub pre_mint_tool: String,

    pub tool_type: String,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct Approval {
    /// Account that can transfer/send the token
    pub spender: Addr,
    /// When the Approval expires (maybe Expiration::never)
    pub expires: Expiration,
}

impl Approval {
    pub fn is_expired(&self, block: &BlockInfo) -> bool {
        self.expires.is_expired(block)
    }
}

pub const CONTRACT_INFO: Item<ContractInfoResponse> = Item::new("nft_info");
pub const MINTER: Item<Addr> = Item::new("minter");
pub const TOKEN_COUNT: Item<u64> = Item::new("num_tokens");

// Stored as (granter, operator) giving operator full control over granter's account
pub const OPERATORS: Map<(&Addr, &Addr), Expiration> = Map::new("operators");

pub fn num_tokens(storage: &dyn Storage) -> StdResult<u64> {
    Ok(TOKEN_COUNT.may_load(storage)?.unwrap_or_default())
}

pub fn increment_tokens(storage: &mut dyn Storage) -> StdResult<u64> {
    let val = num_tokens(storage)? + 1;
    TOKEN_COUNT.save(storage, &val)?;

    Ok(val)
}

pub struct TokenIndexes<'a> {
    // pk goes to second tuple element
    pub owner: MultiIndex<'a, (Addr, Vec<u8>), TokenInfo>,
}

impl<'a> IndexList<TokenInfo> for TokenIndexes<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<TokenInfo>> + '_> {
        let v: Vec<&dyn Index<TokenInfo>> = vec![&self.owner];
        Box::new(v.into_iter())
    }
}

pub fn tokens<'a>() -> IndexedMap<'a, &'a str, TokenInfo, TokenIndexes<'a>> {
    let indexes = TokenIndexes {
        owner: MultiIndex::new(
            |d: &TokenInfo, k: Vec<u8>| (d.owner.clone(), k),
            "tokens",
            "tokens__owner",
        ),
    };
    IndexedMap::new("tokens", indexes)
}

pub const LEVEL_DATA: Map<&str, u16> = Map::new("level_data");

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct Config {
    pub minter: String,
    pub team_addr: String,
    pub market_addr: String,
    pub legal_addr: String,
    pub burn_addr: String,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct RewardToken {
    pub item_name: String,
    pub mining_rate: u64,
    pub mining_waiting_time: u64,
}

pub fn distribute_amount(
    store: &mut dyn Storage,
    item: String,
    amount: Uint128,
    config: &Config,
    env: &Env,
) {
    if amount == Uint128::zero() {
        return;
    }
    let burn_amount = amount.multiply_ratio(Uint128::from(25u128), Uint128::from(100u128));
    let team_market_amount = amount.multiply_ratio(Uint128::from(10u128), Uint128::from(100u128));
    let legal_amount = amount.multiply_ratio(Uint128::from(5u128), Uint128::from(100u128));
    let contract_pool_amount = amount - burn_amount - team_market_amount - team_market_amount;
    add_amount_in_item_address(
        store,
        config.legal_addr.to_string(),
        item.to_string(),
        legal_amount,
    );
    add_amount_in_item_address(
        store,
        config.team_addr.to_string(),
        item.to_string(),
        team_market_amount,
    );
    add_amount_in_item_address(
        store,
        config.market_addr.to_string(),
        item.to_string(),
        team_market_amount,
    );
    add_amount_in_item_address(
        store,
        env.contract.address.to_string(),
        item.to_string(),
        contract_pool_amount,
    );
    add_amount_in_item_address(store, config.burn_addr.to_string(), item, burn_amount);
}

pub fn add_amount_in_item_address(
    store: &mut dyn Storage,
    addr: String,
    item: String,
    amount: Uint128,
) {
    let mut item_key = addr;
    item_key.push_str(&item);
    let mut item_amount = if let Some(item_amount) = USER_ITEM_AMOUNT
        .may_load(store, item_key.to_string())
        .unwrap()
    {
        item_amount
    } else {
        Uint128::zero()
    };
    item_amount += amount;
    USER_ITEM_AMOUNT
        .save(store, item_key.to_string(), &item_amount)
        .unwrap();
}
// #[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
// pub struct UserTokenInfo {
//     pub amount: String,
//     pub mining_rate: u64,
//     pub mining_waiting_time: u64,
// }
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct ToolTemplate {
    pub name: String,
    pub description: String,
    pub image: String,
    pub rarity: String,
    pub required_gwood_amount: Uint128,
    pub required_gfood_amount: Uint128,
    pub required_ggold_amount: Uint128,
    pub required_gstone_amount: Uint128,
}
pub const RARITY_TYPES: Map<String, String> = Map::new("Rarities");
pub const CONFIG: Item<Config> = Item::new("Config");
pub const REWARDS: Map<String, Vec<String>> = Map::new("Rewards");
//pub const REWARD_ITEMS: Item<HashSet<String>> = Item::new("RewardItems");
pub const USER_STAKED_INFO: Map<String, HashSet<String>> = Map::new("UserStakedInfo");
pub const REWARD_TOKEN: Map<String, RewardToken> = Map::new("RewardToken");
pub const NFT_NAMES: Item<Vec<String>> = Item::new("CommonNftNames");
pub const USER_ENERGY_LEVEL: Map<String, Uint128> = Map::new("UserEnergyLevel");
pub const USER_ITEM_AMOUNT: Map<String, Uint128> = Map::new("UserItemAmount");
pub const ITEM_TOKEN_MAPPING: Map<String, String> = Map::new("ItemTokenMapping"); //key will be address and value will be item name
pub const TOKEN_ITEM_MAPPING: Map<String, String> = Map::new("TokenItemMapping"); //key will be item name and value will be address
pub const LAST_GEN_TOKEN_ID: Item<u64> = Item::new("LastGenTokenId");
pub const TOOL_TEMPLATE_MAP: Map<String, ToolTemplate> = Map::new("ToolTemplateMap");
