use std::time::{SystemTime, UNIX_EPOCH};

use chrono::format::format;
use jsonrpc_http_server::jsonrpc_core::{IoHandler, Params, Value};
use jsonrpc_http_server::ServerBuilder;
use log::*;
use web3::signing::keccak256;

use crate::chain::{self, TASK_MSG_QUEUE, process_task_data};


pub async fn start_rpc_server(addr:String) -> jsonrpc_http_server::Server {
    let mut io = IoHandler::default();

    io.add_method("ReceiveTask", |params: Params| async {   //receive user side paramter and then cache/submit one proof task
        info!("receive ReceiveTask msg of {:?}",params.clone());
        let req_input: Vec<Value> = match params.parse(){
            Ok(r) => r,
            Err(_) => {
                return Ok(Value::String("parameter invalid".to_string()))
            },
        };
        if req_input.len() != 1 {
            return Ok(Value::String("parameter invalid".to_string()))
        }

        let random_number = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap().as_millis(); 

        let task=format!("{}{}{}",req_input[0].to_string(),"#".to_string(),random_number.to_string());
        let result = hex::encode(keccak256(&task.clone().into_bytes()));

        receive_task(task).await;  

        Ok(Value::String(result))
        
    });

    io.add_method("Test", |params: Params| async { //just for test
        let _: Vec<Value> = match params.parse(){
            Ok(r) => r,
            Err(_) => {
                return Ok(Value::String("parameter invalid".to_string()))
            },
        };
        chain::test();
        Ok(Value::String("success".to_string()))
    });

    info!("start the server on :{}",addr.clone());
 
    let server = ServerBuilder::new(io)
        .threads(2)
        .start_http(&addr.parse().unwrap())
        .unwrap();
    
    server
}

pub async fn receive_task(task:String){
    info!("receive one task data is {:?}",task);
    let mut queue = TASK_MSG_QUEUE.lock().await;
    queue.push_back(task);
}

pub async fn loop_task_data() -> web3::Result<()> {
    let mut queue = TASK_MSG_QUEUE.lock().await;
    while queue.len() > 0 {
        info!(" start to process the task data of len : {}",queue.len());
        let item = queue.pop_front().unwrap();
        process_task_data(item).await;
    }
    Ok(())
}

