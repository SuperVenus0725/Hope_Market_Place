use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Sum of portion is not 1 ")]
    PortionError{},

    #[error("NFT contract Error")]
    WrongNFTContractError{},
    
    #[error("Token contract Error")]
    WrongTokenContractError{},

     #[error("No data")]
    NoData{},

    #[error("Not Enough Funds")]
    NotEnoughFunds {},

    #[error("Escrow expired (end_height {end_height:?} end_time {end_time:?})")]
    Expired {
        end_height: Option<u64>,
        end_time: Option<u64>,
    },

    #[error("Escrow not expired")]
    NotExpired {},
}
