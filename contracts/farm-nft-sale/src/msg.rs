use cosmwasm_std::Uint128;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub admin: String,
    pub ust_address: String,
    pub reserve_addr: String,
    pub pack_rate: Uint128,
    pub nft_contract_address: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UpdateConfigMsg {
    pub admin: Option<String>,
    pub ust_address: Option<String>,
    pub reserve_addr: Option<String>,
    pub pack_rate: Option<Uint128>,
    pub nft_contract_address: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    BuyPack {
        tool_type: String,
        stage: u8,
        proof: Vec<String>,
    },
    UpdateConfig(UpdateConfigMsg),
    RegisterMerkleRoot {
        /// MerkleRoot is hex-encoded merkle root.
        merkle_root: String,
    },
    // Claim does not check if contract has enough funds, owner must ensure it.
    // Claim {
    //     stage: u8,
    //     amount: Uint128,
    //     /// Proof is hex-encoded merkle proof.
    //     proof: Vec<String>,
    // },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    QueryRemainingAllPackCount {},

    QueryRemainingPackCount { tool_type: String },
    MerkleRoot { stage: u8 },
    LatestStage {},
    IsClaimed { stage: u8, address: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MerkleRootResponse {
    pub stage: u8,
    /// MerkleRoot is hex-encoded merkle root.
    pub merkle_root: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct LatestStageResponse {
    pub latest_stage: u8,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct IsClaimedResponse {
    pub is_claimed: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}
