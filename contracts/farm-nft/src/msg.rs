use cosmwasm_std::{Addr, Binary, Uint128};
use cw20::Cw20ReceiveMsg;
use cw721::{Cw721ReceiveMsg, OwnerOfResponse};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    /// Name of the NFT contract
    pub name: String,
    /// Symbol of the NFT contract
    pub symbol: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MintMsg {
    pub owner: Addr,
    /// Unique ID of the NFT
    pub token_id: String,
    /// Identifies the asset to which this NFT represents
    pub name: String,
    /// A URI pointing to an image representing the asset
    pub image: String,
    /// Describes the asset to which this NFT represents (may be empty)
    pub description: Option<String>,
    /// Custom extensions
    pub rarity: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct BoostMsg {
    pub token_ids: Vec<String>,
    /// Unique ID of the NFT
    pub token_id: String,
    /// Identifies the asset to which this NFT represents
    pub name: String,
    /// A URI pointing to an image representing the asset
    pub image: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Mint a new NFT, can only be called by the contract minter
    Mint(MintMsg),

    /// Merge existing NFTs and mint new level NFT
    //  Boost(BoostMsg),

    /// Transfer is a base message to move a token to another account without triggering actions
    TransferNft {
        recipient: String,
        token_id: String,
    },
    Burn {
        token_id: String,
    },

    /// Send is a base message to transfer a token to a contract and trigger an action
    /// on the receiving contract.
    SendNft {
        contract: String,
        token_id: String,
        msg: Binary,
    },

    ReceiveNft(Cw721ReceiveMsg),

    ClaimReward {
        token_id: String,
    },
    Receive(Cw20ReceiveMsg),

    AddRewardToken {
        contract_addr: String,
        tool_name: String,
        mining_rate: Uint128,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// Return the owner of the given token, error if token does not exist
    /// Return type: OwnerOfResponse
    OwnerOf { token_id: String },

    /// Total number of tokens issued
    NumTokens {},

    /// With MetaData Extension.
    /// Returns top-level metadata about the contract: `ContractInfoResponse`
    ContractInfo {},

    /// With MetaData Extension.
    /// Returns metadata about one particular token, based on *ERC721 Metadata JSON Schema*
    /// but directly from the contract: `NftInfoResponse`
    NftInfo { token_id: String },

    /// With MetaData Extension.
    /// Returns the result of both `NftInfo` and `OwnerOf` as one query as an optimization
    /// for clients: `AllNftInfo`
    AllNftInfo { token_id: String },

    /// With Enumerable extension.
    /// Returns all tokens owned by the given address, [] if unset.
    /// Return type: TokensResponse.
    Tokens {
        owner: String,
        start_after: Option<String>,
        limit: Option<u32>,
    },

    /// With Enumerable extension.
    /// Requires pagination. Lists all token_ids controlled by the contract.
    /// Return type: TokensResponse.
    AllTokens {
        start_after: Option<String>,
        limit: Option<u32>,
    },
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct TokenResponse {
    pub token_id: String,
    pub owner: Addr,
    pub name: String,
    pub image: Option<String>,
    pub level: u16,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct Extension {
    /// Identifies the asset to which this NFT represents
    pub name: String,
    /// Describes the asset to which this NFT represents
    pub description: String,
    /// "A URI pointing to a resource with mime type image/* representing the asset to which this
    /// NFT represents. Consider making any images at a width between 320 and 1080 pixels and aspect
    /// ratio between 1.91:1 and 4:5 inclusive.
    /// TODO: Use https://docs.rs/url_serde for type-safety
    pub image: Option<String>,
    pub rarity: String,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct NftInfoResponse {
    pub token_uri: String,
    pub extension: Extension,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct AllNftInfoResponse {
    /// Who can transfer the token
    pub access: OwnerOfResponse,
    /// Data on the token itself,
    pub info: NftInfoResponse,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Cw721HookMsg {
    /// Who can transfer the token
    Stake {},
    /// Data on the token itself,
    OpenPack {},
}
