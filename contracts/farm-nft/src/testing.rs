use crate::contract::{execute, instantiate};
use crate::mock::mock_dependencies;
use crate::msg::{Cw721HookMsg, ExecuteMsg, InstantiateMsg, MintMsg, ToolTemplateMsg};
use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::Coin;
use cosmwasm_std::{to_binary, Uint128};
use cw721::Cw721ReceiveMsg;

mod tests {
    use super::*;

    #[test]
    fn test_open_pack() {
        let mut deps = mock_dependencies(&[Coin {
            denom: "uusd".to_string(),
            amount: Uint128::from(200u128),
        }]);

        //let env = mock_env();
        let init_msg = InstantiateMsg {
            name: "Miners World".to_string(),
            symbol: "*****".to_string(),
            burn_addr: "terra1jayfkw90qt20jym2cxl9jktgr9t0lcn7rl3043".to_string(),
            team_addr: "terra1jayfkw90qt20jym2cxl9jktgr9t0lcn7rl3043".to_string(),
            market_addr: "terra1jsc62klfy7xszeyxkewjl0dglz7r6hf3hap82u".to_string(),
            legal_addr: "terra1n8dh5t0h8255j5nlsavyca52g6xjd0wfywdgwp".to_string(),
            stake_limit: 20,
            durability_from_start_time: 2592000,
            reserve_addr: "reserve_address".to_string(),
            repair_kit_waiting_time: 360u64,
        };
        let minter = mock_info(&"minter".to_string(), &[]);
        let user = mock_info(&"user1".to_string(), &[]);
        let _result = instantiate(deps.as_mut(), mock_env(), minter.clone(), init_msg).unwrap();

        let add_stakeable_token_msg = ExecuteMsg::AddToolTypeNames {
            tool_type: "Wood Miner".to_string(),
        };

        let _res = execute(
            deps.as_mut(),
            mock_env(),
            minter.clone(),
            add_stakeable_token_msg,
        )
        .unwrap();

        let add_stakeable_token_msg = ExecuteMsg::AddToolTypeNames {
            tool_type: "Food Miner".to_string(),
        };

        let _res = execute(
            deps.as_mut(),
            mock_env(),
            minter.clone(),
            add_stakeable_token_msg,
        )
        .unwrap();

        let add_stakeable_token_msg = ExecuteMsg::AddToolTypeNames {
            tool_type: "Gold Miner".to_string(),
        };

        let _res = execute(
            deps.as_mut(),
            mock_env(),
            minter.clone(),
            add_stakeable_token_msg,
        )
        .unwrap();

        let add_stakeable_token_msg = ExecuteMsg::AddToolTypeNames {
            tool_type: "Stone Miner".to_string(),
        };

        let _res = execute(
            deps.as_mut(),
            mock_env(),
            minter.clone(),
            add_stakeable_token_msg,
        )
        .unwrap();

        let add_stakeable_token_msg = ExecuteMsg::AddItemToken {
            item_token_addr: "woodaddr".to_string(),
            item_name: "gWood".to_string(),
        };

        let _res = execute(
            deps.as_mut(),
            mock_env(),
            minter.clone(),
            add_stakeable_token_msg,
        )
        .unwrap();

        let add_stakeable_token_msg = ExecuteMsg::AddItemToken {
            item_token_addr: "foodaddr".to_string(),
            item_name: "gFood".to_string(),
        };

        let _res = execute(
            deps.as_mut(),
            mock_env(),
            minter.clone(),
            add_stakeable_token_msg,
        )
        .unwrap();

        let add_stakeable_token_msg = ExecuteMsg::AddItemToken {
            item_token_addr: "Goldaddr".to_string(),
            item_name: "gGold".to_string(),
        };

        let _res = execute(
            deps.as_mut(),
            mock_env(),
            minter.clone(),
            add_stakeable_token_msg,
        )
        .unwrap();

        let add_stakeable_token_msg = ExecuteMsg::AddItemToken {
            item_token_addr: "Stoneaddr".to_string(),
            item_name: "gStone".to_string(),
        };

        let _res = execute(
            deps.as_mut(),
            mock_env(),
            minter.clone(),
            add_stakeable_token_msg,
        )
        .unwrap();

        let add_stakeable_token_msg = ExecuteMsg::AddToolTemplate({
            ToolTemplateMsg {
                tool_type: "Wood Miner".to_string(),
                name: "Wood Miner Pack".to_string(),
                description: "".to_string(),
                image: "ipfs://Qmcnz2b3XkMsMwXLnAD5qXz9cGAHWRr74wyBFm1qB6UHQW".to_string(),
                rarity: "Pack".to_string(),
                required_gwood_amount: Uint128::zero(),
                required_gfood_amount: Uint128::zero(),
                required_ggold_amount: Uint128::zero(),
                required_gstone_amount: Uint128::zero(),
                durability: 10,
            }
        });

        let _res = execute(
            deps.as_mut(),
            mock_env(),
            minter.clone(),
            add_stakeable_token_msg,
        )
        .unwrap();

        let add_stakeable_token_msg = ExecuteMsg::AddToolTemplate({
            ToolTemplateMsg {
                tool_type: "Wood Miner".to_string(),
                name: "Axe".to_string(),
                description: "".to_string(),
                image: "ipfs://Qmcnz2b3XkMsMwXLnAD5qXz9cGAHWRr74wyBFm1qB6UHQW".to_string(),
                rarity: "Common".to_string(),
                required_gwood_amount: Uint128::zero(),
                required_gfood_amount: Uint128::zero(),
                required_ggold_amount: Uint128::zero(),
                required_gstone_amount: Uint128::zero(),
                durability: 10,
            }
        });

        let _res = execute(
            deps.as_mut(),
            mock_env(),
            minter.clone(),
            add_stakeable_token_msg,
        )
        .unwrap();

        let add_stakeable_token_msg = ExecuteMsg::AddToolTemplate({
            ToolTemplateMsg {
                tool_type: "Stone Miner".to_string(),
                name: "Chisel".to_string(),
                description: "".to_string(),
                image: "ipfs://Qmcnz2b3XkMsMwXLnAD5qXz9cGAHWRr74wyBFm1qB6UHQW".to_string(),
                rarity: "Common".to_string(),
                required_gwood_amount: Uint128::zero(),
                required_gfood_amount: Uint128::zero(),
                required_ggold_amount: Uint128::zero(),
                required_gstone_amount: Uint128::zero(),
                durability: 10,
            }
        });

        let _res = execute(
            deps.as_mut(),
            mock_env(),
            minter.clone(),
            add_stakeable_token_msg,
        )
        .unwrap();

        let add_stakeable_token_msg = ExecuteMsg::AddToolTemplate({
            ToolTemplateMsg {
                tool_type: "Gold Miner".to_string(),
                name: "Hammer".to_string(),
                description: "".to_string(),
                image: "ipfs://Qmcnz2b3XkMsMwXLnAD5qXz9cGAHWRr74wyBFm1qB6UHQW".to_string(),
                rarity: "Common".to_string(),
                required_gwood_amount: Uint128::zero(),
                required_gfood_amount: Uint128::zero(),
                required_ggold_amount: Uint128::zero(),
                required_gstone_amount: Uint128::zero(),
                durability: 10,
            }
        });

        let _res = execute(
            deps.as_mut(),
            mock_env(),
            minter.clone(),
            add_stakeable_token_msg,
        )
        .unwrap();

        let add_stakeable_token_msg = ExecuteMsg::AddToolTemplate({
            ToolTemplateMsg {
                tool_type: "Gold Miner".to_string(),
                name: "Hammer".to_string(),
                description: "".to_string(),
                image: "ipfs://Qmcnz2b3XkMsMwXLnAD5qXz9cGAHWRr74wyBFm1qB6UHQW".to_string(),
                rarity: "Common".to_string(),
                required_gwood_amount: Uint128::zero(),
                required_gfood_amount: Uint128::zero(),
                required_ggold_amount: Uint128::zero(),
                required_gstone_amount: Uint128::zero(),
                durability: 10,
            }
        });

        let _res = execute(
            deps.as_mut(),
            mock_env(),
            minter.clone(),
            add_stakeable_token_msg,
        )
        .unwrap();

        let add_stakeable_token_msg = ExecuteMsg::AddToolTemplate({
            ToolTemplateMsg {
                tool_type: "Food Miner".to_string(),
                name: "Fish Net".to_string(),
                description: "".to_string(),
                image: "ipfs://Qmcnz2b3XkMsMwXLnAD5qXz9cGAHWRr74wyBFm1qB6UHQW".to_string(),
                rarity: "Common".to_string(),
                required_gwood_amount: Uint128::zero(),
                required_gfood_amount: Uint128::zero(),
                required_ggold_amount: Uint128::zero(),
                required_gstone_amount: Uint128::zero(),
                durability: 10,
            }
        });

        let _res = execute(
            deps.as_mut(),
            mock_env(),
            minter.clone(),
            add_stakeable_token_msg,
        )
        .unwrap();

        let add_distribution_msg = ExecuteMsg::AddRewardToken {
            item_name: "gWood".to_string(),
            tool_name: "Axe".to_string(),
            mining_rate: 100u64,
            mining_waiting_time: 100u64,
        };
        execute(
            deps.as_mut(),
            mock_env(),
            minter.clone(),
            add_distribution_msg,
        )
        .unwrap();

        let add_distribution_msg = ExecuteMsg::AddRewardToken {
            item_name: "gFood".to_string(),
            tool_name: "Chisel".to_string(),
            mining_rate: 100u64,
            mining_waiting_time: 100u64,
        };
        execute(
            deps.as_mut(),
            mock_env(),
            minter.clone(),
            add_distribution_msg,
        )
        .unwrap();

        let add_distribution_msg = ExecuteMsg::AddRewardToken {
            item_name: "gGold".to_string(),
            tool_name: "Hammer".to_string(),
            mining_rate: 100u64,
            mining_waiting_time: 100u64,
        };
        execute(
            deps.as_mut(),
            mock_env(),
            minter.clone(),
            add_distribution_msg,
        )
        .unwrap();

        let add_distribution_msg = ExecuteMsg::AddRewardToken {
            item_name: "gStone".to_string(),
            tool_name: "Fish Net".to_string(),
            mining_rate: 100u64,
            mining_waiting_time: 100u64,
        };
        execute(
            deps.as_mut(),
            mock_env(),
            minter.clone(),
            add_distribution_msg,
        )
        .unwrap();

        let add_distribution_msg = ExecuteMsg::Mint(MintMsg {
            name: "Wood Miner Pack".to_string(),
            tool_type: "Wood Miner".to_string(),
            pre_mint_tool: Some("Wood Miner".to_string()),
            //owner: mk.accAddress, //User
            owner: user.sender.clone(),
            rarity: "Pack".to_string(),
            minting_count: Some(0u64),
        });
        execute(
            deps.as_mut(),
            mock_env(),
            minter.clone(),
            add_distribution_msg,
        )
        .unwrap();

        let add_distribution_msg = ExecuteMsg::BatchMint(MintMsg {
            name: "Axe".to_string(),
            tool_type: "Wood Miner".to_string(),
            pre_mint_tool: None,
            owner: mock_env().contract.address.clone(),
            rarity: "Common".to_string(),
            minting_count: Some(4u64),
        });
        execute(
            deps.as_mut(),
            mock_env(),
            minter.clone(),
            add_distribution_msg,
        )
        .unwrap();

        let add_distribution_msg = ExecuteMsg::BatchMint(MintMsg {
            name: "Chisel".to_string(),
            tool_type: "Stone Miner".to_string(),
            pre_mint_tool: None,
            //owner: mk.accAddress, //User
            owner: mock_env().contract.address.clone(),
            rarity: "Common".to_string(),
            minting_count: Some(4u64),
        });
        execute(
            deps.as_mut(),
            mock_env(),
            minter.clone(),
            add_distribution_msg,
        )
        .unwrap();

        let add_distribution_msg = ExecuteMsg::BatchMint(MintMsg {
            name: "Hammer".to_string(),
            tool_type: "Gold Miner".to_string(),
            pre_mint_tool: None,
            //owner: mk.accAddress, //User
            owner: mock_env().contract.address.clone(),
            rarity: "Common".to_string(),
            minting_count: Some(4u64),
        });
        execute(
            deps.as_mut(),
            mock_env(),
            minter.clone(),
            add_distribution_msg,
        )
        .unwrap();

        let add_distribution_msg = ExecuteMsg::BatchMint(MintMsg {
            name: "Fish Net".to_string(),
            tool_type: "Food Miner".to_string(),
            pre_mint_tool: None,
            //owner: mk.accAddress, //User
            owner: mock_env().contract.address.clone(),
            rarity: "Common".to_string(),
            minting_count: Some(4u64),
        });
        execute(
            deps.as_mut(),
            mock_env(),
            minter.clone(),
            add_distribution_msg,
        )
        .unwrap();

        let stake_msg = ExecuteMsg::ReceiveNft(Cw721ReceiveMsg {
            sender: "user1".to_string(),
            token_id: "1".to_string(),
            msg: to_binary(&Cw721HookMsg::OpenPack {}).unwrap(),
        });

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info(&mock_env().contract.address.clone().to_string(), &[]),
            stake_msg,
        )
        .unwrap();
    }
}

// cargo test -- --show-output
