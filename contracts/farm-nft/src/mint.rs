#[cfg(not(feature = "library"))]
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response, Uint128};

use crate::contract::{_check_can_send, burn, mint};
use crate::error::ContractError;
use crate::msg::MintMsg;
use crate::state::{
    distribute_amount, tokens, CONFIG, GAME_DEV_TOKENS_NAME, RARITY_TYPES, TOOL_TEMPLATE_MAP,
    USER_ITEM_AMOUNT,
};

/// mint common rarity type NFT in exchange game dev tokens
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
    let game_dev_token_set = GAME_DEV_TOKENS_NAME.load(deps.storage)?;

    for (index, game_dev_token_name) in game_dev_token_set.into_iter().enumerate() {
        let mut user_addr = String::from(&info.sender.to_string());
        user_addr.push_str(&game_dev_token_name);
        let mut item_required_amount = if let Some(item_required_amount) =
            USER_ITEM_AMOUNT.may_load(deps.storage, game_dev_token_name.to_string())?
        {
            item_required_amount
        } else {
            Uint128::zero()
        };
        if item_required_amount < *tool_template.required_amount.get(index).unwrap() {
            return Err(ContractError::InSufficientFunds {});
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
