use cosmwasm_std::{
    entry_point, to_binary, Coin, Deps, DepsMut, Env, MessageInfo, Response,from_binary,Binary,
    StdResult, Uint128,CosmosMsg,WasmMsg,Decimal,BankMsg,Order,Pair
};

use cw2::set_contract_version;
use cw20::{ Cw20ExecuteMsg,Cw20ReceiveMsg};
use cw721::{Cw721ReceiveMsg, Cw721ExecuteMsg};

use crate::error::{ContractError};
use crate::msg::{ ExecuteMsg, InstantiateMsg, QueryMsg,SellNft, BuyNft};
use crate::state::{State,CONFIG,Offering, OFFERINGS,Asset,UserInfo};
use crate::package::{OfferingsResponse,QueryOfferingsResult};
use std::str::from_utf8;

const CONTRACT_NAME: &str = "Hope_Market_Place";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let state = State {
        owner:info.sender.to_string(),
        token_address:String::from("token_address"),
        nft_address :String::from("nft_address"),
        offering_id:0
    };
    CONFIG.save(deps.storage,&state)?;
    Ok(Response::default())
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
    ExecuteMsg::ReceiveNft(msg) =>execute_receive_nft(deps,env,info,msg),
    ExecuteMsg::Receive(msg) =>execute_receive(deps,env,info,msg),
    ExecuteMsg::BuyNft { offering_id } =>execute_buy_nft(deps,env,info,offering_id),
    ExecuteMsg::SetAdminsList { members } => execute_set_members(deps,env,info,members),
    ExecuteMsg::WithdrawNft { offering_id } => execute_withdraw(deps,env,info,offering_id),
    ExecuteMsg::SetTokenAddress {address} => execute_token_address(deps,env,info,address),
    ExecuteMsg::SetNftAddress { address } =>execute_nft_address(deps,env,info,address),
    ExecuteMsg::ChangeOwner { address } =>execute_change_owner(deps,env,info,address),
    }
}


fn execute_receive_nft(
     deps: DepsMut,
    _env:Env,
    info: MessageInfo,
    rcv_msg: Cw721ReceiveMsg,
)-> Result<Response, ContractError> {
    
    let mut state = CONFIG.load(deps.storage)?;
    
    if info.sender.to_string()!=state.nft_address{
        return Err(ContractError::WrongNFTContractError { });
    }

    let msg:SellNft = from_binary(&rcv_msg.msg)?;
    
    state.offering_id = state.offering_id + 1;
    CONFIG.save(deps.storage, &state)?;

    let off = Offering {
        token_id: rcv_msg.token_id,
        seller: deps.api.addr_validate(&rcv_msg.sender)?.to_string(),
        list_price: msg.list_price.clone(),
    };

    OFFERINGS.save(deps.storage, &state.offering_id.to_string(), &off)?;
    let price_string = format!("{} ", msg.list_price.amount);

    Ok(Response::new()
        .add_attribute("price_string", price_string)
    )
}

fn execute_receive(
    deps: DepsMut,
    _env:Env,
    info: MessageInfo,
    rcv_msg: Cw20ReceiveMsg,
)-> Result<Response, ContractError> {
    let state = CONFIG.load(deps.storage)?;

    if info.sender.to_string() != state.token_address{
        return Err(ContractError::WrongTokenContractError  { })
    }

    let msg:BuyNft = from_binary(&rcv_msg.msg)?;
    let off = OFFERINGS.load(deps.storage, &msg.offering_id)?;

    
    if off.list_price.denom != "hope".to_string(){
        return Err(ContractError::NotEnoughFunds  { })
    }

    if off.list_price.amount > rcv_msg.amount{
        return Err(ContractError::NotEnoughFunds  { })
    }

    OFFERINGS.remove( deps.storage, &msg.offering_id);


    Ok(Response::new()
        .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: state.nft_address.to_string(),
                funds: vec![],
                msg: to_binary(&Cw721ExecuteMsg::TransferNft {
                    recipient: deps.api.addr_validate(&rcv_msg.sender)?.to_string(),
                    token_id: off.token_id.clone(),
            })?,
        }))
        .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: state.token_address.to_string(),
                funds: vec![],
                msg: to_binary(&Cw20ExecuteMsg::Transfer { 
                    recipient: off.seller, 
                    amount: rcv_msg.amount })?,
        }))
)
}

fn execute_buy_nft(
    deps: DepsMut,
    _env:Env,
    info: MessageInfo,
    offering_id: String,
) -> Result<Response, ContractError> {
     let state = CONFIG.load(deps.storage)?;

   
    let off = OFFERINGS.load(deps.storage, &offering_id)?;

    let amount= info
        .funds
        .iter()
        .find(|c| c.denom == off.list_price.denom)
        .map(|c| Uint128::from(c.amount))
        .unwrap_or_else(Uint128::zero);

    if off.list_price.amount>amount{
        return Err(ContractError::NotEnoughFunds {  })
    }

    OFFERINGS.remove( deps.storage, &offering_id);


    Ok(Response::new()
        .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: state.nft_address.to_string(),
                funds: vec![],
                msg: to_binary(&Cw721ExecuteMsg::TransferNft {
                    recipient: info.sender.to_string(),
                    token_id: off.token_id.clone(),
            })?,
        }))
        .add_message(CosmosMsg::Bank(BankMsg::Send {
                to_address: off.seller,
                amount:vec![Coin{
                    denom:off.list_price.denom,
                    amount:amount
                }]
        }))
)
}

fn execute_withdraw(
    deps: DepsMut,
    _env:Env,
    info: MessageInfo,
    offering_id: String,
) -> Result<Response, ContractError> {
    let off = OFFERINGS.load(deps.storage,&offering_id)?;
    let state = CONFIG.load(deps.storage)?;

    if off.seller == info.sender.to_string(){
      
        OFFERINGS.remove(deps.storage,&offering_id);
        Ok(Response::new()
            .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: state.nft_address.to_string(),
                funds: vec![],
                msg: to_binary(&Cw721ExecuteMsg::TransferNft {
                    recipient: deps.api.addr_validate(&off.seller)?.to_string(),
                    token_id: off.token_id.clone(),
            })?,
        }))
    )
    }
    else {
        return Err(ContractError::Unauthorized {});
    }
    
}

fn execute_set_members(
     deps: DepsMut,
    _env:Env,
    info: MessageInfo,
    members: Vec<UserInfo>,
)->Result<Response,ContractError>{
    Ok(Response::default())
}

fn execute_token_address(
    deps: DepsMut,
    _env:Env,
    info: MessageInfo,
    address: String,
) -> Result<Response, ContractError> {
    let mut state = CONFIG.load(deps.storage)?;
    deps.api.addr_validate(&address)?;
    
    state.token_address = address;

    if state.owner != info.sender.to_string() {
        return Err(ContractError::Unauthorized {});
    }

    CONFIG.save(deps.storage, &state)?;
    Ok(Response::default())
}

fn execute_nft_address(
    deps: DepsMut,
    _env:Env,
    info: MessageInfo,
    address: String,
) -> Result<Response, ContractError> {
    let mut state = CONFIG.load(deps.storage)?;
    deps.api.addr_validate(&address)?;
    state.nft_address = address;
    
    if state.owner != info.sender.to_string() {
        return Err(ContractError::Unauthorized {});
    }

    CONFIG.save(deps.storage, &state)?;
    Ok(Response::default())
}


fn execute_change_owner(
    deps: DepsMut,
    _env:Env,
    info: MessageInfo,
    address: String,
) -> Result<Response, ContractError> {
    let mut state = CONFIG.load(deps.storage)?;

    if state.owner != info.sender.to_string() {
        return Err(ContractError::Unauthorized {});
    }
    deps.api.addr_validate(&address)?;
    state.owner = address;
    CONFIG.save(deps.storage,&state)?;
    Ok(Response::default())
}



#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetStateInfo {} => to_binary(&query_state_info(deps)?),
        QueryMsg::GetOfferings { }=> to_binary(&query_get_offerings(deps)?)
    }
}

pub fn query_state_info(deps:Deps) -> StdResult<State>{
    let state =  CONFIG.load(deps.storage)?;
    Ok(state)
}

pub fn query_get_offerings(deps:Deps) -> StdResult<OfferingsResponse>{
    let res: StdResult<Vec<QueryOfferingsResult>> = OFFERINGS
        .range(deps.storage, None, None, Order::Ascending)
        .map(|kv_item| parse_offering(deps, kv_item))
        .collect();
    Ok(OfferingsResponse {
        offerings: res?, // Placeholder
    })
}

fn parse_offering(
    deps:Deps,
    item: StdResult<Pair<Offering>>,
) -> StdResult<QueryOfferingsResult> {
    item.and_then(|(k, offering)| {
        let id = from_utf8(&k)?;
        Ok(QueryOfferingsResult {
            id: id.to_string(),
            token_id: offering.token_id,
            list_price: offering.list_price,
            seller: deps.api.addr_validate(&offering.seller)?.to_string(),
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{ CosmosMsg, Coin};

    #[test]
    fn testing() {
        //Instantiate
        let mut deps = mock_dependencies(&[]);
        let instantiate_msg = InstantiateMsg {
        };
        let info = mock_info("creator", &[]);
        let res = instantiate(deps.as_mut(), mock_env(), info, instantiate_msg).unwrap();
        assert_eq!(0, res.messages.len());
        let state = query_state_info(deps.as_ref()).unwrap();
        assert_eq!(state.owner,"creator".to_string());

        //Change Owner

        let info = mock_info("creator", &[]);
        let msg = ExecuteMsg::ChangeOwner { address:"owner".to_string()};
        execute(deps.as_mut(), mock_env(), info, msg).unwrap();
         let state = query_state_info(deps.as_ref()).unwrap();
        assert_eq!(state.owner,"owner".to_string());

        //Change Token Contract Address

        let info = mock_info("owner", &[]);
        let msg = ExecuteMsg::SetTokenAddress  { address:"token_address1".to_string()};
        execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        let state = query_state_info(deps.as_ref()).unwrap();
        assert_eq!(state.token_address,"token_address1".to_string());

        //Change NFT contract Address

        let info = mock_info("owner", &[]);
        let msg = ExecuteMsg::SetNftAddress  { address:"nft_address1".to_string()};
        execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        let state = query_state_info(deps.as_ref()).unwrap();
        assert_eq!(state.nft_address,"nft_address1".to_string());
        
        //Send NFT to marketplace contract

        let cw721_msg = SellNft{
            list_price:Asset{
                denom:"ujuno".to_string(),
                amount:Uint128::new(2)
            }
        };

        let info = mock_info("nft_address1", &[]);
        let msg = ExecuteMsg::ReceiveNft(Cw721ReceiveMsg{
            sender:"owner1".to_string(),
            token_id:"Hope.1".to_string(),
            msg:to_binary(&cw721_msg).unwrap()
        });
        let res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
        assert_eq!(0,res.messages.len());

        let msg = ExecuteMsg::ReceiveNft(Cw721ReceiveMsg{
            sender:"owner2".to_string(),
            token_id:"Hope.2".to_string(),
            msg:to_binary(&cw721_msg).unwrap()
        });
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0,res.messages.len());

        let nft_market_datas = query_get_offerings(deps.as_ref()).unwrap();
        assert_eq!(nft_market_datas.offerings,
            vec![
                QueryOfferingsResult{
                    id :"1".to_string(),
                    token_id:"Hope.1".to_string(),
                    seller : "owner1".to_string(),
                    list_price:Asset { 
                        denom: "ujuno".to_string(),
                        amount: Uint128::new(2) 
                    }
                },
                QueryOfferingsResult{
                    id :"2".to_string(),
                    token_id:"Hope.2".to_string(),
                    seller : "owner2".to_string(),
                    list_price:Asset { 
                        denom: "ujuno".to_string(),
                        amount: Uint128::new(2) 
                    }
                }
            ]
        );

        //Withdraw nft from market place

        let info = mock_info("owner1", &[]);
        let msg = ExecuteMsg::WithdrawNft { offering_id: "1".to_string() };
        let res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
        assert_eq!(1,res.messages.len());
        assert_eq!(res.messages[0].msg, CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: "nft_address1".to_string(),
            funds: vec![],
            msg: to_binary(&Cw721ExecuteMsg::TransferNft {
                    recipient: "owner1".to_string(),
                    token_id: "Hope.1".to_string(),
            }).unwrap(),
        }));      
        
        let nft_market_datas = query_get_offerings(deps.as_ref()).unwrap();
        assert_eq!(nft_market_datas.offerings,
            vec![
                QueryOfferingsResult{
                    id :"2".to_string(),
                    token_id:"Hope.2".to_string(),
                    seller : "owner2".to_string(),
                    list_price:Asset { 
                        denom: "ujuno".to_string(),
                        amount: Uint128::new(2) 
                    }
                }
            ]
        );

        //Send NFT to marketplace contract

        let cw721_msg = SellNft{
            list_price:Asset{
                denom:"hope".to_string(),
                amount:Uint128::new(10)
            }
        };

        let info = mock_info("nft_address1", &[]);
        let msg = ExecuteMsg::ReceiveNft(Cw721ReceiveMsg{
            sender:"owner3".to_string(),
            token_id:"Hope.3".to_string(),
            msg:to_binary(&cw721_msg).unwrap()
        });
        let res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let nft_market_datas = query_get_offerings(deps.as_ref()).unwrap();
        assert_eq!(nft_market_datas.offerings,
            vec![            
                QueryOfferingsResult{
                    id :"2".to_string(),
                    token_id:"Hope.2".to_string(),
                    seller : "owner2".to_string(),
                    list_price:Asset { 
                        denom: "ujuno".to_string(),
                        amount: Uint128::new(2) 
                    }
                }, QueryOfferingsResult{
                    id :"3".to_string(),
                    token_id:"Hope.3".to_string(),
                    seller : "owner3".to_string(),
                    list_price:Asset { 
                        denom: "hope".to_string(),
                        amount: Uint128::new(10) 
                    }
                }
            ]
        );

        //Buy nft using token

        let cw20_msg = BuyNft{
             offering_id:"3".to_string()
        };

        let info = mock_info("token_address1", &[]);
        let msg = ExecuteMsg::Receive(Cw20ReceiveMsg{
            sender:"buyer".to_string(),
            amount:Uint128::new(10),
            msg:to_binary(&cw20_msg).unwrap()
        });
        let res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
        assert_eq!(2,res.messages.len());
        assert_eq!(res.messages[0].msg, CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: "nft_address1".to_string(),
            funds: vec![],
            msg: to_binary(&Cw721ExecuteMsg::TransferNft {
                    recipient: "buyer".to_string(),
                    token_id: "Hope.3".to_string(),
            }).unwrap(),
        }));
         assert_eq!(res.messages[1].msg, CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: "token_address1".to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                    recipient: "owner3".to_string(),
                    amount:Uint128::new(10)
            }).unwrap(),
        }));

        let nft_market_datas = query_get_offerings(deps.as_ref()).unwrap();
        assert_eq!(nft_market_datas.offerings,
            vec![            
                QueryOfferingsResult{
                    id :"2".to_string(),
                    token_id:"Hope.2".to_string(),
                    seller : "owner2".to_string(),
                    list_price:Asset { 
                        denom: "ujuno".to_string(),
                        amount: Uint128::new(2) 
                    }
                }
            ]
        );

        //Buy nft using stable coin 

        let info = mock_info("buyer2", &[Coin{
            denom:"ujuno".to_string(),
            amount:Uint128::new(2)
        }]);
        let msg = ExecuteMsg::BuyNft { offering_id: "2".to_string() };
        let res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
        assert_eq!(res.messages.len(),2);
        assert_eq!(res.messages[0].msg,CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: state.nft_address.to_string(),
                funds: vec![],
                msg: to_binary(&Cw721ExecuteMsg::TransferNft {
                    recipient: "buyer2".to_string(),
                    token_id: "Hope.2".to_string(),
            }).unwrap()
        }));
        assert_eq!(res.messages[1].msg,CosmosMsg::Bank(BankMsg::Send {
                to_address: "owner2".to_string(),
                amount:vec![Coin{
                    denom:"ujuno".to_string(),
                    amount:Uint128::new(2)
                }]
        }));

        let nft_market_datas = query_get_offerings(deps.as_ref()).unwrap();
        assert_eq!(nft_market_datas.offerings,
            vec![]
        );
    }
}
    
  