#[cfg(not(feature = "library"))]
use cosmwasm_std::{
    entry_point, from_binary, to_binary, Binary, BlockInfo, CosmosMsg, Deps, DepsMut, Env,
    MessageInfo, Order, QueryRequest, Response, StdError, StdResult, Storage, Uint128, WasmMsg,
    WasmQuery,
};
use cw0::maybe_addr;
use cw2::set_contract_version;
use cw20::{BalanceResponse, Cw20ExecuteMsg, Cw20QueryMsg, Cw20ReceiveMsg};
use cw721::{
    ContractInfoResponse, Cw721ReceiveMsg, Expiration, NumTokensResponse, OwnerOfResponse,
    TokensResponse,
};
use cw_storage_plus::Bound;
use std::collections::HashSet;

use crate::mint::{
    execute_batch_mint, execute_mint, execute_mint_common_nft, execute_mint_upgraded_nft,
};
use crate::msg::{
    AllNftInfoResponse, Cw20HookMsg, Cw721HookMsg, ExecuteMsg, Extension, InstantiateMsg,
    MigrateMsg, NftInfoResponse, QueryMsg, ToolTemplateMsg, UpdateConfigMsg,
};
use crate::state::{
    distribute_amount, num_tokens, tokens, Approval, Config, RewardToken, TokenInfo, ToolTemplate,
    CONFIG, CONTRACT_INFO, GAME_DEV_TOKENS_NAME, ITEM_TOKEN_MAPPING, LAST_GEN_TOKEN_ID, OPERATORS,
    RARITY_TYPES, REPAIRING_FEE, REPAIR_KIT_KEYWORD, REWARD_TOKEN, TOKEN_COUNT, TOKEN_ITEM_MAPPING,
    TOOL_PACK_SET, TOOL_SET_MAP, TOOL_TEMPLATE_MAP, TOOL_TYPE_NAMES, USER_ENERGY_LEVEL,
    USER_ITEM_AMOUNT, USER_REPAIR_KITS, USER_STAKED_INFO,  
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
        repair_kit_waiting_time: msg.repair_kit_waiting_time,
    };

    CONTRACT_INFO.save(deps.storage, &contract_info)?;
    CONFIG.save(deps.storage, &config)?;
    TOOL_TYPE_NAMES.save(deps.storage, &vec![])?;
    LAST_GEN_TOKEN_ID.save(deps.storage, &0u64)?;
    let game_dev_token_set = Vec::<String>::new();
    GAME_DEV_TOKENS_NAME.save(deps.storage, &game_dev_token_set)?;
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
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

        ExecuteMsg::TransferToolPack {
            recipient,
            tool_type,
        } => execute_transfer_tool_pack(deps, info, env, recipient, tool_type),

        ExecuteMsg::AddItemName { item_name } => execute_adding_item(deps, info, item_name),

        ExecuteMsg::AddRaritiesMapping {
            tool_type,
            upgraded_tool_type,
        } => execute_add_rarities_mapping(deps, info, tool_type, upgraded_tool_type),

        ExecuteMsg::UnstakeRepairKit {
            repair_kit_token_id,
        } => execute_unstake_repair_tool(deps, info, env, repair_kit_token_id),

        ExecuteMsg::UseRepairKit { token_id } => execute_use_repair_tool(deps, info, env, token_id),

        ExecuteMsg::AddRepairingFee { rarity, fee } => {
            execute_add_repairing_fee(deps, info, rarity, fee)
        },
    }
}

fn execute_transfer_tool_pack(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    recipient: String,
    tool_type: String,
) -> StdResult<Response> {
    let mut tool_pack_set =
        if let Some(tool_pack_set) = TOOL_PACK_SET.may_load(deps.storage, tool_type.to_string())? {
            tool_pack_set
        } else {
            return Err(StdError::generic_err("No tool pack available to transfer"));
        };

    if tool_pack_set.is_empty() {
        return Err(StdError::generic_err(
            "No tool pack list available to transfer",
        ));
    }
    let token_id = tool_pack_set.swap_remove(0);
    //let token = tokens().load(deps.storage, &token_id)?;
    TOOL_PACK_SET.save(deps.storage, tool_type, &tool_pack_set)?;
    _transfer_nft(deps, &env, &info, &recipient, &token_id)?;

    Ok(Response::new()
        .add_attribute("action", "transfer tool pack")
        .add_attribute("sender", info.sender)
        .add_attribute("recipient", recipient)
        .add_attribute("token_id", token_id))
}
fn execute_update_config(
    deps: DepsMut,
    info: MessageInfo,
    msg: UpdateConfigMsg,
) -> StdResult<Response> {
    let mut config = CONFIG.load(deps.storage)?;
    if config.minter != info.sender {
        return Err(StdError::generic_err("Unauthorized"));
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
    Ok(Response::new()
        .add_attribute("action", "update config")
        .add_attribute("sender", info.sender))
}

fn execute_add_rarities_mapping(
    deps: DepsMut,
    info: MessageInfo,
    tool_type: String,
    upgraded_tool_type: String,
) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;
    if config.minter != info.sender {
        return Err(StdError::generic_err("Unauthorized"));
    }
    RARITY_TYPES.save(deps.storage, tool_type.to_string(), &upgraded_tool_type)?;
    Ok(Response::new()
        .add_attribute("action", "add rarities mapping")
        .add_attribute("tool type", tool_type)
        .add_attribute("upgraded tool type", upgraded_tool_type))
}

fn execute_adding_item(deps: DepsMut, info: MessageInfo, item_name: String) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;
    if config.minter != info.sender {
        return Err(StdError::generic_err("Unauthorized"));
    }
    let mut game_dev_token_set = GAME_DEV_TOKENS_NAME.load(deps.storage)?;
    game_dev_token_set.push(item_name.to_string());
    Ok(Response::new()
        .add_attribute("action", "item added")
        .add_attribute("item", item_name))
}

fn execute_add_repairing_fee(
    deps: DepsMut,
    info: MessageInfo,
    rarity: String,
    fee: Uint128,
) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;

    if config.minter == info.sender {
        return Err(StdError::generic_err("Unauthorized"));
    }

    REPAIRING_FEE.save(deps.storage, rarity.to_string(), &fee)?;

    Ok(Response::new()
        .add_attribute("action", "item added")
        .add_attribute("rarity", rarity)
        .add_attribute("fee", fee))
}

/// to transfer reserve amount of contract pool to withdraw
fn execute_transfer_reserve_amount(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;
    if config.minter != info.sender {
        return Err(StdError::generic_err("Unauthorized"));
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
                return Err(StdError::generic_err("No Item token found"));
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
    Ok(Response::new()
        .add_messages(responses)
        .add_attribute("action", "transfer reserve amount")
        .add_attribute("sender", info.sender))
}

/// adding tool template/snapshot in the contract
fn execute_add_tool_template(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ToolTemplateMsg,
) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.minter {
        return Err(StdError::generic_err("Unauthorized"));
    }
    let mut tool_template = ToolTemplate {
        name: msg.name,
        description: msg.description,
        image: msg.image,
        rarity: msg.rarity.to_string(),
        required_amount: vec![],
        durability: msg.durability,
        token_uri: msg.token_uri,
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
    Ok(Response::default()
        .add_attribute("action", "add tool template")
        .add_attribute("sender", info.sender)
        .add_attribute("tool template name", tool_template.name))
}

/// to withdraw tokens in exchange of game dev tokens
pub fn execute_withdraw(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    item_name: String,
    amount: Uint128,
) -> StdResult<Response> {
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
        return Err(StdError::generic_err("Insufficient funds"));
    }
    let token_addr = if let Some(token_addr) =
        ITEM_TOKEN_MAPPING.may_load(deps.storage, item_name.to_string())?
    {
        token_addr
    } else {
        return Err(StdError::generic_err("Not found"));
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
    Ok(Response::default()
        .add_message(response)
        .add_attribute("action", "withdraw")
        .add_attribute("item_name", item_name.to_string())
        .add_attribute("amount", amount))
}
///adding game dev token against tokns or vice versa e.g. gWood -> some address
pub fn execute_add_item_token(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    item_name: String,
    item_token_addr: String,
) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.minter {
        return Err(StdError::generic_err("Unauthorized"));
    }
    TOKEN_ITEM_MAPPING.save(deps.storage, item_token_addr.to_string(), &item_name)?;
    ITEM_TOKEN_MAPPING.save(deps.storage, item_name.to_string(), &item_token_addr)?;
    Ok(Response::default()
        .add_attribute("action", "add item token")
        .add_attribute("item_name", item_name.to_string())
        .add_attribute("item_address", item_token_addr.to_string()))
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
) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;

    if info.sender != config.minter {
        return Err(StdError::generic_err("Unauthorized"));
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
) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;

    if info.sender != config.minter {
        return Err(StdError::generic_err("Unauthorized"));
    }
    let mut tool_type_names = TOOL_TYPE_NAMES.may_load(deps.storage)?.unwrap();
    for i in tool_type_names.iter() {
        if *i == tool_type {
            return Err(StdError::generic_err("Already exist"));
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
) -> StdResult<Response> {
    match from_binary(&msg.msg) {
        Ok(Cw20HookMsg::Deposit {}) => execute_deposit(deps, env, info, msg),
        Ok(Cw20HookMsg::AdminDeposit {}) => execute_admin_deposit(deps, env, info, msg),
        Err(_err) => Err(StdError::generic_err("Already exist")),
    }
}

///let user deposit tokens in exchange of dev tokens
pub fn execute_deposit(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: Cw20ReceiveMsg,
) -> StdResult<Response> {
    let item_name = if let Some(item_name) =
        TOKEN_ITEM_MAPPING.may_load(deps.storage, info.sender.to_string())?
    {
        item_name
    } else {
        return Err(StdError::generic_err("No token item found"));
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
    Ok(Response::new()
        .add_attribute("action", "deposit")
        .add_attribute("sender", msg.sender)
        .add_attribute("amount", msg.amount))
}

/// let admin deposit tokens
pub fn execute_admin_deposit(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: Cw20ReceiveMsg,
) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;
    if config.minter != msg.sender {
        return Err(StdError::generic_err("Unauthorized"));
    }
    let item_name = if let Some(item_name) =
        TOKEN_ITEM_MAPPING.may_load(deps.storage, info.sender.to_string())?
    {
        item_name
    } else {
        return Err(StdError::generic_err("Not found"));
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
    Ok(Response::new()
        .add_attribute("action", "admin deposit")
        .add_attribute("sender", msg.sender)
        .add_attribute("amount", msg.amount))
}

///let user refill energy to execute claiming reward transaction
pub fn execute_refill_energy(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: u64,
) -> StdResult<Response> {
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
        return Err(StdError::generic_err("Insufficient funds"));
    }
    user_energy_level += amount.multiply_ratio(Uint128::from(3u128), Uint128::from(1u128));

    USER_ENERGY_LEVEL.save(deps.storage, info.sender.to_string(), &user_energy_level)?;
    user_item_amount -= amount;
    USER_ITEM_AMOUNT.save(deps.storage, info.sender.to_string(), &user_item_amount)?;
    distribute_amount(deps.storage, "gFood".to_string(), amount, &config, &env);
    Ok(Response::new()
        .add_attribute("action", "refill energy")
        .add_attribute("sender", info.sender)
        .add_attribute("amount", amount))
}

/// transfering nft to other stakeholder
pub fn execute_transfer_nft(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipient: String,
    token_id: String,
) -> StdResult<Response> {
    _transfer_nft(deps, &env, &info, &recipient, &token_id)?;

    Ok(Response::new()
        .add_attribute("action", "transfer nft")
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
) -> StdResult<Response> {
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
        .add_attribute("action", "send nft")
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
) -> StdResult<Response> {
    println!("contract address {} ", env.contract.address);
    if env.contract.address != info.sender {
        return Err(StdError::generic_err("Unauthorized"));
    }
    match from_binary(&msg.msg) {
        Ok(Cw721HookMsg::Stake {}) => execute_stake(deps, env, msg),
        Ok(Cw721HookMsg::OpenPack {}) => execute_open_pack(deps, env, msg),
        Ok(Cw721HookMsg::StakeRepairKit {}) => execute_stake_repair_kit(deps, info, env, msg),
        Err(_err) => Err(StdError::generic_err("no method found")),
    }
}

/// staking nft for earning reward
pub fn execute_stake(deps: DepsMut, env: Env, msg: Cw721ReceiveMsg) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;
    let mut token = tokens().load(deps.storage, &msg.token_id)?;

    // check that only pack can be opened, not any ohter nft from our contract
    if token.is_pack_token {
        return Err(StdError::generic_err(
            "Not eligible because provided token is a pack token",
        ));
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
        return Err(StdError::generic_err("Limit reached"));
    }

    token.reward_start_time = env.block.time.seconds();
    stake_info.insert(msg.token_id.to_string());
    USER_STAKED_INFO.save(deps.storage, msg.sender.to_string(), &stake_info)?;
    tokens().save(deps.storage, &msg.token_id, &token)?;
    Ok(Response::new()
        .add_attribute("action", "stake")
        .add_attribute("sender", msg.sender.to_string())
        .add_attribute("token_id", msg.token_id))
}

/// unstaking nft
pub fn execute_unstake(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_id: String,
) -> StdResult<Response> {
    let mut responses: Vec<CosmosMsg> = vec![];
    let mut stake_ids_set = if let Some(stake_ids_set) =
        USER_STAKED_INFO.may_load(deps.storage, info.sender.to_string())?
    {
        stake_ids_set
    } else {
        return Err(StdError::generic_err("No staked asset found"));
    };

    if !stake_ids_set.contains(&token_id.to_string()) {
        return Err(StdError::generic_err("No token id found"));
    }
    let token_info = tokens().load(deps.storage, &token_id)?;
    let reward_info = REWARD_TOKEN.load(deps.storage, token_info.name.to_string())?;
    if token_info.reward_start_time + reward_info.mining_waiting_time > env.block.time.seconds() {
        return Err(StdError::generic_err("Time not reached yet"));
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
    Ok(Response::new()
        .add_messages(responses)
        .add_attribute("action", "Unstake")
        .add_attribute("sender", info.sender)
        .add_attribute("token_id", token_id))
}

///let user open pack
pub fn execute_open_pack(deps: DepsMut, env: Env, msg: Cw721ReceiveMsg) -> StdResult<Response> {
    let token = tokens().load(deps.storage, &msg.token_id)?;
    let mut responses: Vec<CosmosMsg> = vec![];

    // check that only pack can be opened, not any other nft from our contract
    if !token.is_pack_token {
        return Err(StdError::generic_err(
            "Not eligible as token is not a pack token",
        ));
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
    let mut number_to_add = result + msg.clone().token_id.parse::<u64>().unwrap();
    let mut time_in_epoch_seconds = env.block.time.nanos();

    // transfering pre minted tool
    transfered = transfer_pack_nfts(
        deps.storage,
        time_in_epoch_seconds,
        token.pre_mint_tool,
        &msg,
        &mut responses,
        contract_addr.clone(),
        number_to_add,
    );
    let mut iterator = 0;
    let mut number = 3;
    let mut failure_count_of_transfer_pack = 0;

    let mut random_number = generate_random_number(
        time_in_epoch_seconds + number_to_add,
        tool_types.len() as u64,
    );

    // if preminted tool set is no more available
    if !transfered.unwrap() {
        number = 4;
    }

    while iterator < number {
        transfered = transfer_pack_nfts(
            deps.storage,
            time_in_epoch_seconds,
            tool_types.get(random_number as usize).unwrap().to_string(),
            &msg,
            &mut responses,
            contract_addr.clone(),
            number_to_add,
        );
        if failure_count_of_transfer_pack == number {
            return Err(StdError::generic_err("No Tool available to distribute"));
        }
        //if random index hit not available tool set
        if !transfered.unwrap() {
            random_number = (random_number + 1) % tool_types.len() as u64;
            if !number_iterated.contains(&random_number) {
                number_iterated.insert(random_number);
                failure_count_of_transfer_pack += 1;
            }
            continue;
        }

        iterator += 1;
        number_to_add += random_number;
        time_in_epoch_seconds += random_number;
        random_number = generate_random_number(
            iterator + time_in_epoch_seconds + number_to_add - random_number,
            tool_types.len() as u64,
        );
    }
    // burning the token
    responses.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr,
        msg: to_binary(&ExecuteMsg::Burn {
            token_id: msg.token_id.to_string(),
        })?,
        funds: vec![],
    }));

    Ok(Response::new()
        .add_messages(responses)
        .add_attribute("action", "open pack")
        .add_attribute("sender", msg.sender)
        .add_attribute("pack_token_id", msg.token_id))
}

pub fn execute_stake_repair_kit(
    deps: DepsMut,
    _info: MessageInfo,
    env: Env,
    msg: Cw721ReceiveMsg,
) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;
    let mut token = if let Some(token) = tokens().may_load(deps.storage, &msg.token_id)? {
        token
    } else {
        return Err(StdError::generic_err("No Token Found"));
    };
    let mut user_repair_kit_key = msg.sender.to_string();
    user_repair_kit_key.push_str(REPAIR_KIT_KEYWORD);
    user_repair_kit_key.push_str(token.tool_type.to_string().as_str());
    if USER_REPAIR_KITS.has(deps.storage, user_repair_kit_key.to_string()) {
        return Err(StdError::generic_err("Repair kit is already deployed"));
    }
    USER_REPAIR_KITS.save(deps.storage, user_repair_kit_key, &msg.token_id)?;
    token.repair_kit_available_time = env.block.time.seconds() + config.repair_kit_waiting_time;
    tokens().save(deps.storage, &msg.token_id, &token)?;
    Ok(Response::new()
        .add_attribute("action", "repair kit staked")
        .add_attribute("token id", msg.token_id)
        .add_attribute("tool_type", token.tool_type))
}

pub fn execute_unstake_repair_tool(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    repair_kit_token_id: String,
) -> StdResult<Response> {
    let token = if let Some(token) = tokens().may_load(deps.storage, &repair_kit_token_id)? {
        token
    } else {
        return Err(StdError::generic_err("no token available"));
    };

    let mut user_repair_kit_key = info.sender.to_string();
    user_repair_kit_key.push_str(REPAIR_KIT_KEYWORD);
    user_repair_kit_key.push_str(token.tool_type.to_string().as_str());

    let user_repair_kit_id = if let Some(user_repair_kit_id) =
        USER_REPAIR_KITS.may_load(deps.storage, user_repair_kit_key.to_string())?
    {
        user_repair_kit_id
    } else {
        return Err(StdError::generic_err("User do not stake any repair kit"));
    };
    let user_repair_kit_token = if let Some(user_repair_kit_token) =
        tokens().may_load(deps.storage, &user_repair_kit_id)?
    {
        user_repair_kit_token
    } else {
        return Err(StdError::generic_err("no user repair kit token available"));
    };
    if user_repair_kit_token.repair_kit_available_time < env.block.time.seconds() {
        USER_REPAIR_KITS.remove(deps.storage, user_repair_kit_key);
    } else {
        return Err(StdError::generic_err("Time not reached yet"));
    }
    let response = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: env.contract.address.into_string(),
        msg: to_binary(&ExecuteMsg::TransferNft {
            recipient: info.sender.to_string(),
            token_id: repair_kit_token_id.to_string(),
        })?,
        funds: vec![],
    });

    Ok(Response::new()
        .add_message(response)
        .add_attribute("action", "repair kit unstaked")
        .add_attribute("token id", repair_kit_token_id)
        .add_attribute("tool_type", token.tool_type))
}

pub fn execute_use_repair_tool(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    token_id: String,
) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;
    let mut token = if let Some(token) = tokens().may_load(deps.storage, &token_id)? {
        token
    } else {
        return Err(StdError::generic_err("no token available"));
    };
    let mut template_key = token.tool_type.to_string();
    template_key.push_str(token.rarity.to_string().as_str());
    let tool_template = TOOL_TEMPLATE_MAP.load(deps.storage, template_key)?;
    let mut user_repair_kit_key = info.sender.to_string();
    user_repair_kit_key.push_str(REPAIR_KIT_KEYWORD);
    user_repair_kit_key.push_str(token.tool_type.to_string().as_str());
    let repairing_fee = if let Some(repairing_fee) =
        REPAIRING_FEE.may_load(deps.storage, token.tool_type.to_string())?
    {
        repairing_fee
    } else {
        return Err(StdError::generic_err("Repairing Fee is not set"));
    };
    let user_repair_kit_id = if let Some(user_repair_kit_id) =
        USER_REPAIR_KITS.may_load(deps.storage, user_repair_kit_key)?
    {
        user_repair_kit_id
    } else {
        return Err(StdError::generic_err(
            "User do not deploy any tool's repair kit",
        ));
    };
    let reward_item =
        if let Some(reward_item) = REWARD_TOKEN.may_load(deps.storage, token.name.to_string())? {
            reward_item
        } else {
            return Err(StdError::generic_err("No item found against tool"));
        };
    let mut user_item_key = info.sender.to_string();
    user_item_key.push_str(reward_item.item_name.as_str());
    let mut user_item_amount =
        if let Some(user_item_amount) = USER_ITEM_AMOUNT.may_load(deps.storage, user_item_key)? {
            user_item_amount
        } else {
            return Err(StdError::generic_err("No item available in user account"));
        };
    if user_item_amount < repairing_fee {
        return Err(StdError::generic_err("Insufficient items"));
    }
    user_item_amount -= repairing_fee;
    distribute_amount(
        deps.storage,
        reward_item.item_name.to_string(),
        repairing_fee,
        &config,
        &env,
    );
    let mut user_repair_kit_token = if let Some(user_repair_kit_token) =
        tokens().may_load(deps.storage, &user_repair_kit_id)?
    {
        user_repair_kit_token
    } else {
        return Err(StdError::generic_err("no user repair kit token available"));
    };
    if token.durability == tool_template.durability {
        return Err(StdError::generic_err(
            "tool is perfectly fine no need to repair",
        ));
    }
    if user_repair_kit_token.repair_kit_available_time < env.block.time.seconds() {
        token.durability = tool_template.durability;
        user_repair_kit_token.repair_kit_available_time = env.block.time.seconds();
    } else {
        return Err(StdError::generic_err("Time not reached yet"));
    }

    tokens().save(deps.storage, &token_id, &token)?;
    tokens().save(deps.storage, &user_repair_kit_id, &user_repair_kit_token)?;
    Ok(Response::new().add_attribute("action", "repair tool"))
}

///transfer opened pack nft
pub fn transfer_pack_nfts(
    store: &mut dyn Storage,
    mut time_in_epoch_seconds: u64,
    tool_type: String,
    msg: &Cw721ReceiveMsg,
    responses: &mut Vec<CosmosMsg>,
    contract_addr: String,
    number_to_add: u64,
) -> StdResult<bool> {
    time_in_epoch_seconds += number_to_add;

    let mut token_ids =
        if let Some(token_ids) = TOOL_SET_MAP.may_load(store, tool_type.to_string())? {
            token_ids
        } else {
            let mut message = " tool type: ".to_string();
            message.push_str(&tool_type);
            message.push_str(" error in token ids");
            return Err(StdError::generic_err(message));
        };

    let random_number = generate_random_number(
        time_in_epoch_seconds + token_ids.len() as u64,
        token_ids.len() as u64,
    );
    if token_ids.is_empty() {
        return Ok(false);
    }
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

    Ok(true)
}

///transfer nft
pub fn _transfer_nft(
    deps: DepsMut,
    env: &Env,
    info: &MessageInfo,
    recipient: &str,
    token_id: &str,
) -> StdResult<TokenInfo> {
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
) -> StdResult<Response> {
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
) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;
    let mut token_info = if let Some(token_info) = tokens().may_load(deps.storage, &token_id)? {
        token_info
    } else {
        return Err(StdError::generic_err("No token found"));
    };

    let mut user_energy_level = if let Some(user_energy_level) =
        USER_ENERGY_LEVEL.may_load(deps.storage, info.sender.to_string())?
    {
        user_energy_level
    } else {
        return Err(StdError::generic_err("No energy"));
    };

    if user_energy_level < Uint128::from(3u128) {
        return Err(StdError::generic_err("Not enough energy"));
    }

    user_energy_level -= Uint128::from(3u128);

    let stake_info = if let Some(stake_info) =
        USER_STAKED_INFO.may_load(deps.storage, info.sender.to_string())?
    {
        stake_info
    } else {
        return Err(StdError::generic_err("No staked asset found"));
    };

    if !stake_info.contains(&token_id) {
        return Err(StdError::generic_err("No token id found"));
    }
    let reward_token = if let Some(reward_token) =
        REWARD_TOKEN.may_load(deps.storage, token_info.name.to_string())?
    {
        reward_token
    } else {
        return Err(StdError::generic_err("No reward token found"));
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
        if token_info.durability < 1 {
            return Err(StdError::generic_err(
                "Kindly repair the tool first to claim reward",
            ));
        }
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
        return Err(StdError::generic_err("Time not reached yet"));
    }
    USER_ENERGY_LEVEL.save(deps.storage, info.sender.to_string(), &user_energy_level)?;
    Ok(Response::new()
        .add_attribute("action", "claim reward")
        .add_attribute("sender", info.sender)
        .add_attribute("token_id", token_id))
}
pub fn execute_revoke(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    spender: String,
    token_id: String,
) -> StdResult<Response> {
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
) -> StdResult<TokenInfo> {
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
            return Err(StdError::generic_err("Expired"));
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
) -> StdResult<Response> {
    // reject expired data as invalid
    let expires = expires.unwrap_or_default();
    if expires.is_expired(&env.block) {
        return Err(StdError::generic_err("Expired"));
    }

    // set the operator for us
    let operator_addr = deps.api.addr_validate(&operator)?;
    OPERATORS.save(deps.storage, (&info.sender, &operator_addr), &expires)?;

    Ok(Response::new()
        .add_attribute("action", "approve all")
        .add_attribute("sender", info.sender)
        .add_attribute("operator", operator))
}

pub fn execute_revoke_all(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    operator: String,
) -> StdResult<Response> {
    let operator_addr = deps.api.addr_validate(&operator)?;
    OPERATORS.remove(deps.storage, (&info.sender, &operator_addr));

    Ok(Response::new()
        .add_attribute("action", "revoke all")
        .add_attribute("sender", info.sender)
        .add_attribute("operator", operator))
}

/// returns true iff the sender can execute approve or reject on the contract
fn check_can_approve(
    deps: Deps,
    env: &Env,
    info: &MessageInfo,
    token: &TokenInfo,
) -> StdResult<()> {
    // owner can approve
    if token.owner == info.sender {
        return Ok(());
    }
    // operator can approve
    let op = OPERATORS.may_load(deps.storage, (&token.owner, &info.sender))?;
    match op {
        Some(ex) => {
            if ex.is_expired(&env.block) {
                Err(StdError::generic_err("Unauthorized"))
            } else {
                Ok(())
            }
        }
        None => Err(StdError::generic_err("Unauthorized")),
    }
}

/// returns true iff the sender can transfer ownership of the token
fn check_can_send(deps: Deps, env: &Env, info: &MessageInfo, token: &TokenInfo) -> StdResult<()> {
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
                Err(StdError::generic_err("Unauthorized"))
            } else {
                Ok(())
            }
        }
        None => Err(StdError::generic_err("Unauthorized")),
    }
}

// pub fn execute_upgrade_tool_level(
//     deps: DepsMut,
//     env: Env,
//     info: MessageInfo,
// ) -> StdResult<Response> {
//     Ok(Response::new().add_attribute("action", "update tool level"))
// }

fn execute_burn(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_id: String,
) -> StdResult<Response> {
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
) -> StdResult<()> {
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
                Err(StdError::generic_err("Unauthorized"))
            } else {
                Ok(())
            }
        }
        None => Err(StdError::generic_err("Unauthorized")),
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

        QueryMsg::QueryRemainingAllPackCount {} => {
            to_binary(&query_remaining_all_pack_count(deps)?)
        }
        QueryMsg::QueryRemainingPackCount { tool_type } => {
            to_binary(&query_remaining_pack_count(deps, tool_type)?)
        }
        QueryMsg::QueryGameDevToken {} => to_binary(&query_game_dev_token(deps)?),
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

fn query_remaining_all_pack_count(deps: Deps) -> StdResult<u64> {
    let mut remaining_packs = 0u64;
    let game_dev_tokens_name = GAME_DEV_TOKENS_NAME.load(deps.storage)?;
    for game_dev_token_name in game_dev_tokens_name {
        remaining_packs += if let Some(tool_set) =
            TOOL_PACK_SET.may_load(deps.storage, game_dev_token_name.to_string())?
        {
            tool_set.len() as u64
        } else {
            0u64
        };
    }
    Ok(remaining_packs)
}

fn query_remaining_pack_count(deps: Deps, tool_type: String) -> StdResult<u64> {
    let tool_count = if let Some(tool_set) = TOOL_PACK_SET.may_load(deps.storage, tool_type)? {
        tool_set.len() as u64
    } else {
        0u64
    };

    Ok(tool_count)
}

fn query_user_token_balance(deps: Deps, user_address: String) -> StdResult<Response> {
    let mut tokens_map = vec![];
    let game_dev_tokens_name = GAME_DEV_TOKENS_NAME.load(deps.storage)?;
    for game_dev_token_name in game_dev_tokens_name.clone() {
        let item_token = if let Some(item_token) =
            ITEM_TOKEN_MAPPING.may_load(deps.storage, game_dev_token_name.to_string())?
        {
            item_token
        } else {
            let mut error_message = String::from(game_dev_token_name.as_str());
            error_message.push_str(" game dev token not found");
            return Err(StdError::generic_err(error_message));
        };
        let amount: BalanceResponse =
            deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: item_token,
                msg: to_binary(&Cw20QueryMsg::Balance {
                    address: user_address.to_string(),
                })?,
            }))?;

        tokens_map.push((game_dev_token_name.to_string(), amount.balance));
    }

    Ok(Response::new().add_attributes(tokens_map))
}

fn query_user_item_info(deps: Deps, user_address: String) -> StdResult<Response> {
    let mut tokens_map = vec![];
    let game_dev_tokens_name = GAME_DEV_TOKENS_NAME.load(deps.storage)?;
    for game_dev_token_name in game_dev_tokens_name.clone() {
        let mut user_item_key = user_address.to_string();
        user_item_key.push_str(game_dev_token_name.as_str());
        let game_dev_token_amount = if let Some(game_dev_token_amount) =
            USER_ITEM_AMOUNT.may_load(deps.storage, user_item_key)?
        {
            game_dev_token_amount
        } else {
            Uint128::zero()
        };

        tokens_map.push((game_dev_token_name.to_string(), game_dev_token_amount));
    }

    let user_energy =
        if let Some(user_energy) = USER_ENERGY_LEVEL.may_load(deps.storage, user_address)? {
            user_energy
        } else {
            Uint128::zero()
        };
    Ok(Response::new()
        .add_attributes(tokens_map)
        .add_attribute("user energy", user_energy))
}

fn query_nft_info(deps: Deps, token_id: String) -> StdResult<NftInfoResponse> {
    let info = tokens().load(deps.storage, &token_id)?;
    let nft_reward_info = if let Some(nft_reward_info) =
        REWARD_TOKEN.may_load(deps.storage, info.name.to_string())?
    {
        nft_reward_info
    } else {
        RewardToken {
            item_name: "None".to_string(),
            mining_rate: 0u64,
            mining_waiting_time: 0u64,
        }
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
        token_uri: tool_template.token_uri,
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
        RewardToken {
            item_name: "None".to_string(),
            mining_rate: 0u64,
            mining_waiting_time: 0u64,
        }
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
            token_uri: tool_template.token_uri,
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

fn query_game_dev_token(deps: Deps) -> StdResult<Vec<String>> {
    GAME_DEV_TOKENS_NAME.load(deps.storage)
}

// #[cfg_attr(not(feature = "library"), entry_point)]
// pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
//     Ok(Response::default())
// }



#[entry_point]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    let ver = cw2::get_contract_version(deps.storage)?;
    // ensure we are migrating from an allowed contract
    if ver.contract != CONTRACT_NAME {
        return Err(StdError::generic_err("Can only upgrade from same type").into());
    }
    // note: better to do proper semver compare, but string compare *usually* works
    // if ver.version >= CONTRACT_VERSION.to_string() {
    //     return Err(StdError::generic_err("Cannot upgrade from a newer version").into());
    // }
    
    // set the new version
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    
    // do any desired state migrations...
    
    Ok(Response::default())
}