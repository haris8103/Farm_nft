#[cfg(not(feature = "library"))]
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response, StdError, StdResult, Storage, Uint128};

use crate::contract::{_check_can_send, burn};
use crate::msg::MintMsg;
use crate::state::{
    distribute_amount, increment_tokens, tokens, TokenInfo, CONFIG, GAME_DEV_TOKENS_NAME,
    LAST_GEN_TOKEN_ID, RARITY_TYPES, TOOL_PACK_SET, TOOL_SET_MAP, TOOL_TEMPLATE_MAP,
    USER_ITEM_AMOUNT,
};

/// to mint multiple nfts in a single transaction
pub fn execute_batch_mint(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: MintMsg,
) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;

    if info.sender != config.minter {
        return Err(StdError::generic_err("Unauthorized"));
    }
    let mut number = 0u64;
    let mut token_ids: String = String::new();
    // create the token
    while number < msg.minting_count.unwrap() {
        token_ids.push_str(mint(deps.storage, &env, &msg).to_string().as_str());
        token_ids.push_str(" ,");
        number += 1;
    }
    println!("tool_type {} , token id {}", msg.tool_type, token_ids);
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
) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;

    if info.sender != config.minter {
        return Err(StdError::generic_err("Unauthorized"));
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
        is_pack_token: false,
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
    } else if msg.rarity.eq_ignore_ascii_case("Pack") {
        let mut tool_pack_set = if let Some(tool_pack_set) = TOOL_PACK_SET
            .may_load(store, msg.tool_type.to_string())
            .unwrap()
        {
            tool_pack_set
        } else {
            vec![]
        };
        tool_pack_set.push(new_toke_id.to_string());
        TOOL_PACK_SET
            .save(store, msg.tool_type.to_string(), &tool_pack_set)
            .unwrap();
        token.is_pack_token = true;
    }
    tokens()
        .update(store, &new_toke_id.to_string(), |old| match old {
            Some(_) => Err(StdError::generic_err("Claimed")),
            None => Ok(token),
        })
        .unwrap();
    new_toke_id
}
/// mint common rarity type NFT in exchange game dev tokens
pub fn execute_mint_common_nft(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    tool_type: String,
) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;
    let mut template_key = tool_type.to_string();
    template_key.push_str("Common");
    let tool_template = if let Some(tool_template) =
        TOOL_TEMPLATE_MAP.may_load(deps.storage, template_key.to_string())?
    {
        tool_template
    } else {
        return Err(StdError::generic_err("Not found"));
    };
    let msg = MintMsg {
        owner: deps.api.addr_validate(&info.sender.to_string())?,
        name: tool_template.name,
        rarity: tool_template.rarity,
        pre_mint_tool: None,
        minting_count: None,
        tool_type,
    };
    let game_dev_token_set = GAME_DEV_TOKENS_NAME.load(deps.storage)?;
    for (index, game_dev_token_name) in game_dev_token_set.into_iter().enumerate() {
        let mut user_addr = String::from(&info.sender.to_string());
        user_addr.push_str(&game_dev_token_name);
        let mut item_required_amount = if let Some(item_required_amount) =
            USER_ITEM_AMOUNT.may_load(deps.storage, user_addr.to_string())?
        {
            item_required_amount
        } else {
            Uint128::zero()
        };
        if item_required_amount < *tool_template.required_amount.get(index).unwrap() {
            let mut message = String::from("dev_token_name: ");
            message.push_str(&game_dev_token_name.to_string());
            message.push_str(" item_required_amount: ");
            message.push_str(&item_required_amount.to_string());
            message.push_str(" tool_template.required_amount.get(index).unwrap(): ");
            message.push_str(
                &tool_template
                    .required_amount
                    .get(index)
                    .unwrap()
                    .to_string(),
            );
            message.push_str(" index: ");
            message.push_str(&index.to_string());
            message.push_str(" InSufficient funds ");
            return Err(StdError::generic_err(message));
        }
        item_required_amount -= *tool_template.required_amount.get(index).unwrap();
        let amount = *tool_template.required_amount.get(index).unwrap();
        distribute_amount(
            deps.storage,
            game_dev_token_name.to_string(),
            amount,
            &config,
            &env,
        );
        USER_ITEM_AMOUNT.save(deps.storage, user_addr.to_string(), &item_required_amount)?;
    }

    mint(deps.storage, &env, &msg);

    Ok(Response::new())
}

///mint upgraded nft in exchang of 1 step low level NFT
pub fn execute_mint_upgraded_nft(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_ids: Vec<String>,
) -> StdResult<Response> {
    let mut token_rarity = "".to_string();
    let mut tool_type = "".to_string();
    if token_ids.len() != 5 {
        return Err(StdError::generic_err("Not eligible"));
    }

    for token_id in token_ids.iter() {
        let token = if let Some(token) = tokens().may_load(deps.storage, token_id)? {
            token
        } else {
            return Err(StdError::generic_err("No token found"));
        };
        let mut template_key = token.tool_type.to_string();
        template_key.push_str(token.rarity.to_string().as_str());

        if token_rarity.is_empty() {
            token_rarity.push_str(&token.rarity);
        } else if token_rarity != token.rarity {
            return Err(StdError::generic_err(
                "Not eligible, kindly provide same rarity tokens",
            ));
        }
        if tool_type.is_empty() {
            tool_type.push_str(&token.tool_type);
        } else if tool_type != token.tool_type {
            return Err(StdError::generic_err(
                "Not eligible, kindly provide same tool type tokens",
            ));
        }
        _check_can_send(deps.as_ref(), &env, &info, &token)?;
        burn(deps.storage, token_id.to_string());
    }
    let upgraded_token_rarity =
        if let Some(upgraded_token_rarity) = RARITY_TYPES.may_load(deps.storage, token_rarity)? {
            upgraded_token_rarity
        } else {
            return Err(StdError::generic_err("No upgraded token found"));
        };
    let mut template_key = tool_type.to_string();
    template_key.push_str(&upgraded_token_rarity);
    let tool_template = if let Some(tool_template) =
        TOOL_TEMPLATE_MAP.may_load(deps.storage, template_key.to_string())?
    {
        tool_template
    } else {
        return Err(StdError::generic_err("No tool template found"));
    };
    let msg = MintMsg {
        owner: deps.api.addr_validate(&info.sender.to_string())?,
        name: tool_template.name,
        rarity: tool_template.rarity,
        pre_mint_tool: None,
        minting_count: None,
        tool_type,
    };
    mint(deps.storage, &env, &msg);
    Ok(Response::new())
}
