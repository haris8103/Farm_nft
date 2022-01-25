/// maps token_id to its level
use cosmwasm_std::{Addr, BlockInfo, StdResult, Storage};
use cw721::{ContractInfoResponse, Expiration};
use cw_storage_plus::{Index, IndexList, IndexedMap, Item, Map, MultiIndex};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

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
    pub cw721_address: String,
}
pub const CONFIG: Item<Config> = Item::new("Config");
pub const REWARDS: Item<Vec<String>> = Item::new("PACKS");
