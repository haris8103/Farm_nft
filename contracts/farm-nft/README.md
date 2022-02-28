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

- owner it will be owner address who will own's NFT.
- name nft tool/pack name.
- rarity rarity of a tool NFT (e.g. Common, Uncommon and Mythic e.t.c).
- pre_mint_tool pre mint tool will be used for pack if pack is opened this will contains pre minted tool_type.
- minting_count it is minting count which tells the number of minting.
- tool_type tool type name.

```sh
pub fn execute_transfer_nft():
```

This function will be used to transfer the nft. Following are the parameters

- recipient address to whom it will be transfered
- token_id id of the NFT

```sh
pub fn execute_send_nft():
```

This function will be used when user want to send NFT to the contract for staking or open pack.

- contract address of the contract
- token_id id of the NFT
- msg it contains the name of the receiving method

```sh
pub fn execute_receive_cw721():
```

This function will be used for when user send's NFT to contract than contract will call the receive message.

- msg it is Cw721ReceiveMsg which contains the enum name (e.g. OpenPack) , sender info and token id

```sh
pub fn execute_open_pack():
```

This function will be used to open pack nft and mint user random nft tools against it.

- msg it is Cw721ReceiveMsg which contains the enum name (e.g. OpenPack) , sender info and token id



```sh
pub fn execute_stake():
```

This function will be used to stake the tool to mine items.

- msg it is Cw721ReceiveMsg which contains the enum name (e.g. Stake) , sender info and token id

```sh
pub fn execute_burn():
```

This function will burn the nft.

- token_id to burn/delete from contract.

```sh
pub fn execute_claim_reward():
```

This function will be used to claim the mining reward (e.g. gWood).

- token_id to claim reward.

```sh
pub fn execute_unstake():
```

This function will be used to stake to claim it rewards.

- token_id to stake.


```sh
pub fn execute_receive_cw20():
```

This function will be used when user sent item tokens to contract.

- msg it is Cw20ReceiveMsg which contains the enum name (e.g. Deposit and AdminDeposit) , sender info and amount


```sh
pub fn execute_deposit():
```

This function will be used when user wants to buy game item token (e.g. gWood).

- msg it is Cw20ReceiveMsg which contains the enum name (e.g. Deposit and AdminDeposit) , sender info and amount


```sh
pub fn execute_admin_deposit():
```

This function will be used when admin wants to buy game item token (e.g. gWood).

- msg it is Cw20ReceiveMsg which contains the enum name (e.g. Deposit and AdminDeposit) , sender info and amount


```sh
pub fn execute_refill_energy():
```

This function will be used when user wants to refill energy in exchange of game item token (gFood).

- food_item_amount to refill energy it requires

```sh
pub fn execute_withdraw():
```

This function will be used when user wants to withdraw game item token in exchange of actual tokens.

- item_name it will contains the item name for which  user wants to exchange
- amount it will be  the amount of the items.

```sh
pub fn execute_mint_common_nft():
```

This function will be used when user wants to mint common NFT in exchange of game tokens.

- tool_type tool type to mint the tool


```sh
pub fn execute_mint_upgraded_nft():
```

This function will be used when user upgrade a tool to upgraded tools in exchange of that type of tools.

- token_ids token ids of tools to upgrade into a new tool

```sh
pub fn execute_transfer_reserve_amount();
```

This function is used to transfer reserve amount to contract.


```sh
pub fn execute_transfer_tool_pack();
```

This function is used to transfer when user wants to buy a tool pack.


- recipient recipient address of the user who wants to buy the pack
- tool_type to buy the pack of that type of pre-minted tool.
