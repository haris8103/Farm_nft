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

    pub team_addr: String,

    pub market_addr: String,

    pub legal_addr: String,

    pub burn_addr: String,

    pub stake_limit: u64,

    pub durability_from_start_time: u64,

    pub reserve_addr: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UpdateConfigMsg {
    pub team_addr: Option<String>,

    pub market_addr: Option<String>,

    pub legal_addr: Option<String>,

    pub burn_addr: Option<String>,

    pub stake_limit: Option<u64>,

    pub durability_from_start_time: Option<u64>,

    pub reserve_addr: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MintMsg {
    pub owner: Addr,
    /// Identifies the asset to which this NFT represents
    pub name: String,
    /// Custom extensions
    pub rarity: String,

    pub pre_mint_tool: Option<String>,

    pub minting_count: Option<u64>,

    pub tool_type: String, //common tool name
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

    Unstake {
        token_id: String,
    },

    Receive(Cw20ReceiveMsg),

    AddRewardToken {
        item_name: String,
        tool_name: String,
        mining_rate: u64,
        mining_waiting_time: u64,
    },
    AddToolTypeNames {
        tool_type: String,
    },
    BatchMint(MintMsg),
    AddItemToken {
        item_name: String,
        item_token_addr: String,
    },
    RefillEnergy {
        food_item_amount: u64,
    },
    Withdraw {
        item_name: String,
        amount: Uint128,
    },
    AddToolTemplate(ToolTemplateMsg),
    MintCommonNft {
        tool_type: String,
    },
    UpgradeNft {
        token_ids: Vec<String>,
    },

    UpdateConfig(UpdateConfigMsg),

    TransferReserveAmount {},

    TransferToolPack {
        recipient: String,
        tool_type: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// Return the owner of the given token, error if token does not exist
    /// Return type: OwnerOfResponse
    OwnerOf {
        token_id: String,
    },

    /// Total number of tokens issued
    NumTokens {},

    /// With MetaData Extension.
    /// Returns top-level metadata about the contract: `ContractInfoResponse`
    ContractInfo {},

    /// With MetaData Extension.
    /// Returns metadata about one particular token, based on *ERC721 Metadata JSON Schema*
    /// but directly from the contract: `NftInfoResponse`
    NftInfo {
        token_id: String,
    },

    /// With MetaData Extension.
    /// Returns the result of both `NftInfo` and `OwnerOf` as one query as an optimization
    /// for clients: `AllNftInfo`
    AllNftInfo {
        token_id: String,
    },

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

    UserStakedInfo {
        user_address: String,
    },
    UserItemBalance {
        user_address: String,
        item_name: String,
    },
    UserEnergyInfo {
        user_address: String,
    },
    UserItemInfo {
        user_address: String,
    },
    UserTokenBalance {
        user_address: String,
    },
    QueryRemainingAllPackCount {},
    QueryRemainingPackCount {
        tool_type: String,
    },

    QueryGameDevToken {},
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
    pub name: String,
    pub description: String,
    pub image: Option<String>,
    pub rarity: String,
    pub mining_rate: u64,
    pub mining_waiting_time: u64,
    pub owner: String,
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

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Cw20HookMsg {
    // RefillEnergy {},
    // MintAxe {},

    // MintFishNet {},

    // MintNft {},
    Deposit {},
    AdminDeposit {},
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub struct ToolTemplateMsg {
    pub tool_type: String,
    pub name: String,
    pub description: String,
    pub image: String,
    pub rarity: String,
    pub required_gwood_amount: Uint128,
    pub required_gfood_amount: Uint128,
    pub required_ggold_amount: Uint128,
    pub required_gstone_amount: Uint128,
    pub durability: u64,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}
