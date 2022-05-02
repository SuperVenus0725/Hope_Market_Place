
use cw20::Cw20ReceiveMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use crate::state::{Asset,UserInfo};
use cosmwasm_std::{Decimal, Uint128};
use cw721::Cw721ReceiveMsg;


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub royalty_portion:Decimal
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
 ReceiveNft(Cw721ReceiveMsg),
 Receive(Cw20ReceiveMsg),
 SetAdminsList{members:Vec<UserInfo>},
 ChangeRoyaltyPortion{royalty_portion:Decimal},
 BuyNft{offering_id:String},
 WithdrawNft{offering_id:String},
 ChangeOwner{address:String},
 SetTokenAddress{address:String},
 SetNftAddress { address:String},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// Returns a human-readable representation of the arbiter.
    GetStateInfo {},
    GetOfferings{},
    GetMembers{}
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct SellNft {
    pub list_price: Asset,
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct BuyNft {
    pub offering_id: String,
}
