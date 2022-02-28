# FARM NFT Contract

## Overview
  Farm Nft is a game where you can interact with nft's in game. 

# Functions

```sh
pub fn instantiate():
```

This function will be used to Initialize the smart contract. It needs following parameter to 
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
pub fn execute_update_config():
```