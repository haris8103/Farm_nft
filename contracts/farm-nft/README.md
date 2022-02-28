# FARM NFT Contract

## Overview
  Farm Nft is a game where you can interact with nft's in game. 

# Functions

```sh
pub fn instantiate():
```

This function will be used to Initialize the smart contract and some initial resources. It needs following parameter to 
initialize. 

- name of the contract
- symbol for the contract
- team_addr development team address
- market_addr market team address
- legal_addr legalization team address 
- burn_addr burning address
- stake_limit limit of staking items
- durability_from_start_time durability reduction after this time
- reserve_addr it is a reserve address for contract

```sh
pub fn execute_add_tool_type_names():
```

This function will be executed to register a tool_type name. Following are the parameter's name.

- tool_type tool_type to be register

```sh
pub fn execute_add_item_token():
```

This function will be executed to add game item token and actual item token address

- item_name name of the  game item token(e.g. gWood, gGold e.t.c.)
- item_token_addr actual item token address

```sh
pub fn execute_add_reward_token():
```

This function will be used to map tool with item (e.g. tool: "Axe" -> item "gWood")

- item_name item name (e.g. gWood) 
- tool_name tool name (e.g. Axe)
- mining_rate mining rate will be awarded after mining waiting time
- mining_waiting_time it is waiting time for mining

```sh
pub fn execute_mint():
```

This function will be used to mint the tool/pack NFT. Only admin/minter will be able to mint. Following are the parameters.

- owner it will be owner address who will own's NFT
- name nft tool/pack name
- rarity rarity of a tool NFT (e.g. Common, Uncommon and Mythic e.t.c)
- pre_mint_tool pre mint tool will be used for pack if pack is opened this will contains pre minted tool_type
- minting_count it is minting count  which will be used in batch mint function.
- tool_type tool type name.

```sh
pub fn execute_batch_mint():
```

This function will be used to mint the tool/pack NFT in batch. Only admin/minter will be able to mint. Following are the parameters.

- owner it will be owner address who will own's NFT
- name nft tool/pack name
- rarity rarity of a tool NFT (e.g. Common, Uncommon and Mythic e.t.c)
- pre_mint_tool pre mint tool will be used for pack if pack is opened this will contains pre minted tool_type
- minting_count it is minting count which tells the number of minting.
- tool_type tool type name.