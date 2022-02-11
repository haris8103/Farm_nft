/// maps token_id to its level
use cosmwasm_std::{Addr, BlockInfo, Env, StdResult, Storage, Uint128};
use cw721::{ContractInfoResponse, Expiration};
use cw_storage_plus::{Index, IndexList, IndexedMap, Item, Map, MultiIndex};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TokenInfo {
    /// The owner of the newly minted NFT
    pub owner: Addr,
    /// Approvals are stored here, as we clear them all upon transfer and cannot accumulate much
    pub approvals: Vec<Approval>,
    /// Identifies the asset to which this NFT represents
    pub name: String,
    /// rarity of a tool
    pub rarity: String,
    /// mining start time to complete the task and get reward from it (e.g. axe -> wood reward)
    pub reward_start_time: u64,
    /// to check whether this token is pack token or not, pack tokens are those which user will get initialy to get tools tokens
    pub is_pack_token: bool,
    /// to get pre mint tool while opening pack
    pub pre_mint_tool: String,
    /// used for to get with tool type
    pub tool_type: String,
    /// durability will get low when reward is claimed
    pub durability: u64,
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

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct Config {
    pub minter: String,
    pub team_addr: String,   //contains team address for development e.t.c
    pub market_addr: String, //contains market address for marketing
    pub legal_addr: String,  //contains leagal address for legalization
    pub burn_addr: String,   //contains burn address for to burn the amount
    pub stake_limit: u64,    //to limit the user to stake tools
    pub durability_start_time: u64, //start time of deducing durability
    pub reserve_addr: String, //reserve address for contract pool funds
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct RewardToken {
    pub item_name: String,        //items name e.g. wood, gold e.t.c
    pub mining_rate: u64,         //its a rate to earn item amount
    pub mining_waiting_time: u64, //its a waiting time to make task complete
}

// distributing amount between stakeholders
pub fn distribute_amount(
    store: &mut dyn Storage,
    item_name: String,
    amount: Uint128,
    config: &Config,
    env: &Env,
) {
    if amount == Uint128::zero() {
        return;
    }
    // burn user amount 25%
    let burn_amount = amount.multiply_ratio(Uint128::from(25u128), Uint128::from(100u128));
    // transferring amount of team 10% and market 10%
    let team_market_amount = amount.multiply_ratio(Uint128::from(10u128), Uint128::from(100u128));
    // transferring amount of legalization process 5%
    let legal_amount = amount.multiply_ratio(Uint128::from(5u128), Uint128::from(100u128));
    // transferring amount of contract pool 50%
    let contract_pool_amount = amount - burn_amount - team_market_amount - team_market_amount;
    //assigning amount to legal address
    add_amount_in_item_address(
        store,
        config.legal_addr.to_string(),
        item_name.to_string(),
        legal_amount,
    );
    //assigning amount to team address
    add_amount_in_item_address(
        store,
        config.team_addr.to_string(),
        item_name.to_string(),
        team_market_amount,
    );
    //assigning amount to marketing address
    add_amount_in_item_address(
        store,
        config.market_addr.to_string(),
        item_name.to_string(),
        team_market_amount,
    );
    //assigning amount to contract address
    add_amount_in_item_address(
        store,
        env.contract.address.to_string(),
        item_name.to_string(),
        contract_pool_amount,
    );
    //assigning amount to burn address
    add_amount_in_item_address(store, config.burn_addr.to_string(), item_name, burn_amount);
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

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct ToolTemplate {
    pub name: String,
    pub description: String,
    pub image: String,
    pub rarity: String,
    pub durability: u64,
    pub required_amount: Vec<Uint128>,
}

pub const RARITY_TYPES: Map<String, String> = Map::new("Rarities"); // contains rarity stages for upgradation
pub const CONFIG: Item<Config> = Item::new("Config");
pub const TOOL_SET_MAP: Map<String, Vec<String>> = Map::new("ToolSet"); // contains tool set section wise e.g. (wood miner -> Axe, Saw e.t.c)
pub const USER_STAKED_INFO: Map<String, HashSet<String>> = Map::new("UserStakedInfo"); // contains user nft staked info
pub const REWARD_TOKEN: Map<String, RewardToken> = Map::new("RewardToken"); //contains reward tokens
pub const TOOL_TYPE_NAMES: Item<Vec<String>> = Item::new("ToolTypeNames"); // contains tool type names
pub const USER_ENERGY_LEVEL: Map<String, Uint128> = Map::new("UserEnergyLevel"); //to contain the user energy for claiming reward
pub const USER_ITEM_AMOUNT: Map<String, Uint128> = Map::new("UserItemAmount"); // contains the amount of items assigned to particular address
pub const ITEM_TOKEN_MAPPING: Map<String, String> = Map::new("ItemTokenMapping"); //key will be address and value will be item name
pub const TOKEN_ITEM_MAPPING: Map<String, String> = Map::new("TokenItemMapping"); //key will be item name and value will be address
pub const LAST_GEN_TOKEN_ID: Item<u64> = Item::new("LastGenTokenId"); //contains the last token id in generating of nft
pub const TOOL_TEMPLATE_MAP: Map<String, ToolTemplate> = Map::new("ToolTemplateMap"); //contains the template of tool or snapshot to create the new one
pub const GAME_DEV_TOKENS_NAME: Item<HashSet<String>> = Item::new("GameDevTokensName"); // contains the name of game dev token e.g. gWood, gGold e.t.c
pub const TOOL_PACK_SET: Map<String, Vec<String>> = Map::new("ToolPackSet"); //contains pack set against tool type