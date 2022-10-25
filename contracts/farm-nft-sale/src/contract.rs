#[cfg(not(feature = "library"))]
use cosmwasm_std::{
    attr, entry_point, to_binary, Addr, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo,
    QueryRequest, Response, StdError, StdResult, Storage, Uint128, WasmMsg, WasmQuery,
};

use crate::msg::{
    ExecuteMsg, InstantiateMsg, IsClaimedResponse, LatestStageResponse, MerkleRootResponse,
    MigrateMsg, QueryMsg, UpdateConfigMsg,
};
use crate::state::{Config, CLAIM, CONFIG, LATEST_STAGE, MERKLE_ROOT};
use cw2::set_contract_version;
use cw_storage_plus::U8Key;
use farm_nft::msg::{ExecuteMsg as NftExecuteMsg, QueryMsg as NftQueryMsg};
use sha2::Digest;
use std::convert::TryInto;

const CONTRACT_NAME: &str = "crates.io:loop-nft";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let config = Config {
        admin: info.sender.to_string(),
        ust_address: msg.ust_address,
        reserve_addr: msg.reserve_addr,
        pack_rate: msg.pack_rate,
        nft_contract_address: msg.nft_contract_address,
    };
    CONFIG.save(deps.storage, &config)?;
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::BuyPack { tool_type, stage, proof } => execute_buy_pack(deps, env, info, stage, proof, tool_type),
        ExecuteMsg::UpdateConfig(msg) => execute_update_config(deps, info, msg),
        ExecuteMsg::RegisterMerkleRoot { merkle_root } => {
            execute_register_merkle_root(deps, env, info, merkle_root)
        }
        // ExecuteMsg::Claim {
        //     stage,
        //     amount,
        //     proof,
        // } => execute_claim(deps, env, info, stage, amount, proof),
    }
}

pub fn execute_register_merkle_root(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    merkle_root: String,
) -> StdResult<Response> {
    let cfg = CONFIG.load(deps.storage)?;

    // if owner set validate, otherwise unauthorized
    if info.sender != cfg.admin {
        return Err(StdError::generic_err("Unauthorized"));
    }

    // check merkle root length
    let mut root_buf: [u8; 32] = [0; 32];
    hex::decode_to_slice(merkle_root.to_string(), &mut root_buf).unwrap();

    let stage = LATEST_STAGE.update(deps.storage, |stage| -> StdResult<_> { Ok(stage + 1) })?;

    MERKLE_ROOT.save(deps.storage, U8Key::from(stage), &merkle_root)?;
    LATEST_STAGE.save(deps.storage, &stage)?;

    Ok(Response::new().add_attributes(vec![
        attr("action", "register_merkle_root"),
        attr("stage", stage.to_string()),
        attr("merkle_root", merkle_root),
    ]))
}

pub fn claim(
    store: &mut dyn Storage,
    info: &MessageInfo,
    stage: u8,
    amount: &Uint128,
    proof: Vec<String>,
) -> StdResult<Response> {
    // verify not claimed
    let claimed = CLAIM.may_load(store, (&info.sender, U8Key::from(stage)))?;
    if claimed.is_some() {
        return Err(StdError::generic_err("Claimed"));
    }

    let merkle_root = MERKLE_ROOT.load(store, stage.into())?;

    let user_input = format!("{}{}", info.sender, amount);
    let hash = sha2::Sha256::digest(user_input.as_bytes())
        .as_slice()
        .try_into()
        .map_err(|_| StdError::generic_err("Wrong length"))
        .unwrap();

    let hash = proof.into_iter().try_fold(hash, |hash, p| {
        let mut proof_buf = [0; 32];
        hex::decode_to_slice(p, &mut proof_buf).unwrap();
        let mut hashes = [hash, proof_buf];
        hashes.sort_unstable();
        sha2::Sha256::digest(&hashes.concat())
            .as_slice()
            .try_into()
            .map_err(|_| StdError::generic_err("Wrong length"))
    })?;

    let mut root_buf: [u8; 32] = [0; 32];
    hex::decode_to_slice(merkle_root, &mut root_buf).unwrap();
    if root_buf != hash {
        return Err(StdError::generic_err("Verification Failed"));
    }

    // Update claim index to the current stage
    CLAIM.save(store, (&info.sender, stage.into()), &true)?;
    Ok(Response::new().add_attribute("action", "claimed"))
}

fn execute_update_config(
    deps: DepsMut,
    info: MessageInfo,
    msg: UpdateConfigMsg,
) -> StdResult<Response> {
    let mut config = CONFIG.load(deps.storage)?;
    if config.admin != info.sender {
        return Err(StdError::generic_err("Unauthorized"));
    }
    if msg.admin.is_some() {
        config.admin = msg.admin.unwrap();
    }
    if msg.ust_address.is_some() {
        config.ust_address = msg.ust_address.unwrap();
    }
    if msg.reserve_addr.is_some() {
        config.reserve_addr = msg.reserve_addr.unwrap();
    }
    if msg.pack_rate.is_some() {
        config.pack_rate = msg.pack_rate.unwrap();
    }
    if msg.nft_contract_address.is_some() {
        config.nft_contract_address = msg.nft_contract_address.unwrap();
    }
    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new())
}

///let user deposit tokens in exchange of dev tokens
pub fn execute_buy_pack(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    stage: u8,
    proof: Vec<String>,
    tool_type: String,
) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;
    assert_sent_native_token_balance(
        deps.storage,
        &info,
        stage,
        proof,
        config.pack_rate,
        config.ust_address,
    )?;

    let callback: CosmosMsg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.nft_contract_address,
        msg: to_binary(&NftExecuteMsg::TransferToolPack {
            recipient: info.sender.to_string(),
            tool_type: tool_type.to_string(),
        })?,
        funds: vec![],
    });

    let mut msg = String::from(tool_type.as_str());
    msg.push_str(" minted");
    Ok(Response::new()
        .add_message(callback)
        .add_attribute("action", msg.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::QueryRemainingAllPackCount {} => {
            to_binary(&query_remaining_all_pack_count(deps)?)
        }
        QueryMsg::QueryRemainingPackCount { tool_type } => {
            to_binary(&query_remaining_pack_count(deps, tool_type)?)
        }
        QueryMsg::MerkleRoot { stage } => to_binary(&query_merkle_root(deps, stage)?),
        QueryMsg::LatestStage {} => to_binary(&query_latest_stage(deps)?),
        QueryMsg::IsClaimed { stage, address } => {
            to_binary(&query_is_claimed(deps, stage, address)?)
        }
    }
}

pub fn query_remaining_all_pack_count(deps: Deps) -> StdResult<u64> {
    let config = CONFIG.load(deps.storage)?;
    let callback: u64 = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: config.nft_contract_address,
        msg: to_binary(&NftQueryMsg::QueryRemainingAllPackCount {})?,
    }))?;
    Ok(callback)
}

pub fn query_remaining_pack_count(deps: Deps, tool_type: String) -> StdResult<u64> {
    let config = CONFIG.load(deps.storage)?;
    let callback: u64 = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: config.nft_contract_address,
        msg: to_binary(&NftQueryMsg::QueryRemainingPackCount { tool_type })?,
    }))?;
    Ok(callback)
}

pub fn query_merkle_root(deps: Deps, stage: u8) -> StdResult<MerkleRootResponse> {
    let merkle_root = MERKLE_ROOT.load(deps.storage, U8Key::from(stage))?;
    let resp = MerkleRootResponse { stage, merkle_root };

    Ok(resp)
}

pub fn query_latest_stage(deps: Deps) -> StdResult<LatestStageResponse> {
    let latest_stage = LATEST_STAGE.load(deps.storage)?;
    let resp = LatestStageResponse { latest_stage };

    Ok(resp)
}

pub fn query_is_claimed(deps: Deps, stage: u8, address: String) -> StdResult<IsClaimedResponse> {
    let key: (&Addr, U8Key) = (&deps.api.addr_validate(&address)?, stage.into());
    let is_claimed = CLAIM.may_load(deps.storage, key)?.unwrap_or(false);
    let resp = IsClaimedResponse { is_claimed };

    Ok(resp)
}

pub fn assert_sent_native_token_balance(
    store: &mut dyn Storage,
    message_info: &MessageInfo,
    stage: u8,
    proof: Vec<String>,
    pack_rate: Uint128,
    ust_address: String,
) -> StdResult<Response> {
    let coin = message_info.funds.iter().find(|x| x.denom == ust_address);

    if coin.is_some() {
        if pack_rate == coin.unwrap().amount {
            Ok(Response::default()
            //     claim(
            //     store,
            //     message_info,
            //     stage,
            //     &coin.unwrap().amount,
            //     proof,
            // )?
        )
        } else {
            Err(StdError::generic_err("Please provide required coins"))
        }
    } else {
        Err(StdError::generic_err("Invalid Coins provided"))
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    Ok(Response::default())
}
