use std::{thread, time};

use chain::dummy_task;
use log::*;
use clap::{load_yaml, App};
use server::loop_task_data;
use web3::signing::keccak256;

mod server;
mod chain;
use crate::{server::start_rpc_server, chain::{PRIV_KEY, RELAYER_URL, CONTRACT}};

#[macro_use]
mod app_marco;

pub async fn process_task_data() {
    loop{
        thread::sleep(time::Duration::from_secs(5));
        debug!("start to process task data");
        match loop_task_data().await{
            Ok(()) => (),
            Err(_) => {
                error!("***process one proof task error***");
            }
        }
    }
}

pub async fn dummy_task_loop(interval:u64) { //dummy onchain task in interval seconds period
    loop{
        thread::sleep(time::Duration::from_secs(interval));
        debug!("start to send dummy task");
        let mut retry:usize = 0;
        loop{
            retry += 1;
            match dummy_task().await{
                Ok(_) => break,
                Err(_) => {
                    if retry <= 1{
                        continue;
                    }else {
                        break;
                    }
                },
            };
        } 
    }
}

#[tokio::main]
async fn main() {

    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
    .filter(Some("chain"), log::LevelFilter::Error)
    .init();

    let cli_param_yml = load_yaml!("app.yml");
    let cli_param = App::from_yaml(cli_param_yml).get_matches();
    let key: String = cli_param.value_of("key").unwrap_or("").into();
    let listen: String = cli_param.value_of("listen").unwrap_or("").into();
    let relayer: String = cli_param.value_of("relayer").unwrap_or("").into();
    let interval: String = cli_param.value_of("interval").unwrap_or("").into();
    let contract_addr: String = cli_param.value_of("contract").unwrap_or("").into();
    
    {
        let mut priv_key = PRIV_KEY.lock().await;
        *priv_key=key.clone();
    
        let mut relayer_url = RELAYER_URL.lock().await;
        *relayer_url=relayer.clone();

        let mut contract = CONTRACT.lock().await;
        *contract=contract_addr.clone();

    }
    
    let myserver = start_rpc_server(listen);

    let srv_handle = tokio::spawn(async move {
        myserver.await.wait();
    });

    let process_task_handle = tokio::spawn(async move {
        process_task_data().await
    });

    let dummy_task_handle = tokio::spawn(async move {
        dummy_task_loop(interval.parse::<u64>().unwrap()).await
    });

    tokio::select! {
      _ = async { srv_handle.await } => {
        info!("server terminal")
        },
      _ = async { process_task_handle.await } => {
        info!("process task handle terminal")
        },
      _ = async { dummy_task_handle.await } => {
        info!("dummy task handle terminal")
       },
    }

}
