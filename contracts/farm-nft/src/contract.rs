#[cfg(not(feature = "library"))]
use cosmwasm_std::{
    entry_point, from_binary, to_binary, Binary, BlockInfo, CosmosMsg, Deps, DepsMut, Env,
    MessageInfo, Order, Response, StdError, StdResult, Storage, Uint128, WasmMsg,
};

use cw0::maybe_addr;
use cw2::set_contract_version;
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg};
use cw721::{
    ContractInfoResponse, Cw721ReceiveMsg, Expiration, NumTokensResponse, OwnerOfResponse,
    TokensResponse,
};
use std::collections::HashSet;
//use cw721_base::contract::{execute_send_nft, execute_transfer_nft};

//use cw721_base::ContractError; // TODO use custom errors instead
use cw_storage_plus::Bound;

use crate::error::ContractError;
use crate::msg::{
    AllNftInfoResponse, Cw721HookMsg, ExecuteMsg, Extension, InstantiateMsg, MintMsg,
    NftInfoResponse, QueryMsg,
};
use crate::state::{
    increment_tokens, num_tokens, tokens, Approval, Config, RewardToken, TokenInfo, CONFIG,
    CONTRACT_INFO, OPERATORS, REWARDS, REWARD_ITEMS, REWARD_TOKEN, TOKEN_COUNT, USER_STAKED_COUNT,
};

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
    };
    CONTRACT_INFO.save(deps.storage, &contract_info)?;
    CONFIG.save(deps.storage, &config)?;
    REWARD_ITEMS.save(deps.storage, &HashSet::<String>::new())?;
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
        //  ExecuteMsg::Boost(msg) => execute_boost(deps, env, info, msg),
        ExecuteMsg::TransferNft {
            recipient,
            token_id,
        } => execute_transfer_nft(deps, env, info, recipient, token_id),
        ExecuteMsg::Burn { token_id } => execute_burn(deps, env, info, token_id),
        ExecuteMsg::SendNft {
            contract,
            token_id,
            msg,
        } => execute_send_nft(deps, env, info, contract, token_id, msg),
        ExecuteMsg::ReceiveNft(msg) => execute_receive_cw721(deps, env, info, msg),
        ExecuteMsg::ClaimReward { token_id } => execute_claim_reward(deps, env, info, token_id),
        ExecuteMsg::Receive(msg) => execute_receive_cw20(deps, env, info, msg),
        ExecuteMsg::AddRewardToken {
            contract_addr,
            tool_name,
            mining_rate,
        } => execute_add_reward_token(deps, env, info, contract_addr, tool_name, mining_rate),
    }
}

pub fn execute_add_reward_token(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    contract_addr: String,
    tool_name: String,
    mining_rate: Uint128,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    if info.sender != config.minter {
        return Err(ContractError::Unauthorized {});
    }
    let reward_token = RewardToken {
        contract_addr: contract_addr.to_string(),
        mining_rate,
    };
    deps.api.addr_validate(contract_addr.as_str())?;
    REWARD_TOKEN.save(deps.storage, tool_name, &reward_token)?;
    Ok(Response::new().add_attribute("action", "distribution token added"))
}

pub fn execute_receive_cw20(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    let callback = CosmosMsg::Wasm(WasmMsg::Execute {
        //sending reward to user
        contract_addr: info.sender.to_string(),
        msg: to_binary(&Cw20ExecuteMsg::Burn { amount: msg.amount })?,
        funds: vec![],
    });
    let msg = MintMsg {
        owner: deps.api.addr_validate(&msg.sender)?,
        name: "Salman".to_string(),
        token_id: "50".to_string(),
        description: Some("".to_string()),
        image: "ipfs://QmVnu7JQVoDRqSgHBzraYp7Hy78HwJtLFi6nUFCowTGdzp/1.png".to_string(),
        rarity: "axe".to_string(),
    };

    mint(deps, &env, &msg);

    Ok(Response::new().add_message(callback))
}
pub fn execute_mint(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: MintMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    if info.sender != config.minter {
        return Err(ContractError::Unauthorized {});
    }

    // create the token
    mint(deps, &env, &msg);

    Ok(Response::new()
        .add_attribute("action", "mint")
        .add_attribute("minter", info.sender)
        .add_attribute("token_id", msg.token_id)
        .add_attribute("contract address", env.contract.address.into_string())
        .add_attribute("owner", msg.owner))
}

pub fn mint(deps: DepsMut, env: &Env, msg: &MintMsg) {
    let mut token = TokenInfo {
        owner: msg.owner.clone(),
        approvals: vec![],
        name: msg.name.to_string(),
        description: msg.description.clone().unwrap_or_default(),
        image: msg.image.to_string(),
        rarity: msg.rarity.to_string(),
        wait_time_for_nft_reward: env.block.time.seconds(),
        reward_start_time: 0,
        is_reward_token: false,
    };
    if msg.owner == env.contract.address {
        let mut token_ids = if let Some(token_ids) = REWARDS
            .may_load(deps.storage, msg.name.to_string())
            .unwrap()
        {
            token_ids
        } else {
            vec![]
        };
        token_ids.push(msg.token_id.to_string());
        let mut set = REWARD_ITEMS.load(deps.storage).unwrap();
        set.insert(msg.name.to_string());
        REWARDS
            .save(deps.storage, msg.name.to_string(), &token_ids)
            .unwrap();
        REWARD_ITEMS.save(deps.storage, &set).unwrap();
        token.is_reward_token = true;
    }
    tokens()
        .update(deps.storage, &msg.token_id, |old| match old {
            Some(_) => Err(ContractError::Claimed {}),
            None => Ok(token),
        })
        .unwrap();

    increment_tokens(deps.storage).unwrap();
}
pub fn execute_transfer_nft(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipient: String,
    token_id: String,
) -> Result<Response, ContractError> {
    _transfer_nft(deps, &env, &info, &recipient, &token_id)?;

    Ok(Response::new()
        .add_attribute("action", "transfer_nft")
        .add_attribute("sender", info.sender)
        .add_attribute("recipient", recipient)
        .add_attribute("token_id", token_id))
}

pub fn execute_send_nft(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    contract: String,
    token_id: String,
    msg: Binary,
) -> Result<Response, ContractError> {
    // Transfer token
    _transfer_nft(deps, &env, &info, &contract, &token_id)?;

    let send = Cw721ReceiveMsg {
        sender: info.sender.to_string(),
        token_id: token_id.clone(),
        msg,
    };

    // Send message
    Ok(Response::new()
        .add_message(send.into_cosmos_msg(contract.clone())?)
        .add_attribute("action", "send_nft")
        .add_attribute("sender", info.sender)
        .add_attribute("recipient", contract)
        .add_attribute("token_id", token_id))
}

pub fn execute_receive_cw721(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: Cw721ReceiveMsg,
) -> Result<Response, ContractError> {
    if env.contract.address != info.sender {
        return Err(ContractError::Unauthorized {});
    }
    match from_binary(&msg.msg) {
        Ok(Cw721HookMsg::Stake {}) => execute_stake(deps, env, msg),
        Ok(Cw721HookMsg::OpenPack {}) => execute_open_pack(deps, env, msg),
        Err(_err) => Err(ContractError::Unauthorized {}),
    }
}

pub fn execute_stake(
    deps: DepsMut,
    _env: Env,
    msg: Cw721ReceiveMsg,
) -> Result<Response, ContractError> {
    let token = tokens().load(deps.storage, &msg.token_id)?;
    
    // check that only pack can be opened, not any ohter nft from our contract
    if token.is_reward_token {
        return Err(ContractError::NotEligible {});
    }
   
    let count =
        if let Some(count) = USER_STAKED_COUNT.may_load(deps.storage, msg.sender.to_string())? {
            count
        } else {
            0u64
        };
    if count > 10 {
        return Err(ContractError::LimitReached {});
    }
    USER_STAKED_COUNT.save(deps.storage, msg.sender.to_string(), &(1 + count))?;

    //    execute_transfer_nft(deps, env, info, contract_addr, msg.token_id);
    // responses.push(CosmosMsg::Wasm(WasmMsg::Execute {
    //     contract_addr: contract_addr.clone(),
    //     msg: to_binary(&ExecuteMsg::TransferNft {
    //         recipient: contract_addr,
    //         token_id: msg.token_id,
    //     })?,
    //     funds: vec![],
    // }));
    Ok(Response::new())
}

pub fn execute_open_pack(
    deps: DepsMut,
    env: Env,
    msg: Cw721ReceiveMsg,
) -> Result<Response, ContractError> {
    let token = tokens().load(deps.storage, &msg.token_id)?;
    let mut responses: Vec<CosmosMsg> = vec![];
    // check that only pack can be opened, not any ohter nft from our contract
    if token.is_reward_token {
        return Err(ContractError::NotEligible {});
    }
    let set: HashSet<String> = REWARD_ITEMS.load(deps.storage)?;
    let contract_addr = env.contract.address.into_string();

    for name in set.iter() {
        let time_in_epoch_seconds = env.block.time.seconds();
        let mut token_ids =
            if let Some(token_ids) = REWARDS.may_load(deps.storage, name.to_string())? {
                token_ids
            } else {
                vec![]
            };
        let random_number = generate_random_number(time_in_epoch_seconds, token_ids.len() as u64);
        let token_id = token_ids.swap_remove(random_number as usize);
        REWARDS.save(deps.storage, name.to_string(), &token_ids)?;

        responses.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: contract_addr.clone(),
            msg: to_binary(&ExecuteMsg::TransferNft {
                recipient: msg.sender.to_string(),
                token_id,
            })?,
            funds: vec![],
        }));
    }
    responses.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr,
        msg: to_binary(&ExecuteMsg::Burn {
            token_id: msg.token_id,
        })?,
        funds: vec![],
    }));

    Ok(Response::new().add_messages(responses))
}

pub fn _transfer_nft(
    deps: DepsMut,
    env: &Env,
    info: &MessageInfo,
    recipient: &str,
    token_id: &str,
) -> Result<TokenInfo, ContractError> {
    let mut token = tokens().load(deps.storage, token_id)?;
    // ensure we have permissions
    check_can_send(deps.as_ref(), env, info, &token)?;
    // set owner and remove existing approvals
    token.owner = deps.api.addr_validate(recipient)?;
    token.approvals = vec![];
    tokens().save(deps.storage, token_id, &token)?;
    Ok(token)
}

pub fn execute_approve(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    spender: String,
    token_id: String,
    expires: Option<Expiration>,
) -> Result<Response, ContractError> {
    _update_approvals(deps, &env, &info, &spender, &token_id, true, expires)?;

    Ok(Response::new()
        .add_attribute("action", "approve")
        .add_attribute("sender", info.sender)
        .add_attribute("spender", spender)
        .add_attribute("token_id", token_id))
}

pub fn execute_claim_reward(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_id: String,
) -> Result<Response, ContractError> {
    let mut callback: Vec<CosmosMsg> = vec![];
    let token_info = if let Some(token_info) = tokens().may_load(deps.storage, &token_id)? {
        token_info
    } else {
        return Err(ContractError::NotFound {});
    };

    if token_info.owner != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    let reward_token = if let Some(reward_token) =
        REWARD_TOKEN.may_load(deps.storage, token_info.name.to_string())?
    {
        reward_token
    } else {
        return Err(ContractError::NoRewardTokenFound {});
    };

    if token_info.reward_start_time + token_info.wait_time_for_nft_reward < env.block.time.seconds()
    {
        callback.push(CosmosMsg::Wasm(WasmMsg::Execute {
            //sending reward to user
            contract_addr: reward_token.contract_addr.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: info.sender.to_string(),
                amount: Uint128::from(reward_token.mining_rate),
            })?,
            funds: vec![],
        }));
    }

    Ok(Response::new().add_messages(callback))
}
pub fn execute_revoke(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    spender: String,
    token_id: String,
) -> Result<Response, ContractError> {
    _update_approvals(deps, &env, &info, &spender, &token_id, false, None)?;

    Ok(Response::new()
        .add_attribute("action", "revoke")
        .add_attribute("sender", info.sender)
        .add_attribute("spender", spender)
        .add_attribute("token_id", token_id))
}

pub fn _update_approvals(
    deps: DepsMut,
    env: &Env,
    info: &MessageInfo,
    spender: &str,
    token_id: &str,
    // if add == false, remove. if add == true, remove then set with this expiration
    add: bool,
    expires: Option<Expiration>,
) -> Result<TokenInfo, ContractError> {
    let mut token = tokens().load(deps.storage, token_id)?;
    // ensure we have permissions
    check_can_approve(deps.as_ref(), env, info, &token)?;

    // update the approval list (remove any for the same spender before adding)
    let spender_addr = deps.api.addr_validate(spender)?;
    token.approvals = token
        .approvals
        .into_iter()
        .filter(|apr| apr.spender != spender_addr)
        .collect();

    // only difference between approve and revoke
    if add {
        // reject expired data as invalid
        let expires = expires.unwrap_or_default();
        if expires.is_expired(&env.block) {
            return Err(ContractError::Expired {});
        }
        let approval = Approval {
            spender: spender_addr,
            expires,
        };
        token.approvals.push(approval);
    }

    tokens().save(deps.storage, token_id, &token)?;

    Ok(token)
}
pub fn generate_random_number(time_in_epoch_seconds: u64, limit: u64) -> u64 {
    time_in_epoch_seconds % limit
}
pub fn execute_approve_all(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    operator: String,
    expires: Option<Expiration>,
) -> Result<Response, ContractError> {
    // reject expired data as invalid
    let expires = expires.unwrap_or_default();
    if expires.is_expired(&env.block) {
        return Err(ContractError::Expired {});
    }

    // set the operator for us
    let operator_addr = deps.api.addr_validate(&operator)?;
    OPERATORS.save(deps.storage, (&info.sender, &operator_addr), &expires)?;

    Ok(Response::new()
        .add_attribute("action", "approve_all")
        .add_attribute("sender", info.sender)
        .add_attribute("operator", operator))
}

pub fn execute_revoke_all(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    operator: String,
) -> Result<Response, ContractError> {
    let operator_addr = deps.api.addr_validate(&operator)?;
    OPERATORS.remove(deps.storage, (&info.sender, &operator_addr));

    Ok(Response::new()
        .add_attribute("action", "revoke_all")
        .add_attribute("sender", info.sender)
        .add_attribute("operator", operator))
}

/// returns true iff the sender can execute approve or reject on the contract
fn check_can_approve(
    deps: Deps,
    env: &Env,
    info: &MessageInfo,
    token: &TokenInfo,
) -> Result<(), ContractError> {
    // owner can approve
    if token.owner == info.sender {
        return Ok(());
    }
    // operator can approve
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

/// returns true iff the sender can transfer ownership of the token
fn check_can_send(
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

pub fn decrement_tokens(storage: &mut dyn Storage) -> StdResult<u64> {
    let val = num_tokens(storage)? - 1;
    TOKEN_COUNT.save(storage, &val)?;
    Ok(val)
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

    Ok(NftInfoResponse {
        token_uri: info.image.to_string(),
        extension: Extension {
            name: info.name,
            description: info.description,
            image: Some(info.image),
            rarity: info.rarity,
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
    Ok(AllNftInfoResponse {
        access: OwnerOfResponse {
            owner: info.owner.to_string(),
            approvals: humanize_approvals(&env.block, &info),
        },
        info: NftInfoResponse {
            token_uri: info.image.to_string(),
            extension: Extension {
                name: info.name,
                description: info.description,
                image: Some(info.image),
                rarity: info.rarity,
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

// #[cfg_attr(not(feature = "library"), entry_point)]
// pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
//     Ok(Response::default())
// }
