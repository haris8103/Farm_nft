#[cfg(not(feature = "library"))]
use cosmwasm_std::{
    entry_point, from_binary, to_binary, Binary, BlockInfo, CosmosMsg, Deps, DepsMut, Env,
    MessageInfo, Order, QueryRequest, Response, StdError, StdResult, Storage, Uint128, WasmMsg,
    WasmQuery,
};
use cw0::maybe_addr;
use cw2::set_contract_version;
use cw20::{BalanceResponse as Cw20BalanceResponse, Cw20ExecuteMsg, Cw20QueryMsg, Cw20ReceiveMsg};
use cw721::{
    ContractInfoResponse, Cw721ReceiveMsg, Expiration, NumTokensResponse, OwnerOfResponse,
    TokensResponse,
};
use cw_storage_plus::Bound;
use std::collections::HashSet;

use crate::error::ContractError;
use crate::mint::{execute_mint_common_nft, execute_mint_upgraded_nft};
use crate::msg::{
    AllNftInfoResponse, Cw20HookMsg, Cw721HookMsg, ExecuteMsg, Extension, InstantiateMsg,
    MigrateMsg, MintMsg, NftInfoResponse, QueryMsg, ToolTemplateMsg, UpdateConfigMsg,
};
use crate::state::{
    distribute_amount, increment_tokens, num_tokens, tokens, Approval, Config, RewardToken,
    TokenInfo, ToolTemplate, CONFIG, CONTRACT_INFO, GAME_DEV_TOKENS_NAME, ITEM_TOKEN_MAPPING,
    LAST_GEN_TOKEN_ID, OPERATORS, RARITY_TYPES, REWARD_TOKEN, TOKEN_COUNT, TOKEN_ITEM_MAPPING,
    TOOL_SET_MAP, TOOL_TEMPLATE_MAP, TOOL_TYPE_NAMES, USER_ENERGY_LEVEL, USER_ITEM_AMOUNT,
    USER_STAKED_INFO,
};

const CONTRACT_NAME: &str = "crates.io:loop-nft";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const DEFAULT_LIMIT: u32 = 10;
const MAX_LIMIT: u32 = 30;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
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
        team_addr: msg.team_addr,
        market_addr: msg.market_addr,
        legal_addr: msg.legal_addr,
        burn_addr: msg.burn_addr,
        stake_limit: msg.stake_limit,
        durability_start_time: env.block.time.seconds() + msg.durability_from_start_time,
        reserve_addr: msg.reserve_addr,
    };

    CONTRACT_INFO.save(deps.storage, &contract_info)?;
    CONFIG.save(deps.storage, &config)?;
    TOOL_TYPE_NAMES.save(deps.storage, &vec![])?;
    LAST_GEN_TOKEN_ID.save(deps.storage, &0u64)?;
    RARITY_TYPES.save(deps.storage, "Common".to_string(), &"Uncommon".to_string())?;
    RARITY_TYPES.save(deps.storage, "Uncommon".to_string(), &"Rare".to_string())?;
    RARITY_TYPES.save(deps.storage, "Rare".to_string(), &"Legendary".to_string())?;
    RARITY_TYPES.save(deps.storage, "Legendary".to_string(), &"Mythic".to_string())?;
    let mut game_dev_token_set = HashSet::<String>::new();
    game_dev_token_set.insert("gWood".to_string());
    game_dev_token_set.insert("gFood".to_string());
    game_dev_token_set.insert("gGold".to_string());
    game_dev_token_set.insert("gStone".to_string());
    GAME_DEV_TOKENS_NAME.save(deps.storage, &game_dev_token_set)?;
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
        ExecuteMsg::Unstake { token_id } => execute_unstake(deps, env, info, token_id),
        ExecuteMsg::Receive(msg) => execute_receive_cw20(deps, env, info, msg),
        ExecuteMsg::AddRewardToken {
            item_name,
            tool_name,
            mining_rate,
            mining_waiting_time,
        } => execute_add_reward_token(
            deps,
            env,
            info,
            item_name,
            tool_name,
            mining_rate,
            mining_waiting_time,
        ),
        ExecuteMsg::AddToolTypeNames { tool_type } => {
            execute_add_tool_type_names(deps, env, info, tool_type)
        }
        ExecuteMsg::BatchMint(msg) => execute_batch_mint(deps, env, info, msg),
        ExecuteMsg::AddItemToken {
            item_name,
            item_token_addr,
        } => execute_add_item_token(deps, env, info, item_name, item_token_addr),
        ExecuteMsg::RefillEnergy { food_item_amount } => {
            execute_refill_energy(deps, env, info, food_item_amount)
        }

        ExecuteMsg::Withdraw { item_name, amount } => {
            execute_withdraw(deps, env, info, item_name, amount)
        }
        ExecuteMsg::AddToolTemplate(msg) => execute_add_tool_template(deps, env, info, msg),
        ExecuteMsg::MintCommonNft { tool_type } => {
            execute_mint_common_nft(deps, env, info, tool_type)
        }

        ExecuteMsg::UpgradeNft { token_ids } => {
            execute_mint_upgraded_nft(deps, env, info, token_ids)
        }
        ExecuteMsg::UpdateConfig(msg) => execute_update_config(deps, info, msg),

        ExecuteMsg::TransferReserveAmount {} => execute_transfer_reserve_amount(deps, info, env),
    }
}

fn execute_update_config(
    deps: DepsMut,
    info: MessageInfo,
    msg: UpdateConfigMsg,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    if config.minter != info.sender {
        return Err(ContractError::Unauthorized {});
    }
    if msg.team_addr.is_some() {
        config.team_addr = msg.team_addr.unwrap();
    }
    if msg.market_addr.is_some() {
        config.market_addr = msg.market_addr.unwrap();
    }
    if msg.legal_addr.is_some() {
        config.legal_addr = msg.legal_addr.unwrap();
    }
    if msg.burn_addr.is_some() {
        config.burn_addr = msg.burn_addr.unwrap();
    }
    if msg.stake_limit.is_some() {
        config.stake_limit = msg.stake_limit.unwrap();
    }
    if msg.durability_from_start_time.is_some() {
        config.durability_start_time += msg.durability_from_start_time.unwrap();
    }
    if msg.reserve_addr.is_some() {
        config.reserve_addr = msg.reserve_addr.unwrap();
    }
    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new())
}

/// to transfer reserve amount of contract pool to withdraw
fn execute_transfer_reserve_amount(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if config.minter != info.sender {
        return Err(ContractError::Unauthorized {});
    }
    let mut responses: Vec<CosmosMsg> = vec![];
    let game_dev_token_set = GAME_DEV_TOKENS_NAME.load(deps.storage)?;
    // iterating over dev token to get the reserve amount from all of them
    for game_dev_token_name in game_dev_token_set {
        let mut user_address = env.contract.address.to_string();
        user_address.push_str(&game_dev_token_name.to_string());
        if let Some(contract_pool_amount) =
            USER_ITEM_AMOUNT.may_load(deps.storage, user_address.to_string())?
        {
            let token_addr = if let Some(token_addr) =
                ITEM_TOKEN_MAPPING.may_load(deps.storage, game_dev_token_name.to_string())?
            {
                token_addr
            } else {
                return Err(ContractError::NotFound {});
            };
            // transfering contract pool to reserve addr
            responses.push(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: token_addr,
                msg: to_binary(&Cw20ExecuteMsg::Transfer {
                    recipient: config.reserve_addr.to_string(),
                    amount: contract_pool_amount,
                })
                .unwrap(),
                funds: vec![],
            }));
            contract_pool_amount
        } else {
            Uint128::zero()
        };
        //updating amount in map
        USER_ITEM_AMOUNT.save(
            deps.storage,
            game_dev_token_name.to_string(),
            &Uint128::zero(),
        )?;
    }
    Ok(Response::new().add_messages(responses))
}

/// adding tool template/snapshot in the contract
fn execute_add_tool_template(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ToolTemplateMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.minter {
        return Err(ContractError::Unauthorized {});
    }
    let mut tool_template = ToolTemplate {
        name: msg.name,
        description: msg.description,
        image: msg.image,
        rarity: msg.rarity.to_string(),
        required_amount: vec![],
        durability: msg.durability,
    };

    tool_template
        .required_amount
        .push(msg.required_gwood_amount);
    tool_template
        .required_amount
        .push(msg.required_gfood_amount);
    tool_template
        .required_amount
        .push(msg.required_ggold_amount);
    tool_template
        .required_amount
        .push(msg.required_gstone_amount);

    let mut template_key = msg.tool_type;
    template_key.push_str(&msg.rarity);
    TOOL_TEMPLATE_MAP.save(deps.storage, template_key, &tool_template)?;
    Ok(Response::default())
}

/// to withdraw tokens in exchange of game dev tokens
pub fn execute_withdraw(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    item_name: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let mut user_item_key = info.sender.to_string();

    user_item_key.push_str(&item_name);

    let mut user_item_amount = if let Some(user_item_amount) =
        USER_ITEM_AMOUNT.may_load(deps.storage, user_item_key.to_string())?
    {
        user_item_amount
    } else {
        Uint128::zero()
    };
    if user_item_amount < amount {
        return Err(ContractError::InSufficientFunds {});
    }
    let token_addr =
        if let Some(token_addr) = ITEM_TOKEN_MAPPING.may_load(deps.storage, item_name)? {
            token_addr
        } else {
            return Err(ContractError::NotFound {});
        };
    //transfering tokens to user
    let response = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: token_addr,
        msg: to_binary(&Cw20ExecuteMsg::Transfer {
            recipient: info.sender.to_string(),
            amount,
        })
        .unwrap(),
        funds: vec![],
    });
    user_item_amount -= amount;
    USER_ITEM_AMOUNT.save(deps.storage, user_item_key, &user_item_amount)?;
    Ok(Response::default().add_message(response))
}
///adding game dev token against tokns or vice versa e.g. gWood -> some address
pub fn execute_add_item_token(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    item_name: String,
    item_token_addr: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.minter {
        return Err(ContractError::Unauthorized {});
    }
    TOKEN_ITEM_MAPPING.save(deps.storage, item_token_addr.to_string(), &item_name)?;
    ITEM_TOKEN_MAPPING.save(deps.storage, item_name, &item_token_addr)?;
    Ok(Response::default())
}

///adding reward token
pub fn execute_add_reward_token(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    item_name: String,
    tool_name: String,
    mining_rate: u64,
    mining_waiting_time: u64,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    if info.sender != config.minter {
        return Err(ContractError::Unauthorized {});
    }

    let reward_token = RewardToken {
        item_name,
        mining_rate,
        mining_waiting_time,
    };

    REWARD_TOKEN.save(deps.storage, tool_name, &reward_token)?;
    Ok(Response::new().add_attribute("action", "distribution token added"))
}

/// adding tool type name
pub fn execute_add_tool_type_names(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    tool_type: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    if info.sender != config.minter {
        return Err(ContractError::Unauthorized {});
    }
    let mut tool_type_names = TOOL_TYPE_NAMES.may_load(deps.storage)?.unwrap();
    for i in tool_type_names.iter() {
        if *i == tool_type {
            return Err(ContractError::AlreadyExisits {});
        }
    }

    tool_type_names.push(tool_type);
    TOOL_TYPE_NAMES.save(deps.storage, &tool_type_names)?;
    Ok(Response::new().add_attribute("action", "common name added"))
}

/// receiving cw20 tokens
pub fn execute_receive_cw20(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    match from_binary(&msg.msg) {
        Ok(Cw20HookMsg::Deposit {}) => execute_deposit(deps, env, info, msg),
        Ok(Cw20HookMsg::AdminDeposit {}) => execute_admin_deposit(deps, env, info, msg),
        Err(_err) => Err(ContractError::Unauthorized {}),
    }
}

///let user deposit tokens in exchange of dev tokens
pub fn execute_deposit(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    let item_name = if let Some(item_name) =
        TOKEN_ITEM_MAPPING.may_load(deps.storage, info.sender.to_string())?
    {
        item_name
    } else {
        return Err(ContractError::NotFound {});
    };
    let mut user_item_key = msg.sender.to_string();
    user_item_key.push_str(&item_name);
    let mut user_item_amount = if let Some(user_item_amount) =
        USER_ITEM_AMOUNT.may_load(deps.storage, user_item_key.to_string())?
    {
        user_item_amount
    } else {
        Uint128::zero()
    };
    user_item_amount += msg.amount;
    USER_ITEM_AMOUNT.save(deps.storage, user_item_key, &user_item_amount)?;
    Ok(Response::new())
}

/// let admin deposit tokens
pub fn execute_admin_deposit(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if config.minter != msg.sender {
        return Err(ContractError::Unauthorized {});
    }
    let item_name = if let Some(item_name) =
        TOKEN_ITEM_MAPPING.may_load(deps.storage, info.sender.to_string())?
    {
        item_name
    } else {
        return Err(ContractError::NotFound {});
    };
    let mut contract_item_key = env.contract.address.to_string();
    contract_item_key.push_str(&item_name);
    let mut contract_item_amount = if let Some(contract_item_amount) =
        USER_ITEM_AMOUNT.may_load(deps.storage, contract_item_key.to_string())?
    {
        contract_item_amount
    } else {
        Uint128::zero()
    };
    contract_item_amount += msg.amount;
    USER_ITEM_AMOUNT.save(deps.storage, contract_item_key, &contract_item_amount)?;
    Ok(Response::new())
}

///let user refill energy to execute claiming reward transaction
pub fn execute_refill_energy(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: u64,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    let mut user_energy_level = if let Some(user_energy_level) =
        USER_ENERGY_LEVEL.may_load(deps.storage, info.sender.to_string())?
    {
        user_energy_level
    } else {
        Uint128::zero()
    };

    let mut user_item_key = info.sender.to_string();
    user_item_key.push_str("gFood");
    let mut user_item_amount =
        if let Some(user_item_amount) = USER_ITEM_AMOUNT.may_load(deps.storage, user_item_key)? {
            user_item_amount
        } else {
            Uint128::zero()
        };
    let amount = Uint128::from(amount);
    if user_item_amount < amount {
        return Err(ContractError::InSufficientFunds {});
    }
    user_energy_level += amount.multiply_ratio(Uint128::from(3u128), Uint128::from(1u128));

    USER_ENERGY_LEVEL.save(deps.storage, info.sender.to_string(), &user_energy_level)?;
    user_item_amount -= amount;
    USER_ITEM_AMOUNT.save(deps.storage, info.sender.to_string(), &user_item_amount)?;
    distribute_amount(deps.storage, "gFood".to_string(), amount, &config, &env);
    Ok(Response::new())
}

/// to mint multiple nfts in a single transaction
pub fn execute_batch_mint(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: MintMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    if info.sender != config.minter {
        return Err(ContractError::Unauthorized {});
    }
    let mut number = 0u64;
    let mut token_ids: String = String::new();
    // create the token
    while number < msg.minting_count.unwrap() {
        token_ids.push_str(mint(deps.storage, &env, &msg).to_string().as_str());
        token_ids.push_str(" ,");
        number += 1;
    }

    Ok(Response::new()
        .add_attribute("action", "mint")
        .add_attribute("minter", info.sender)
        .add_attribute("token_ids", token_ids.as_str())
        .add_attribute("contract address", env.contract.address.into_string())
        .add_attribute("owner", msg.owner))
}

/// mint a nsft
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
    let token_id = mint(deps.storage, &env, &msg);

    Ok(Response::new()
        .add_attribute("action", "mint")
        .add_attribute("minter", info.sender)
        .add_attribute("token_id", token_id.to_string())
        .add_attribute("contract address", env.contract.address.into_string())
        .add_attribute("owner", msg.owner))
}

///minitng functionality
pub fn mint(store: &mut dyn Storage, env: &Env, msg: &MintMsg) -> u64 {
    let mut template_key = msg.tool_type.to_string();
    template_key.push_str(msg.rarity.to_string().as_str());
    let tool_template = TOOL_TEMPLATE_MAP
        .load(store, template_key.to_string())
        .unwrap();

    let mut token = TokenInfo {
        name: msg.name.to_string(),
        owner: msg.owner.clone(),
        approvals: vec![],
        rarity: msg.rarity.to_string(),
        reward_start_time: env.block.time.seconds(),
        is_pack_token: true,
        pre_mint_tool: msg.pre_mint_tool.clone().unwrap_or_else(|| "".to_string()),
        tool_type: msg.tool_type.to_string(),
        durability: tool_template.durability,
    };
    increment_tokens(store).unwrap();
    let last_gen_token_id = LAST_GEN_TOKEN_ID.load(store).unwrap();
    let new_toke_id = last_gen_token_id + 1;
    //save last generated token id
    LAST_GEN_TOKEN_ID.save(store, &new_toke_id).unwrap();

    //if contract is the owner than nft it means user has opened a pack
    if msg.owner == env.contract.address {
        let mut token_ids = if let Some(token_ids) = TOOL_SET_MAP
            .may_load(store, msg.tool_type.to_string())
            .unwrap()
        {
            token_ids
        } else {
            vec![]
        };
        token_ids.push(new_toke_id.to_string());
        //saving in tool set map
        TOOL_SET_MAP
            .save(store, msg.tool_type.to_string(), &token_ids)
            .unwrap();
        token.is_pack_token = false;
    }
    tokens()
        .update(store, &new_toke_id.to_string(), |old| match old {
            Some(_) => Err(ContractError::Claimed {}),
            None => Ok(token),
        })
        .unwrap();
    new_toke_id
}

/// transfering nft to other stakeholder
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

/// sending token
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
        .add_message(send.into_cosmos_msg(contract.clone())?) // calling receiving cw721 functionalithy
        .add_attribute("action", "send_nft")
        .add_attribute("sender", info.sender)
        .add_attribute("recipient", contract)
        .add_attribute("token_id", token_id))
}

/// receive cw721
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

/// staking nft for earning reward
pub fn execute_stake(
    deps: DepsMut,
    env: Env,
    msg: Cw721ReceiveMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let mut token = tokens().load(deps.storage, &msg.token_id)?;

    // check that only pack can be opened, not any ohter nft from our contract
    if token.is_pack_token {
        return Err(ContractError::NotEligible {});
    }
    // if user is staking first time than the user will get 200 energy
    if USER_ENERGY_LEVEL
        .may_load(deps.storage, msg.sender.to_string())?
        .is_none()
    {
        USER_ENERGY_LEVEL.save(
            deps.storage,
            msg.sender.to_string(),
            &Uint128::from(200u128),
        )?;
    }

    let mut stake_info = if let Some(stake_info) =
        USER_STAKED_INFO.may_load(deps.storage, msg.sender.to_string())?
    {
        stake_info
    } else {
        HashSet::<String>::new()
    };

    if stake_info.len() as u64 > config.stake_limit {
        return Err(ContractError::LimitReached {});
    }

    token.reward_start_time = env.block.time.seconds();
    stake_info.insert(msg.token_id.to_string());
    USER_STAKED_INFO.save(deps.storage, msg.sender, &stake_info)?;
    tokens().save(deps.storage, &msg.token_id, &token)?;
    Ok(Response::new())
}

/// unstaking nft
pub fn execute_unstake(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_id: String,
) -> Result<Response, ContractError> {
    let mut responses: Vec<CosmosMsg> = vec![];
    let mut stake_ids_set = if let Some(stake_ids_set) =
        USER_STAKED_INFO.may_load(deps.storage, info.sender.to_string())?
    {
        stake_ids_set
    } else {
        return Err(ContractError::NotFound {});
    };

    if !stake_ids_set.contains(&token_id.to_string()) {
        return Err(ContractError::NotFound {});
    }
    let token_info = tokens().load(deps.storage, &token_id)?;
    let reward_info = REWARD_TOKEN.load(deps.storage, token_info.name.to_string())?;
    if token_info.reward_start_time + reward_info.mining_waiting_time > env.block.time.seconds() {
        return Err(ContractError::TimeNotReached {});
    }
    // transfer it back to user
    responses.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: env.contract.address.into_string(),
        msg: to_binary(&ExecuteMsg::TransferNft {
            recipient: info.sender.to_string(),
            token_id: token_id.to_string(),
        })?,
        funds: vec![],
    }));
    stake_ids_set.remove(&token_id);
    USER_STAKED_INFO.save(deps.storage, info.sender.to_string(), &stake_ids_set)?;
    Ok(Response::new().add_messages(responses))
}

///let user open pack
pub fn execute_open_pack(
    deps: DepsMut,
    env: Env,
    msg: Cw721ReceiveMsg,
) -> Result<Response, ContractError> {
    let token = tokens().load(deps.storage, &msg.token_id)?;
    let mut responses: Vec<CosmosMsg> = vec![];
    // check that only pack can be opened, not any other nft from our contract
    if !token.is_pack_token {
        return Err(ContractError::NotEligible {});
    }

    let tool_types: Vec<String> = TOOL_TYPE_NAMES.may_load(deps.storage)?.unwrap();
    let contract_addr = env.clone().contract.address.into_string();
    let mut result: u64 = 0u64;
    let user_addr_bytes = msg.clone().sender.into_bytes();
    for user_addr_byte in user_addr_bytes {
        result += user_addr_byte as u64;
    }
    let mut transfered;
    let mut number_iterated: HashSet<u64> = HashSet::<u64>::new();
    let number_to_add = result + msg.clone().token_id.parse::<u64>().unwrap();

    // transfering pre minted tool
    transfered = transfer_pack_nfts(
        deps.storage,
        env.clone(),
        token.pre_mint_tool,
        &msg,
        &mut responses,
        contract_addr.clone(),
        number_to_add,
    );
    let mut number = 0;
    let mut failure_count_of_transfer_pack = 0;
    let time_in_epoch_seconds = env.block.time.nanos();

    let mut random_number = generate_random_number(
        time_in_epoch_seconds + number_to_add,
        tool_types.len() as u64,
    );
    // if preminted tool set is no more available
    if !transfered {
        number = -1;
    }
    while number < 3 {
        transfered = transfer_pack_nfts(
            deps.storage,
            env.clone(),
            tool_types.get(random_number as usize).unwrap().to_string(),
            &msg,
            &mut responses,
            contract_addr.clone(),
            number_to_add,
        );

        if failure_count_of_transfer_pack == 4 {
            return Err(ContractError::NoTokenAvailableForDistribute {});
        }
        //if random index hit not available tool set
        if !transfered {
            random_number = (random_number + 1) % tool_types.len() as u64;
            if !number_iterated.contains(&random_number) {
                number_iterated.insert(random_number);
                failure_count_of_transfer_pack += 1;
            }
            continue;
        }
        number += 1;
        random_number = generate_random_number(
            time_in_epoch_seconds * number_to_add / random_number,
            tool_types.len() as u64,
        );
    }
    // burning the token
    responses.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr,
        msg: to_binary(&ExecuteMsg::Burn {
            token_id: msg.token_id,
        })?,
        funds: vec![],
    }));

    Ok(Response::new().add_messages(responses))
}

///transfer opened pack nft
pub fn transfer_pack_nfts(
    store: &mut dyn Storage,
    env: Env,
    tool_type: String,
    msg: &Cw721ReceiveMsg,
    responses: &mut Vec<CosmosMsg>,
    contract_addr: String,
    number_to_add: u64,
) -> bool {
    let time_in_epoch_seconds = env.block.time.nanos() + number_to_add;
    let mut token_ids =
        if let Some(token_ids) = TOOL_SET_MAP.may_load(store, tool_type.to_string()).unwrap() {
            token_ids
        } else {
            vec![]
        };
    if token_ids.len() as u64 <= 0 {
        return false;
    }
    let random_number = generate_random_number(
        time_in_epoch_seconds + token_ids.len() as u64,
        token_ids.len() as u64,
    );
    let token_id = token_ids.swap_remove(random_number as usize);
    TOOL_SET_MAP.save(store, tool_type, &token_ids).unwrap();

    responses.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr,
        msg: to_binary(&ExecuteMsg::TransferNft {
            recipient: msg.sender.to_string(),
            token_id,
        })
        .unwrap(),
        funds: vec![],
    }));

    true
}

///transfer nft
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

/// claiming reward
pub fn execute_claim_reward(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_id: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let mut token_info = if let Some(token_info) = tokens().may_load(deps.storage, &token_id)? {
        token_info
    } else {
        return Err(ContractError::NotFound {});
    };

    let mut user_energy_level = if let Some(user_energy_level) =
        USER_ENERGY_LEVEL.may_load(deps.storage, info.sender.to_string())?
    {
        user_energy_level
    } else {
        return Err(ContractError::NoEnergy {});
    };

    if user_energy_level < Uint128::from(3u128) {
        return Err(ContractError::NotEnoughEnergy {});
    }

    user_energy_level -= Uint128::from(3u128);

    let stake_info = if let Some(stake_info) =
        USER_STAKED_INFO.may_load(deps.storage, info.sender.to_string())?
    {
        stake_info
    } else {
        return Err(ContractError::NotEligible {});
    };

    if !stake_info.contains(&token_id) {
        return Err(ContractError::NotFound {});
    }
    let reward_token = if let Some(reward_token) =
        REWARD_TOKEN.may_load(deps.storage, token_info.name.to_string())?
    {
        reward_token
    } else {
        return Err(ContractError::NoRewardTokenFound {});
    };

    if token_info.reward_start_time + reward_token.mining_waiting_time < env.block.time.seconds() {
        let mut user_item_key = info.sender.to_string();
        user_item_key.push_str(&reward_token.item_name);
        let mut user_item_amount = if let Some(user_item_amount) =
            USER_ITEM_AMOUNT.may_load(deps.storage, user_item_key.to_string())?
        {
            user_item_amount
        } else {
            Uint128::zero()
        };

        if config.durability_start_time < env.block.time.seconds() {
            token_info.durability -= 1;
        }

        user_item_amount += Uint128::from(reward_token.mining_rate);
        USER_ITEM_AMOUNT.save(deps.storage, user_item_key.to_string(), &user_item_amount)?;
        let mut contract_item_key = env.contract.address.to_string();
        contract_item_key.push_str(&reward_token.item_name);
        let mut contract_item_amount = USER_ITEM_AMOUNT
            .may_load(deps.storage, contract_item_key.to_string())?
            .unwrap();
        contract_item_amount -= Uint128::from(reward_token.mining_rate);
        USER_ITEM_AMOUNT.save(deps.storage, contract_item_key, &contract_item_amount)?;
        token_info.reward_start_time = env.block.time.seconds();
        tokens().save(deps.storage, &token_id, &token_info)?;
    } else {
        return Err(ContractError::TimeNotReached {});
    }
    USER_ENERGY_LEVEL.save(deps.storage, info.sender.to_string(), &user_energy_level)?;
    Ok(Response::new())
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
    burn(deps.storage, token_id.to_string());

    Ok(Response::new()
        .add_attribute("action", "burn")
        .add_attribute("sender", info.sender)
        .add_attribute("token_id", token_id))
}

pub fn burn(store: &mut dyn Storage, token_id: String) {
    tokens().remove(store, &token_id).unwrap();
    decrement_tokens(store).unwrap();
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
        QueryMsg::UserStakedInfo { user_address } => {
            to_binary(&query_user_staked_info(deps, user_address)?)
        }
        QueryMsg::UserItemBalance {
            user_address,
            item_name,
        } => to_binary(&query_user_item_balance(deps, user_address, item_name)?),
        QueryMsg::UserEnergyInfo { user_address } => {
            to_binary(&query_user_energy_info(deps, user_address)?)
        }
        QueryMsg::UserItemInfo { user_address } => {
            to_binary(&query_user_item_info(deps, user_address)?)
        }
        QueryMsg::UserTokenBalance { user_address } => {
            to_binary(&query_user_token_balance(deps, user_address)?)
        }
    }
}

fn query_contract_info(deps: Deps) -> StdResult<ContractInfoResponse> {
    CONTRACT_INFO.load(deps.storage)
}
fn query_user_energy_info(deps: Deps, user_address: String) -> StdResult<Uint128> {
    if let Some(user_energy) = USER_ENERGY_LEVEL.may_load(deps.storage, user_address)? {
        Ok(user_energy)
    } else {
        Ok(Uint128::zero())
    }
}
fn query_user_item_balance(
    deps: Deps,
    user_address: String,
    item_name: String,
) -> StdResult<Uint128> {
    let mut user_item_key = user_address;
    user_item_key.push_str(&item_name);
    if let Some(amount) = USER_ITEM_AMOUNT.may_load(deps.storage, user_item_key)? {
        Ok(amount)
    } else {
        Ok(Uint128::zero())
    }
}

fn query_user_token_balance(deps: Deps, user_address: String) -> StdResult<Response> {
    let wood_token_addr = ITEM_TOKEN_MAPPING
        .may_load(deps.storage, "gWood".to_string())?
        .unwrap();
    let food_token_addr = ITEM_TOKEN_MAPPING
        .may_load(deps.storage, "gFood".to_string())?
        .unwrap();
    let stone_token_addr = ITEM_TOKEN_MAPPING
        .may_load(deps.storage, "gStone".to_string())?
        .unwrap();
    let gold_token_addr = ITEM_TOKEN_MAPPING
        .may_load(deps.storage, "gGold".to_string())?
        .unwrap();

    let wood_amount: Cw20BalanceResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: wood_token_addr,
            msg: to_binary(&Cw20QueryMsg::Balance {
                address: user_address.to_string(),
            })?,
        }))?;

    let food_amount: Cw20BalanceResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: food_token_addr,
            msg: to_binary(&Cw20QueryMsg::Balance {
                address: user_address.to_string(),
            })?,
        }))?;

    let gold_amount: Cw20BalanceResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: stone_token_addr,
            msg: to_binary(&Cw20QueryMsg::Balance {
                address: user_address.to_string(),
            })?,
        }))?;

    let stone_amount: Cw20BalanceResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: gold_token_addr,
            msg: to_binary(&Cw20QueryMsg::Balance {
                address: user_address,
            })?,
        }))?;

    Ok(Response::new()
        .add_attribute("wood token balance", wood_amount.balance.to_string())
        .add_attribute("food token balance", food_amount.balance.to_string())
        .add_attribute("gold token balance", gold_amount.balance.to_string())
        .add_attribute("stone token balance", stone_amount.balance.to_string()))
}

fn query_user_item_info(deps: Deps, user_address: String) -> StdResult<Response> {
    let mut user_item_key = user_address.to_string();
    user_item_key.push_str("gWood");
    let g_wood_amount =
        if let Some(g_wood_amount) = USER_ITEM_AMOUNT.may_load(deps.storage, user_item_key)? {
            g_wood_amount
        } else {
            Uint128::zero()
        };

    let mut user_item_key = user_address.to_string();
    user_item_key.push_str("gFood");
    let g_food_amount =
        if let Some(g_food_amount) = USER_ITEM_AMOUNT.may_load(deps.storage, user_item_key)? {
            g_food_amount
        } else {
            Uint128::zero()
        };

    let mut user_item_key = user_address.to_string();
    user_item_key.push_str("gGold");
    let g_gold_amount =
        if let Some(g_gold_amount) = USER_ITEM_AMOUNT.may_load(deps.storage, user_item_key)? {
            g_gold_amount
        } else {
            Uint128::zero()
        };

    let mut user_item_key = user_address.to_string();
    user_item_key.push_str("gStone");
    let g_stone_amount =
        if let Some(g_stone_amount) = USER_ITEM_AMOUNT.may_load(deps.storage, user_item_key)? {
            g_stone_amount
        } else {
            Uint128::zero()
        };

    let user_energy =
        if let Some(user_energy) = USER_ENERGY_LEVEL.may_load(deps.storage, user_address)? {
            user_energy
        } else {
            Uint128::zero()
        };

    Ok(Response::new()
        .add_attribute("gWood", g_wood_amount)
        .add_attribute("gFood", g_food_amount)
        .add_attribute("gGold", g_gold_amount)
        .add_attribute("gStone", g_stone_amount)
        .add_attribute("user_energy", user_energy))
}

fn query_nft_info(deps: Deps, token_id: String) -> StdResult<NftInfoResponse> {
    let info = tokens().load(deps.storage, &token_id)?;
    let nft_reward_info = if let Some(nft_reward_info) =
        REWARD_TOKEN.may_load(deps.storage, info.name.to_string())?
    {
        nft_reward_info
    } else {
        return Err(StdError::generic_err("token not found"));
    };

    let mut template_key = info.tool_type.to_string();
    template_key.push_str(info.rarity.as_str());
    let tool_template = if let Some(tool_template) =
        TOOL_TEMPLATE_MAP.may_load(deps.storage, template_key.to_string())?
    {
        tool_template
    } else {
        return Err(StdError::generic_err("No token found"));
    };

    Ok(NftInfoResponse {
        token_uri: tool_template.image.to_string(),
        extension: Extension {
            name: info.name,
            description: tool_template.description,
            image: Some(tool_template.image),
            rarity: info.rarity,
            mining_waiting_time: nft_reward_info.mining_waiting_time,
            mining_rate: nft_reward_info.mining_rate,
            owner: info.owner.to_string(),
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
    let nft_reward_info = if let Some(nft_reward_info) =
        REWARD_TOKEN.may_load(deps.storage, info.name.to_string())?
    {
        nft_reward_info
    } else {
        return Err(StdError::generic_err("token not found"));
    };

    let mut template_key = info.tool_type.to_string();
    template_key.push_str(info.rarity.as_str());
    let tool_template = if let Some(tool_template) =
        TOOL_TEMPLATE_MAP.may_load(deps.storage, template_key.to_string())?
    {
        tool_template
    } else {
        return Err(StdError::generic_err("No token found"));
    };

    Ok(AllNftInfoResponse {
        access: OwnerOfResponse {
            owner: info.owner.to_string(),
            approvals: humanize_approvals(&env.block, &info),
        },
        info: NftInfoResponse {
            token_uri: tool_template.image.to_string(),
            extension: Extension {
                name: info.name,
                description: tool_template.description,
                image: Some(tool_template.image),
                rarity: info.rarity,
                mining_waiting_time: nft_reward_info.mining_waiting_time,
                mining_rate: nft_reward_info.mining_rate,
                owner: info.owner.to_string(),
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

fn query_user_staked_info(deps: Deps, user_address: String) -> StdResult<HashSet<String>> {
    let stake_token_ids =
        if let Some(stake_token_ids) = USER_STAKED_INFO.may_load(deps.storage, user_address)? {
            stake_token_ids
        } else {
            HashSet::<String>::new()
        };

    Ok(stake_token_ids)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    Ok(Response::default())
}
