# FARM NFT Contract

## Overview
  Farm Nft is a game where you can interact with nft's in game. 

# Functions

```sh
pub fn instantiate():
```

This function will be used to Initialize the smart contract and some initial resources. It needs following parameter to 
initialize. 

- admin of the contract
- ust_address from user which will be sent by user to but pack
- reserve_addr of the contract
- pack_rate rate of the pack
- nft_contract_address nft contract address


```sh
pub fn execute_receive_cw20():
```

This function will be used when user sent item tokens to contract.

- msg it is Cw20ReceiveMsg which contains the enum name (e.g. PackFood ,PackGold ,PackStone and PackWood ) , sender info and amount
- tool_type it is type of pre_mint_tool
