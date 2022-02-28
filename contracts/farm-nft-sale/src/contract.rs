#[cfg(not(feature = "library"))]
use cosmwasm_std::{
    entry_point, from_binary, to_binary, Binary,  CosmosMsg, Deps, DepsMut, Env,
    MessageInfo, Response,  StdResult, WasmMsg, QueryRequest,
    WasmQuery, StdError,
};

use cw2::set_contract_version;
use cw20::{Cw20ReceiveMsg};
use crate::msg::{
    Cw20HookMsg, ExecuteMsg, InstantiateMsg,
    MigrateMsg, QueryMsg, UpdateConfigMsg,
};
use crate::state::{
    Config, CONFIG,
};
use farm_nft::msg::{ExecuteMsg as NftExecuteMsg, QueryMsg as NftQueryMsg};

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
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    match msg {
        ExecuteMsg::Receive(msg) => execute_receive_cw20(deps, env, info, msg),
        ExecuteMsg::UpdateConfig(msg) => execute_update_config(deps, info, msg),
    }
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

/// receiving cw20 tokens
pub fn execute_receive_cw20(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: Cw20ReceiveMsg,
) -> StdResult<Response> {
    match from_binary(&msg.msg) {
        Ok(Cw20HookMsg::PackFood {}) => execute_buy_pack(deps, env, info, msg, "Food Miner".to_string()),
        Ok(Cw20HookMsg::PackGold {}) => execute_buy_pack(deps, env, info, msg, "Gold Miner".to_string()),
        Ok(Cw20HookMsg::PackStone {}) => execute_buy_pack(deps, env, info, msg, "Stone Miner".to_string()),
        Ok(Cw20HookMsg::PackWood {}) => execute_buy_pack(deps, env, info, msg, "Wood Miner".to_string()),
        Err(_err) => Err(StdError::generic_err("Unauthorized")),
    }
}

///let user deposit tokens in exchange of dev tokens
pub fn execute_buy_pack(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: Cw20ReceiveMsg,
    tool_type: String,
) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;
    if config.ust_address != info.sender {
        return Err(StdError::generic_err("Not eligible tokens provided"));
    }

    if msg.amount != config.pack_rate {
        return Err(StdError::generic_err("Kindly provided amount mentioned in pack rate"));
    }
    let callback: CosmosMsg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.nft_contract_address,
        msg: to_binary(&NftExecuteMsg::TransferToolPack {
            recipient: msg.sender,
            tool_type: tool_type.to_string(),
        })?,
        funds: vec![],
    });
    let mut msg = String::from(tool_type.as_str());
    msg.push_str(" minted");
    Ok(Response::new().add_message(callback).add_attribute("action", msg.to_string()))
}



#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::QueryRemainingAllPackCount{}
            => to_binary(&query_remaining_all_pack_count(deps)?),
        QueryMsg::QueryRemainingPackCount {
            tool_type
        } => to_binary(&query_remaining_pack_count(deps, tool_type)?),
    }
}

pub fn query_remaining_all_pack_count (
    deps: Deps,
) -> StdResult<u64> {
    let config = CONFIG.load(deps.storage)?;
    let callback: u64  = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: config.nft_contract_address,
        msg: to_binary(&NftQueryMsg::QueryRemainingAllPackCount{} 
        )?,
    }))?;
    Ok(callback)
}


pub fn query_remaining_pack_count (
    deps: Deps,
    tool_type: String,
) -> StdResult<u64> {
    let config = CONFIG.load(deps.storage)?;
    let callback: u64 = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: config.nft_contract_address,
        msg: to_binary(&NftQueryMsg::QueryRemainingPackCount {
            tool_type,
        })?,
    }))?;
    Ok(callback)
}


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    Ok(Response::default())
}
