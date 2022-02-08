#[cfg(not(feature = "library"))]
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response, Uint128};

use crate::contract::{_check_can_send, burn, mint};
use crate::error::ContractError;
use crate::msg::MintMsg;
use crate::state::{
    distribute_amount, tokens, CONFIG, RARITY_TYPES, TOOL_TEMPLATE_MAP, USER_ITEM_AMOUNT,
};

pub fn execute_mint_common_nft(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    tool_type: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let mut template_key = tool_type.to_string();
    template_key.push_str("Common");
    let tool_template = if let Some(tool_template) =
        TOOL_TEMPLATE_MAP.may_load(deps.storage, template_key.to_string())?
    {
        tool_template
    } else {
        return Err(ContractError::NotFound {});
    };
    let msg = MintMsg {
        owner: deps.api.addr_validate(&info.sender.to_string())?,
        name: tool_template.name,
        rarity: tool_template.rarity,
        pre_mint_tool: None,
        minting_count: None,
        tool_type,
    };

    let mut user_addr = String::from(&info.sender.to_string());
    user_addr.push_str("gWood");
    let mut user_gwood_amount = if let Some(user_gwood_amount) =
        USER_ITEM_AMOUNT.may_load(deps.storage, user_addr.to_string())?
    {
        user_gwood_amount
    } else {
        Uint128::zero()
    };
    if user_gwood_amount < tool_template.required_gwood_amount {
        return Err(ContractError::InSufficientFunds {});
    }
    user_gwood_amount -= tool_template.required_gwood_amount;
    let amount = tool_template.required_gwood_amount;
    distribute_amount(deps.storage, "gWood".to_string(), amount, &config, &env);

    let mut user_addr = String::from(&info.sender.to_string());
    user_addr.push_str("gGold");
    let mut user_ggold_amount = if let Some(user_ggold_amount) =
        USER_ITEM_AMOUNT.may_load(deps.storage, user_addr.to_string())?
    {
        user_ggold_amount
    } else {
        Uint128::zero()
    };
    if user_ggold_amount < tool_template.required_ggold_amount {
        return Err(ContractError::InSufficientFunds {});
    }
    user_ggold_amount -= tool_template.required_ggold_amount;
    USER_ITEM_AMOUNT.save(deps.storage, user_addr, &user_ggold_amount)?;

    let amount = tool_template.required_ggold_amount;
    distribute_amount(deps.storage, "gGold".to_string(), amount, &config, &env);
    let mut user_addr = String::from(&info.sender.to_string());
    user_addr.push_str("gStone");
    let mut user_gstone_amount = if let Some(user_gstone_amount) =
        USER_ITEM_AMOUNT.may_load(deps.storage, user_addr.to_string())?
    {
        user_gstone_amount
    } else {
        Uint128::zero()
    };
    if user_gstone_amount < tool_template.required_gstone_amount {
        return Err(ContractError::InSufficientFunds {});
    }
    user_gstone_amount -= tool_template.required_gstone_amount;
    USER_ITEM_AMOUNT.save(deps.storage, user_addr, &user_gstone_amount)?;

    let amount = tool_template.required_gstone_amount;

    distribute_amount(deps.storage, "gStone".to_string(), amount, &config, &env);

    mint(deps.storage, &env, &msg);

    Ok(Response::new())
}

pub fn execute_mint_special_nft(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_ids: Vec<String>,
) -> Result<Response, ContractError> {
    let mut token_rarity = "".to_string();
    let mut tool_type = "".to_string();
    if token_ids.len() != 5 {
        return Err(ContractError::NotEligible {});
    }

    for token_id in token_ids.iter() {
        let token = if let Some(token) = tokens().may_load(deps.storage, token_id)? {
            token
        } else {
            return Err(ContractError::NotFound {});
        };
        let mut template_key = token.tool_type.to_string();
        template_key.push_str(token.rarity.to_string().as_str());

        if token_rarity.is_empty() {
            token_rarity.push_str(&token.rarity);
        } else if token_rarity != token.rarity {
            return Err(ContractError::NotEligible {});
        }
        if tool_type.is_empty() {
            tool_type.push_str(&token.tool_type);
        } else if tool_type != token.tool_type {
            return Err(ContractError::NotEligible {});
        }
        _check_can_send(deps.as_ref(), &env, &info, &token)?;
        burn(deps.storage, token_id.to_string());
    }
    let upgraded_token_rarity =
        if let Some(upgraded_token_rarity) = RARITY_TYPES.may_load(deps.storage, token_rarity)? {
            upgraded_token_rarity
        } else {
            return Err(ContractError::NotFound {});
        };
    let mut template_key = tool_type.to_string();
    template_key.push_str(&upgraded_token_rarity);
    let tool_template = if let Some(tool_template) =
        TOOL_TEMPLATE_MAP.may_load(deps.storage, template_key.to_string())?
    {
        tool_template
    } else {
        return Err(ContractError::NotFound {});
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
