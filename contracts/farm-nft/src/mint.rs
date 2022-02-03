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
    AllNftInfoResponse, Cw20HookMsg, Cw721HookMsg, ExecuteMsg, Extension, InstantiateMsg,
    MigrateMsg, MintMsg, NftInfoResponse, QueryMsg,
};
use crate::state::{
    increment_tokens, num_tokens, tokens, Approval, Config, RewardToken, TokenInfo, CONFIG,
    CONTRACT_INFO, ITEM_TOKEN_MAPPING, NFT_NAMES, OPERATORS, REWARDS, REWARD_TOKEN, TOKEN_COUNT,
    TOKEN_ITEM_MAPPING, TOOL_TEMPLATE_MAP, USER_ENERGY_LEVEL, USER_ITEM_AMOUNT, USER_STAKED_INFO,
    distribute_amount,
};

use crate::contract::mint;

pub fn execute_mint_common_nft(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    tool_type: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let tool_template = if let Some(tool_template) =
        TOOL_TEMPLATE_MAP.may_load(deps.storage, tool_type.to_string())?
    {
        tool_template
    } else {
        return Err(ContractError::NotFound {});
    };
    let msg = MintMsg {
        owner: deps.api.addr_validate(&info.sender.to_string())?,
        name: tool_template.name,
        description: Some(tool_template.description),
        image: tool_template.image,
        rarity: tool_template.rarity,
        pre_mint_tool: None,
        minting_count: None,
        tool_type,
    };

    let mut user_addr = String::from(&info.sender.to_string());
    user_addr.push_str("gwood");
    let mut user_gwood_amount = if let Some(user_gwood_amount) =
        USER_ITEM_AMOUNT.may_load(deps.storage, user_addr.to_string())?
    {
        user_gwood_amount
    } else {
        return Err(ContractError::NotFound {});
    };
    if user_gwood_amount < tool_template.required_gwood_amount {
        return Err(ContractError::InSufficientFunds {});
    }
    user_gwood_amount -= tool_template.required_gwood_amount;
    let amount = tool_template.required_gwood_amount;
    distribute_amount(deps.storage, "gwood".to_string(), amount, &config, &env);
    // let burn_amount = amount.multiply_ratio(Uint128::from(25u128), Uint128::from(100u128));
    // let team_market_amount = amount.multiply_ratio(Uint128::from(10u128), Uint128::from(100u128));
    // let legal_amount = amount.multiply_ratio(Uint128::from(5u128), Uint128::from(100u128));
    // let contract_pool_amount = amount.multiply_ratio(Uint128::from(50u128), Uint128::from(100u128));
    // add_amount_in_item_address(deps.storage, config.legal_addr.to_string(), "gwood".to_string(), legal_amount);
    // add_amount_in_item_address(deps.storage, config.team_addr.to_string(), "gwood".to_string(), team_market_amount);
    // add_amount_in_item_address(deps.storage, config.market_addr.to_string(), "gwood".to_string(), team_market_amount);
    // add_amount_in_item_address(deps.storage, env.contract.address.to_string(), "gwood".to_string(), contract_pool_amount);
    // add_amount_in_item_address(deps.storage, config.burn_addr.to_string(), "gwood".to_string(), burn_amount);

    // USER_ITEM_AMOUNT.save(deps.storage, user_addr, &user_gwood_amount)?;
    // let mut user_addr = String::from(&info.sender.to_string());
    // user_addr.push_str("gfood");
    // let mut user_gfood_amount =
    //     if let Some(user_gfood_amount) = USER_ITEM_AMOUNT.may_load(deps.storage, user_addr.to_string())? {
    //         user_gfood_amount
    //     } else {
    //         return Err(ContractError::NotFound {});
    //     };
    // if user_gfood_amount < tool_template.required_gfood_amount {
    //     return Err(ContractError::InSufficientFunds {});
    // }

    // user_gfood_amount -=  tool_template.required_gfood_amount;
    // USER_ITEM_AMOUNT.save(deps.storage, user_addr, &user_gfood_amount)?;

    // let amount = tool_template.required_gfood_amount;
    // let burn_amount = amount.multiply_ratio(Uint128::from(25u128), Uint128::from(100u128));
    // let team_and_market_amount = amount.multiply_ratio(Uint128::from(10u128), Uint128::from(100u128));
    // let legal_amount = amount.multiply_ratio(Uint128::from(5u128), Uint128::from(100u128));
    // let contract_pool_amount = amount.multiply_ratio(Uint128::from(50u128), Uint128::from(100u128));

    // let mut legal_item_key = config.legal_addr.to_string();
    // legal_item_key.push_str("gfood");

    // let mut legal_item_amount =
    //     if let Some(legal_item_amount) = USER_ITEM_AMOUNT.may_load(deps.storage, legal_item_key)? {
    //         legal_item_amount
    //     } else {
    //         Uint128::zero()
    //     };
    // legal_item_amount += legal_amount;
    // USER_ITEM_AMOUNT.save(
    //     deps.storage,
    //     config.legal_addr.to_string(),
    //     &legal_item_amount,
    // )?;

    // let mut contract_item_key = env.contract.address.to_string();
    // contract_item_key.push_str("gfood");

    // let mut contract_item_amount =
    //     if let Some(contract_item_amount) = USER_ITEM_AMOUNT.may_load(deps.storage, contract_item_key)? {
    //         contract_item_amount
    //     } else {
    //         Uint128::zero()
    //     };
    // contract_item_amount += contract_pool_amount;
    // USER_ITEM_AMOUNT.save(
    //     deps.storage,
    //     config.legal_addr.to_string(),
    //     &contract_item_amount,
    // )?;

    // let mut team_item_key = config.team_addr.to_string();
    // team_item_key.push_str("gfood");
    // let mut team_item_amount =
    //     if let Some(team_item_amount) = USER_ITEM_AMOUNT.may_load(deps.storage, team_item_key)? {
    //         team_item_amount
    //     } else {
    //         Uint128::zero()
    //     };
    // team_item_amount += team_and_market_amount;
    // USER_ITEM_AMOUNT.save(
    //     deps.storage,
    //     config.team_addr.to_string(),
    //     &team_item_amount,
    // )?;

    //sending 10% to marketing address
    // let mut marketing_item_key = config.market_addr.to_string();
    // marketing_item_key.push_str("gfood");
    // let mut marketing_item_amount = if let Some(marketing_item_amount) =
    //     USER_ITEM_AMOUNT.may_load(deps.storage, marketing_item_key)?
    // {
    //     marketing_item_amount
    // } else {
    //     Uint128::zero()
    // };
    // marketing_item_amount += team_and_market_amount;
    // USER_ITEM_AMOUNT.save(
    //     deps.storage,
    //     config.market_addr.to_string(),
    //     &marketing_item_amount,
    // )?;

    let mut user_addr = String::from(&info.sender.to_string());
    user_addr.push_str("ggold");
    let mut user_ggold_amount = if let Some(user_ggold_amount) =
        USER_ITEM_AMOUNT.may_load(deps.storage, user_addr.to_string())?
    {
        user_ggold_amount
    } else {
        return Err(ContractError::NotFound {});
    };
    if user_ggold_amount < tool_template.required_ggold_amount {
        return Err(ContractError::InSufficientFunds {});
    }
    user_ggold_amount -= tool_template.required_ggold_amount;
    USER_ITEM_AMOUNT.save(deps.storage, user_addr, &user_ggold_amount)?;

    let amount = tool_template.required_ggold_amount;
    distribute_amount(deps.storage, "ggold".to_string(), amount, &config, &env);

    // let burn_amount = amount.multiply_ratio(Uint128::from(25u128), Uint128::from(100u128));
    // let team_market_amount = amount.multiply_ratio(Uint128::from(10u128), Uint128::from(100u128));
    // let legal_amount = amount.multiply_ratio(Uint128::from(5u128), Uint128::from(100u128));
    // let contract_pool_amount = amount.multiply_ratio(Uint128::from(50u128), Uint128::from(100u128));
    // add_amount_in_item_address(deps.storage, config.legal_addr.to_string(), "ggold".to_string(), legal_amount);
    // add_amount_in_item_address(deps.storage, config.team_addr.to_string(), "ggold".to_string(), team_market_amount);
    // add_amount_in_item_address(deps.storage, config.market_addr.to_string(), "ggold".to_string(), team_market_amount);
    // add_amount_in_item_address(deps.storage, env.contract.address.to_string(), "ggold".to_string(), contract_pool_amount);
    // add_amount_in_item_address(deps.storage, config.burn_addr.to_string(), "ggold".to_string(), burn_amount);

    let mut user_addr = String::from(&info.sender.to_string());
    user_addr.push_str("gstone");
    let mut user_gstone_amount = if let Some(user_gstone_amount) =
        USER_ITEM_AMOUNT.may_load(deps.storage, user_addr.to_string())?
    {
        user_gstone_amount
    } else {
        return Err(ContractError::NotFound {});
    };
    if user_gstone_amount < tool_template.required_gstone_amount {
        return Err(ContractError::InSufficientFunds {});
    }
    user_gstone_amount -= tool_template.required_gstone_amount;
    USER_ITEM_AMOUNT.save(deps.storage, user_addr, &user_gstone_amount)?;

    let amount = tool_template.required_gstone_amount;

    distribute_amount(deps.storage, "gstone".to_string(), amount, &config, &env);

    // let burn_amount = amount.multiply_ratio(Uint128::from(25u128), Uint128::from(100u128));
    // let team_market_amount = amount.multiply_ratio(Uint128::from(10u128), Uint128::from(100u128));
    // let legal_amount = amount.multiply_ratio(Uint128::from(5u128), Uint128::from(100u128));
    // let contract_pool_amount = amount.multiply_ratio(Uint128::from(50u128), Uint128::from(100u128));
    // add_amount_in_item_address(deps.storage, config.legal_addr.to_string(), "gstone".to_string(), legal_amount);
    // add_amount_in_item_address(deps.storage, config.team_addr.to_string(), "gstone".to_string(), team_market_amount);
    // add_amount_in_item_address(deps.storage, config.market_addr.to_string(), "gstone".to_string(), team_market_amount);
    // add_amount_in_item_address(deps.storage, env.contract.address.to_string(), "gstone".to_string(), contract_pool_amount);
    // add_amount_in_item_address(deps.storage, config.burn_addr.to_string(), "gstone".to_string(), burn_amount);

    mint(deps.storage, &env, &msg);

    Ok(Response::new())
}



// pub fn execute_mint_axe(
//     deps: DepsMut,
//     env: Env,
//     info: MessageInfo,
//     msg: Cw20ReceiveMsg,
// ) -> Result<Response, ContractError> {
//     if msg.amount < Uint128::from(2000u128) {
//         return Err(ContractError::NotEligible {});
//     }
//     let callback = CosmosMsg::Wasm(WasmMsg::Execute {
//         //sending reward to user
//         contract_addr: info.sender.to_string(),
//         msg: to_binary(&Cw20ExecuteMsg::Burn { amount: msg.amount })?,
//         funds: vec![],
//     });
//     let msg = MintMsg {
//         owner: deps.api.addr_validate(&msg.sender)?,
//         name: "Axe".to_string(),
//         description: Some("".to_string()),
//         image: "ipfs://QmVnu7JQVoDRqSgHBzraYp7Hy78HwJtLFi6nUFCowTGdzp/1.png".to_string(),
//         rarity: "axe".to_string(),
//         pre_mint_tool: None,
//         minting_count: None,
//         category: "Axe".to_string(),
//     };

//     mint(deps.storage, &env, &msg);

//     Ok(Response::new().add_message(callback))
// }

// pub fn execute_mint_fist_net(
//     deps: DepsMut,
//     env: Env,
//     info: MessageInfo,
//     msg: Cw20ReceiveMsg,
// ) -> Result<Response, ContractError> {
//     if msg.amount < Uint128::from(2000u128) {
//         return Err(ContractError::NotEligible {});
//     }

//     let callback = CosmosMsg::Wasm(WasmMsg::Execute {
//         //sending reward to user
//         contract_addr: info.sender.to_string(),
//         msg: to_binary(&Cw20ExecuteMsg::Burn { amount: msg.amount })?,
//         funds: vec![],
//     });
//     let msg = MintMsg {
//         owner: deps.api.addr_validate(&msg.sender)?,
//         name: "Salman".to_string(),
//         description: Some("".to_string()),
//         image: "ipfs://QmVnu7JQVoDRqSgHBzraYp7Hy78HwJtLFi6nUFCowTGdzp/1.png".to_string(),
//         rarity: "axe".to_string(),
//         pre_mint_tool: None,
//         minting_count: None,
//         category: "Axe".to_string(),
//     };

//     mint(deps.storage, &env, &msg);

//     Ok(Response::new().add_message(callback))
// }
