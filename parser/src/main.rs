use std::{thread::{sleep, self}, time::Duration, collections::HashMap};

pub mod pusher;

use blockchain::ChainProvider;
use primitives::{chain::Chain, Transaction};
use settings::Settings;
use storage::DatabaseClient;

use crate::pusher::Pusher;

#[tokio::main]
pub async fn main() {
    println!("Hello, parser!");

    let settings: Settings = Settings::new().unwrap();
    let mut database_client: DatabaseClient = DatabaseClient::new(&settings.postgres.url);
    let bnbchain_client = blockchain::BNBChainClient::new(
        settings.chains.binance.url,
        settings.chains.binance.api
    );
    let mut pusher = Pusher::new(
        settings.pusher.url,
        settings.postgres.url,
        settings.pusher.ios.topic,
    );

    // let providers: Vec<Box<dyn ChainProvider>> = vec![
    //     Box::new(blockchain::BNBChainClient::new(
    //         settings.chains.binance.url,
    //         settings.chains.binance.api
    //     )),
    //     Box::new(blockchain::BNBChainClient::new(
    //         settings.chains.binance.url,
    //         settings.chains.binance.api
    //     )),
    // ];

    // for provider in providers {
    //     tokio::spawn(async move {

    //         println!("launch provider: {:?}", provider.get_chain());

    //         loop {
    //             let latest_block: i32 = provider.get_latest_block().await.unwrap();
    //             println!("latest_block: {:?}", latest_block);

    //             //thread::sleep(Duration::from_secs(2))
    //         }
    //     });
    // }

    loop {
        let chain = Chain::Binance;
        let state = database_client.get_parser_state(chain).unwrap();

        let latest_block = bnbchain_client.get_latest_block().await;
        match latest_block {
            Ok(latest_block) => {
                let _ = database_client.set_parser_state_latest_block(chain, latest_block);
                if state.current_block + state.await_blocks >= state.latest_block {
                    
                    println!("parser ahead. current_block: {}, latest_block: {}, await_blocks: {}", state.current_block, state.latest_block, state.await_blocks);
        
                    thread::sleep(Duration::from_secs(settings.pusher.timeout)); continue;
                }
             },
            Err(err) => {
                println!("latest_block error: {:?}", err);

                sleep(Duration::from_secs(settings.pusher.timeout)); continue;
            }
        }
        
        println!("current_block: {}, latest_block: {}", state.current_block, state.latest_block);
 
        let mut next_block = state.current_block + 1;
        
        loop {
            println!("next_block: {:?}, to go: {}", next_block, state.latest_block - next_block);

            let transactions = bnbchain_client.get_transactions(next_block).await;
            match transactions {
                Ok(transactions) => {
                    let _ = database_client.set_parser_state_current_block(chain, next_block);
                    let addresses = transactions.clone().into_iter().map(|x| x.addresses() ).flatten().collect();
                    let subscriptions = database_client.get_subscriptions(chain, addresses).unwrap();
                    let mut transactions_map: HashMap<String, Transaction> = HashMap::new();

                    for subscription in subscriptions {
                        for transaction in transactions.clone() {
                            if transaction.addresses().contains(&subscription.address) {
                                let device = database_client.get_device_by_id(subscription.device_id).unwrap();
                                println!("Push: device: {}, transaction: {:?}", subscription.device_id, transaction.hash);
                                
                                transactions_map.insert(transaction.clone().id, transaction.clone());

                                let result = pusher.push(device.as_primitive(), transaction.clone()).await;
                                match result {
                                    Ok(result) => { println!("Push: result: {:?}", result); },
                                    Err(err) => { println!("Push: error: {:?}", err); }
                                }
                            }
                        }
                    }

                    let transactions: Vec<storage::models::Transaction> = transactions_map
                        .into_iter()
                        .map(|x| x.1)
                        .collect::<Vec<Transaction>>()
                        .into_iter().map(|x| {
                            return storage::models::Transaction::from_primitive(x);
                        }).collect();

                    database_client.add_transactions(transactions).unwrap();
                },
                Err(err) => {
                    println!("get transactions error: {:?}", err);
                }
            }

            if next_block >= state.latest_block || next_block % 100 == 0  {
                break
            }

            next_block += 1;
        }
    }
}