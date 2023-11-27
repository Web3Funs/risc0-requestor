use std::{str::FromStr, sync::Arc, collections::VecDeque};
use log::*;
use core::str;
use rand::seq::SliceRandom;
use serde_derive::{Deserialize,Serialize};
use ethereum_private_key_to_address::PrivateKey;
use chrono::{Duration, Utc};

use reqwest::Client;

use web3::{
    ethabi::{ethereum_types::U256,Function, ParamType, Param, StateMutability, Token},
    types::{Address,Bytes, TransactionParameters}, 
};

use web3::types::BlockNumber::Latest;

use lazy_static::lazy_static;

lazy_static! {
    pub static ref PRIV_KEY: tokio::sync::Mutex<String> = {      //priv_key
        tokio::sync::Mutex::new(String::from(""))
    };
    pub static ref RELAYER_URL: tokio::sync::Mutex<String> = {   //relayer rpc url
        tokio::sync::Mutex::new(String::from(""))
    };
    pub static ref TASK_MSG_QUEUE: Arc<tokio::sync::Mutex<VecDeque<String>>> = {
        Arc::new(tokio::sync::Mutex::new(VecDeque::new()))
    };
    pub static ref CONTRACT: tokio::sync::Mutex<String> = {      //contract
        tokio::sync::Mutex::new(String::from(""))
      };
}


///config chain urls
pub const SEPOLIA_CHAIN_URLS: [&str; 1] = [
    "https://eth-sepolia.g.alchemy.com/v2/kMO8lL7g44IJOGR-Om-xxxxxxxx",
];


//Onchain paramter
pub const  GAS_UPPER : &str = "1000000";


//Dummy task info
const REWARD_TOKEN:&str="0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2";
const REWARD:u64 = 10000;
const LIABILITY_TOKEN:&str="0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2";
const LIABILITY:u64 = 10000;
const LIABILITY_WINDOW:u64=36000;

#[derive(Debug, Serialize, Deserialize)]
struct RpcRequest {
    jsonrpc: String,
    method: String,
    params: Vec<String>,
    id: u64,
}

#[derive(Debug, Serialize, Deserialize,Default,Clone)]
struct TaskResponse {
    pub prover: String,
    pub instance: String,
    pub reward_token: String,
    pub reward: u64,
    pub liability_window: u64,
    pub liability_token: String,
    pub liability: u64,
    pub expiry: u64,
    pub signature: String,
}


#[derive(Debug, Serialize, Deserialize)]
struct RpcResponse {
    jsonrpc: String,
    result: String,
    id: u64,
}

/// get the account nonce value
pub async fn get_nonce(addr:Address) -> U256{
    loop {
        for url in SEPOLIA_CHAIN_URLS.iter() {
            let transport = match web3::transports::Http::new(&url){
                Ok(r)=>{r},
                Err(_e) => {
                    continue;
                },
            };
            let web3 = web3::Web3::new(transport);
            info!("addr is {:?}",addr);
            let nonce= match web3.eth().transaction_count(addr,Some(Latest)).await{
                Ok(r)=>{r},
                Err(_e) => {
                    continue;
                },
            };
            info!("nonce value is {:?}",nonce.clone());
            return nonce
    }
 }
}

/// 1.1 multiple of the network gas
pub async fn gas_price() -> U256{
    loop {
        for url in SEPOLIA_CHAIN_URLS.iter() {
            let transport = match web3::transports::Http::new(&url){
                Ok(r)=>{r},
                Err(_e) => {
                    continue;
                },
            };
            let web3 = web3::Web3::new(transport);
            let gas_price= match web3.eth().gas_price().await{
                Ok(r) => r,
                Err(_) => continue,
            };

            let upper_gas = (gas_price.as_u64())*(100)/(90)*(10);
            info!("gas price value is {:?}",upper_gas.clone());
            return U256::from_dec_str(&upper_gas.to_string()).unwrap()      
    }
 }
}

/// submit proof data to sepolia chain
pub async fn submit_task(  
    instance:Bytes,
    prover:Address,
    reward_token:Address,
    reward_amount:U256,
    liability_window:u64,
    liability_token:Address,
    liability_amount:U256,
    expiry:u64,
    signature:Bytes
) -> Result<String, String> { 

    let url_str = SEPOLIA_CHAIN_URLS.choose(&mut rand::thread_rng()).unwrap();
    let transport = web3::transports::Http::new(url_str).unwrap();
    let web3 = web3::Web3::new(transport);

    let ctr = CONTRACT.lock().await;
    let ctr_addr = (*ctr).clone();
    let contract_address = Address::from_str(ctr_addr.as_str()).unwrap();

    let func = Function {
        name: "submitTask".to_owned(),
        inputs: vec![
            Param { name: "instance".to_owned(), kind: ParamType::Bytes, internal_type: None },
            Param { name: "prover".to_owned(), kind: ParamType::Address, internal_type: None },
            Param { name: "rewardToken".to_owned(), kind: ParamType::Address, internal_type: None }, 
            Param { name: "rewardAmount".to_owned(), kind: ParamType::Uint(256), internal_type: None }, 
            Param { name: "liabilityWindow".to_owned(), kind: ParamType::Uint(64), internal_type: None }, 
            Param { name: "liabilityToken".to_owned(), kind: ParamType::Address, internal_type: None },
            Param { name: "liabilityAmount".to_owned(), kind: ParamType::Uint(256), internal_type: None },
            Param { name: "expiry".to_owned(), kind: ParamType::Uint(64), internal_type: None },
            Param { name: "signature".to_owned(), kind: ParamType::Bytes, internal_type: None },
             
        ],
        outputs: vec![],
        constant: Some(false),
        state_mutability: StateMutability::Payable,
    };

      //enocde send tx input parameters
    let mut data_vec_input:Vec<Token>=Vec::new();
    data_vec_input.push(Token::Bytes(instance.0));
    data_vec_input.push(Token::Address(prover));
    data_vec_input.push(Token::Address(reward_token));
    data_vec_input.push(Token::Uint(reward_amount));
    data_vec_input.push(Token::Uint(liability_window.into()));
    data_vec_input.push(Token::Address(liability_token));
    data_vec_input.push(Token::Uint(liability_amount));
    data_vec_input.push(Token::Uint(expiry.into()));
    data_vec_input.push(Token::Bytes(signature.0));

    let tx_data = func.encode_input(&data_vec_input).unwrap();

    let priv_key = PRIV_KEY.lock().await;
    let key = (*priv_key).clone();
    let prvk = web3::signing::SecretKey::from_str(key.as_str()).unwrap();
    let private_key = PrivateKey::from_str(key.as_str()).unwrap();
    let addr = private_key.address();

    let tx_object = TransactionParameters {
        to: Some(contract_address),
        gas_price:Some(gas_price().await), 
        gas:U256::from_dec_str(GAS_UPPER).unwrap(),
        nonce:Some(get_nonce(Address::from_str(addr.as_str()).unwrap()).await),
        data:Bytes(tx_data),
        ..Default::default()
    };
        //send tx to network
    loop {
        let signed = match web3.accounts().sign_transaction(tx_object.clone(), &prvk).await {
            Ok(r) => {
                r
            },
            Err(_) => {
                continue
            }
        };

        let result = match web3.eth().send_raw_transaction(signed.raw_transaction).await{
            Ok(r )=> r,
            Err(e) => {
                return Ok(e.to_string())
            },
        };
            
        info!("invoke a tx hash is : {:?}",result);
        return Ok(hex::encode(result.as_bytes()))
    }
}

pub async fn dummy_task() -> Result<String, String> {   //TBD
  info!("start to send dummy_task");
  match assign_task("4.2 5.4 5.5#83e369c7c2e9c0afee6f754505da85e128545ede909608ee33f3431dac7266dc".to_string()).await{ //replace one task parameter String
    Ok(_) => {
        Ok("dummy task send success".to_string())
    },
    Err(_) => {
        error!("dummy task generate failed");
        Err("dummy task send failed".to_string())
    },
}
}

pub async fn assign_task(instance:String)  -> Result<(), String> {  //TBD
    let client = Client::new();
    let request = RpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "ReceiveTask".to_string(),
        params: vec![instance.to_string(), LIABILITY_WINDOW.to_string(),LIABILITY_TOKEN.to_string(),LIABILITY.to_string(),REWARD_TOKEN.to_string(),REWARD.to_string()],
        id: 1,
    };

    let relayer_url = RELAYER_URL.lock().await;
    let relayer_endpoint = (*relayer_url).clone();

    // let response: RpcResponse = client
    //     .post(relayer_endpoint.clone()) //relayer rpc address
    //     .json(&request)
    //     .send()
    //     .await.unwrap()
    //     .json()
    //     .await.unwrap();

    let response_res= match client
        .post(relayer_endpoint) //relayer rpc address
        .json(&request)
        .send()
        .await{
            Ok(r) => r,
            Err(_) => return Err("invode relayer failed".to_string()),
        };
    let response:RpcResponse=match response_res.json().await{
        Ok(r) => r,
        Err(_) => return Err("invode relayer failed".to_string()),
    };

    let task_response:TaskResponse=match serde_json::from_str(response.result.as_str()){
        Ok(r) => r,
        Err(_) => {
            info!("can not parse the relayer response:{:?}",response.result);
            return  Err("assign_task parse response error".to_string())
        },
    };
    info!("receice relayer response result is : {:?}", task_response); 

    let instance=Bytes::from(task_response.instance);
    let addr=Address::from_str(task_response.prover.as_str()).unwrap();
    let reward_token=Address::from_str(task_response.reward_token.as_str()).unwrap();
    let reward = U256::from(task_response.reward);
    let liability_window = u64::from(task_response.liability_window);
    let liability_token=Address::from_str(task_response.liability_token.as_str()).unwrap();
    let liability_amount = U256::from(task_response.liability);
    let expiry = u64::from(task_response.expiry);
    let signature=Bytes::from(task_response.signature);

    //send onchain transcations
    let _ = match submit_task(instance,addr,reward_token,reward,liability_window,liability_token,liability_amount,expiry,signature).await{
        Ok(r) => {
            info!("send submit_task success, tx hash is {:?}",r)
        }
        Err(e) => {
            error!("send submit_task error, reason:{:?}",e)
        },
    };
    Ok(())
}

pub async fn process_task_data(task:String){        //submit the task
    match assign_task(task.clone()).await{
        Ok(()) => (),
        Err(_) => {
            error!("assign the task:{} failed",task)
        }
    }
}


pub fn test() {
    println!("{}",Utc::now().timestamp());
    let dt = (Utc::now() + Duration::seconds(100)).timestamp();
    println!("today date + 137 days {}", dt);
}


  
  
