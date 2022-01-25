#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, BlockInfo, Deps, DepsMut, Env, MessageInfo, Order, Response, StdError,
    StdResult, Storage, WasmMsg, CosmosMsg,
};

use cw0::maybe_addr;
use cw2::set_contract_version;
use cw721::{
    ContractInfoResponse, NumTokensResponse, OwnerOfResponse, TokensResponse, Cw721ReceiveMsg
};

use cw721_base::contract::{execute_send_nft, execute_transfer_nft};
use cw721_base::state::{increment_tokens, num_tokens, tokens, Approval, TokenInfo, CONTRACT_INFO, TOKEN_COUNT, OPERATORS};
use cw721_base::ContractError; // TODO use custom errors instead
use cw_storage_plus::Bound;

use crate::msg::{
    AllNftInfoResponse, BoostMsg, ExecuteMsg, Extension, InstantiateMsg, MintMsg, NftInfoResponse,
    QueryMsg,
};

use crate::state::{Config, CONFIG, LEVEL_DATA};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:loop-nft";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// used for limiting queries
const DEFAULT_LIMIT: u32 = 10;
const MAX_LIMIT: u32 = 30;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let contract_info = ContractInfoResponse {
        name: msg.name,
        symbol: msg.symbol,
    };

    let config = Config {
        minter: _info.sender.to_string(),
        cw721_address: "".to_string(),
    };
    CONTRACT_INFO.save(deps.storage, &contract_info)?;
    CONFIG.save(deps.storage, &config)?;
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Mint(msg) => execute_mint(deps, env, info, msg),
        ExecuteMsg::Boost(msg) => execute_boost(deps, env, info, msg),
        ExecuteMsg::TransferNft {
            recipient,
            token_id,
        } => execute_transfer_nft(deps, env, info, recipient, token_id),
        ExecuteMsg::Burn {
            token_id,
        } => execute_burn(deps, env, info,token_id),
        ExecuteMsg::SendNft {
            contract,
            token_id,
            msg,
        } => execute_send_nft(deps, env, info, contract, token_id, msg),
        ExecuteMsg::ReceiveNft (  msg ) => {
            execute_receive(deps, env, info, msg)
        }
    }
}

fn execute_burn(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_id: String,
) -> Result<Response, ContractError> {
    let token = tokens().load(deps.storage, &token_id)?;
    _check_can_send(deps.as_ref(), &env, &info, &token)?;

    tokens().remove(deps.storage, &token_id)?;
    decrement_tokens(deps.storage)?;

    Ok(Response::new()
        .add_attribute("action", "burn")
        .add_attribute("sender", info.sender)
        .add_attribute("token_id", token_id))
}

pub fn decrement_tokens( storage: &mut dyn Storage) -> StdResult<u64> {
    let val = num_tokens(storage)? - 1;
    TOKEN_COUNT.save(storage, &val)?;
    Ok(val)
}

pub fn execute_receive(
    _deps: DepsMut,
    env: Env, 
    info: MessageInfo,
    msg: Cw721ReceiveMsg,
) -> Result<Response, ContractError> {
    if env.contract.address != info.sender {
        return Err(ContractError::Unauthorized {});
    }
    //let mut messages: Vec<CosmosMsg> = Vec::new();
    //let config = CONFIG.load(deps.storage)?;
   // let mut token = tokens().load(deps.storage, &msg.token_id.to_string())?;
    let contract_addr = env.contract.address.into_string();
    

    //execute_transfer_nft(deps, env, info, sender, "2".to_string())?;

    let callback = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: contract_addr.clone(),
        msg: to_binary(&ExecuteMsg::TransferNft {
            recipient: msg.sender.to_string(),
            token_id: "2".to_string(),
        })?,
        funds: vec![],
    });
 
   // execute_burn(deps, env, info.clone(), msg.token_id.to_string())?;

    let callback2 = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: contract_addr,
        msg: to_binary(&ExecuteMsg::Burn {
            token_id: msg.token_id,
        })?,
        funds: vec![],
    });

    // let mint_msg = Cw721ExecuteMsg::Mint(MintMsg::<Extension> {
    //     token_id: config.unused_token_id.to_string(),
    //     owner: sender,
    //     token_uri: config.token_uri.clone().into(),
    //     extension: config.extension.clone(),
    // });
    Ok(Response::new().add_message(callback).add_message(callback2))
}

pub fn _check_can_send(
    deps: Deps,
    env: &Env,
    info: &MessageInfo,
    token: &TokenInfo,
) -> Result<(), ContractError> {
    // owner can send
    if token.owner == info.sender {
        return Ok(());
    }

    // any non-expired token approval can send
    if token
        .approvals
        .iter()
        .any(|apr| apr.spender == info.sender && !apr.is_expired(&env.block))
    {
        return Ok(());
    }

    // operator can send
    let op = OPERATORS.may_load(deps.storage, (&token.owner, &info.sender))?;
    match op {
        Some(ex) => {
            if ex.is_expired(&env.block) {
                Err(ContractError::Unauthorized {})
            } else {
                Ok(())
            }
        }
        None => Err(ContractError::Unauthorized {}),
    }
}

pub fn execute_mint(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: MintMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    if info.sender != config.minter {
        return Err(ContractError::Std(StdError::generic_err("Unauthorized")));
    }

    // create the token
    let token = TokenInfo {
        name: msg.name.clone(),
        description: "".to_string(),
        image: Some(msg.image.clone()),
        owner: msg.owner,
        approvals: vec![],
    };
    tokens().update(deps.storage, &msg.token_id, |old| match old {
        Some(_) => Err(ContractError::Claimed {}),
        None => Ok(token),
    })?;

    // update tokens count
    increment_tokens(deps.storage)?;

    LEVEL_DATA.save(deps.storage, &msg.token_id, &0u16)?;

    Ok(Response::new()
        .add_attribute("action", "mint")
        .add_attribute("token_id", msg.token_id)
        .add_attribute("name", msg.name)
        .add_attribute("owner", info.sender.to_string()))
}

pub fn execute_boost(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: BoostMsg,
) -> Result<Response, ContractError> {
    if msg.token_ids.len() != 5 {
        return Err(ContractError::Std(StdError::generic_err(
            "Need 5 nfts to boost",
        )));
    }
    let level = LEVEL_DATA
        .may_load(deps.storage, &msg.token_ids[0])?
        .unwrap();

    for token_id in msg.token_ids.iter() {
        let data = LEVEL_DATA.may_load(deps.storage, token_id)?.unwrap();
        if data != level {
            return Err(ContractError::Std(StdError::generic_err(
                "Need 5 nfts to be on the same level",
            )));
        }
    }

    for token_id in msg.token_ids.iter() {
        tokens().remove(deps.storage, token_id)?;
    }

    // create the token
    let token = TokenInfo {
        name: msg.name.clone(),
        description: "".to_string(),
        image: Some(msg.image.clone()),
        owner: info.sender.clone(),
        approvals: vec![],
    };
    tokens().update(deps.storage, &msg.token_id, |old| match old {
        Some(_) => Err(ContractError::Claimed {}),
        None => Ok(token),
    })?;

    // update tokens count
    increment_tokens(deps.storage)?;

    LEVEL_DATA.save(deps.storage, &msg.token_id, &(level + 1))?;

    Ok(Response::new()
        .add_attribute("action", "boost")
        .add_attribute("token_id", msg.token_id)
        .add_attribute("name", msg.name)
        .add_attribute("owner", info.sender.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::ContractInfo {} => to_binary(&query_contract_info(deps)?),
        QueryMsg::NftInfo { token_id } => to_binary(&query_nft_info(deps, token_id)?),
        QueryMsg::OwnerOf { token_id } => to_binary(&query_owner_of(deps, env, token_id)?),
        QueryMsg::AllNftInfo { token_id } => to_binary(&query_all_nft_info(deps, env, token_id)?),
        QueryMsg::NumTokens {} => to_binary(&query_num_tokens(deps)?),
        QueryMsg::Tokens {
            owner,
            start_after,
            limit,
        } => to_binary(&query_tokens(deps, owner, start_after, limit)?),
        QueryMsg::AllTokens { start_after, limit } => {
            to_binary(&query_all_tokens(deps, start_after, limit)?)
        }
    }
}

fn query_contract_info(deps: Deps) -> StdResult<ContractInfoResponse> {
    CONTRACT_INFO.load(deps.storage)
}

fn query_nft_info(deps: Deps, token_id: String) -> StdResult<NftInfoResponse> {
    let info = tokens().load(deps.storage, &token_id)?;
    let level = LEVEL_DATA.may_load(deps.storage, &token_id)?.unwrap();
    Ok(NftInfoResponse {
        token_uri: info.image.as_ref().unwrap().to_string(),
        extension: Extension {
            name: info.name,
            description: info.description,
            image: info.image,
            level,
        },
    })
}

fn query_owner_of(deps: Deps, env: Env, token_id: String) -> StdResult<OwnerOfResponse> {
    let info = tokens().load(deps.storage, &token_id)?;
    Ok(OwnerOfResponse {
        owner: info.owner.to_string(),
        approvals: humanize_approvals(&env.block, &info),
    })
}

fn query_all_nft_info(deps: Deps, env: Env, token_id: String) -> StdResult<AllNftInfoResponse> {
    let info = tokens().load(deps.storage, &token_id)?;
    let level = LEVEL_DATA.may_load(deps.storage, &token_id)?.unwrap();
    Ok(AllNftInfoResponse {
        access: OwnerOfResponse {
            owner: info.owner.to_string(),
            approvals: humanize_approvals(&env.block, &info),
        },
        info: NftInfoResponse {
            token_uri: info.image.as_ref().unwrap().to_string(),
            extension: Extension {
                name: info.name,
                description: info.description,
                image: info.image,
                level,
            },
        },
    })
}

fn humanize_approvals(block: &BlockInfo, info: &TokenInfo) -> Vec<cw721::Approval> {
    info.approvals
        .iter()
        .filter(|apr| !apr.is_expired(block))
        .map(humanize_approval)
        .collect()
}

fn humanize_approval(approval: &Approval) -> cw721::Approval {
    cw721::Approval {
        spender: approval.spender.to_string(),
        expires: approval.expires,
    }
}

fn query_num_tokens(deps: Deps) -> StdResult<NumTokensResponse> {
    let count = num_tokens(deps.storage)?;
    Ok(NumTokensResponse { count })
}

fn query_tokens(
    deps: Deps,
    owner: String,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<TokensResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = start_after.map(Bound::exclusive);

    let owner_addr = deps.api.addr_validate(&owner)?;
    let pks: Vec<_> = tokens()
        .idx
        .owner
        .prefix(owner_addr)
        .keys(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .collect();

    let res: Result<Vec<_>, _> = pks.iter().map(|v| String::from_utf8(v.to_vec())).collect();
    let tokens = res.map_err(StdError::invalid_utf8)?;
    Ok(TokensResponse { tokens })
}

fn query_all_tokens(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<TokensResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start_addr = maybe_addr(deps.api, start_after)?;
    let start = start_addr.map(|addr| Bound::exclusive(addr.as_ref()));

    let tokens: StdResult<Vec<String>> = tokens()
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| item.map(|(k, _)| String::from_utf8_lossy(&k).to_string()))
        .collect();
    Ok(TokensResponse { tokens: tokens? })
}
