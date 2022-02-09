use cosmwasm_std::StdError;
use thiserror::Error;

// TODO finish implementation of custom errors
#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("token_id already claimed")]
    Claimed {},

    #[error("Cannot set approval that is already expired")]
    Expired {},

    #[error("No token found")]
    NotFound {},

    #[error("Not eligible")]
    NotEligible {},

    #[error("Already exists")]
    AlreadyExisits {},

    #[error("No Reward Token Found")]
    NoRewardTokenFound {},

    #[error("Limit reached")]
    LimitReached {},

    #[error("Time not reached")]
    TimeNotReached {},

    #[error("No Energy")]
    NoEnergy {},

    #[error("Not enough Energy")]
    NotEnoughEnergy {},

    #[error("Insufficient funds")]
    InSufficientFunds {},

    #[error("No Token available for distribute")]
    NoTokenAvailableForDistribute {},
}

impl From<cw721_base::ContractError> for ContractError {
    fn from(err: cw721_base::ContractError) -> Self {
        match err {
            cw721_base::ContractError::Std(error) => ContractError::Std(error),
            cw721_base::ContractError::Unauthorized {} => ContractError::Unauthorized {},
            cw721_base::ContractError::Claimed {} => ContractError::Claimed {},
            cw721_base::ContractError::Expired {} => ContractError::Expired {},
        }
    }
}
