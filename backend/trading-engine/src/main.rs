use std::sync::Arc;

use arc_swap::ArcSwap;
use rust_decimal_macros::dec;
use tokio::sync::{mpsc, oneshot};

use futures::StreamExt;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::{ClientConfig, Message};

use crate::kafka::handle_kafka_message;
use crate::types::types::KafkaMessages;
use crate::types::{
    positions::Positions,
    types::{CurrentPrice, PositionManagerMsg, UserManagerMsg, WalletManagerMsg},
    users::Users,
    wallet::Wallets,
};

mod kafka;
mod types;

#[tokio::main]
async fn main() {
    let latest_price = Arc::new(ArcSwap::from(Arc::new(CurrentPrice {
        bid: dec!(0),
        ask: dec!(0),
    })));

    let mut users: Users = Users::new();
    let wallets: Wallets = Wallets::new();
    let mut positions: Positions = Positions::new(latest_price.clone());

    let (user_tx, mut user_rx) = mpsc::unbounded_channel::<UserManagerMsg>();
    let (wallet_tx, mut wallet_rx) = mpsc::unbounded_channel::<WalletManagerMsg>();
    let (position_tx, mut position_rx) = mpsc::unbounded_channel::<PositionManagerMsg>();

    let consumer: StreamConsumer = ClientConfig::new()
        .set("bootstrap.servers", "localhost:9092")
        .set("group.id", "rust-analyzer")
        .set("auto.offset.reset", "earliest")
        .create()
        .expect("Consumer creation failed");

    //  HACK:
    let wallet_tx_ = wallet_tx.clone();

    tokio::spawn(async move {
        consumer
            .subscribe(&["priceUpdate"])
            .expect("Can't subscribe");

        println!("Consumer started");

        let mut stream = consumer.stream();
        while let Some(message) = stream.next().await {
            match message {
                Ok(m) => {
                    let payload = match m.payload_view::<str>() {
                        Some(Ok(s)) => s,
                        _ => "",
                    };
                    let key = m.key().map(|k| String::from_utf8_lossy(k).to_string());

                    if let Some(key) = key {
                        let parsed_message = handle_kafka_message(key.as_str(), payload);

                        match parsed_message {
                            KafkaMessages::IncomingPrices(price) => {}
                            KafkaMessages::Order(order) => {
                                println!("{:?}", order);
                                let (oneshot_tx, oneshot_rx) =
                                    oneshot::channel::<Result<String, String>>();

                                let sent = position_tx.send(PositionManagerMsg::Open {
                                    user_id: order.user_id.clone(),
                                    order: order,
                                    responder: oneshot_tx,
                                });

                                if let Err(err) = sent {
                                    eprintln!("[KAFKA CONSUMER ORDER] {}", err);
                                }

                                match oneshot_rx.await {
                                    Ok(Ok(response)) => {
                                        println!("{}", response);
                                    }
                                    Ok(Err(err)) => {
                                        println!("{}", err);
                                    }
                                    Err(err) => {
                                        eprintln!("{}", err);
                                    }
                                }
                            }
                            KafkaMessages::CreateUser(signup_req) => {
                                signup_req.email;
                            }
                            KafkaMessages::InvalidMessage => {}
                        }
                    }
                }
                Err(e) => eprintln!("Kafka error: {}", e),
            }
        }
    });

    // Manages users
    tokio::spawn(async move {
        while let Some(msg) = user_rx.recv().await {
            match msg {
                UserManagerMsg::Create(create_msg) => {
                    let sent = match users
                        .create_user(create_msg.username, wallet_tx_.clone())
                        .await
                    {
                        Ok(user_id) => create_msg.responder.send(Ok(user_id)),
                        Err(err) => create_msg.responder.send(Err(err)),
                    };

                    if let Err(_) = sent {
                        eprintln!("[error responding to create user message]");
                    }
                }
            };
        }
    });

    // Manages wallet
    tokio::spawn(async move {
        let mut wallets = wallets;
        while let Some(msg) = wallet_rx.recv().await {
            match msg {
                WalletManagerMsg::Credit {
                    user_id,
                    amount,
                    responder,
                } => match wallets.get_balance(&user_id) {
                    Some(current_balance) => {
                        if let Err(err) = wallets.update_balance(user_id, current_balance + amount)
                        {
                            eprintln!("{}", err);
                        }
                        if let Err(_) = responder.send(Ok(())) {
                            eprintln!("[ERROR] wallet oneshot channel closed");
                        }
                    }
                    None => {
                        if let Err(_) = responder.send(Err("Could not find wallet".to_string())) {
                            eprintln!("[ERROR] wallet oneshot channel closed");
                        }
                    }
                },
                WalletManagerMsg::Debit {
                    user_id,
                    amount,
                    responder,
                } => match wallets.get_balance(&user_id) {
                    Some(current_balance) => {
                        if let Err(err) = wallets.update_balance(user_id, current_balance - amount)
                        {
                            eprintln!("{}", err);
                        }
                        if let Err(_) = responder.send(Ok(())) {
                            eprintln!("[ERROR] wallet oneshot channel closed");
                        }
                    }
                    None => {
                        if let Err(_) = responder.send(Err("Could not find wallet".to_string())) {
                            eprintln!("[ERROR] wallet oneshot channel closed");
                        }
                    }
                },
                WalletManagerMsg::GetBalance { user_id, responder } => {
                    let sent = match wallets.get_balance(&user_id) {
                        Some(balance) => responder.send(Some(balance)),
                        None => responder.send(None),
                    };

                    if let Err(_) = sent {
                        println!("[ERROR RESPONDING BACK TO GET BALANCE]");
                    }
                }
                WalletManagerMsg::Create { user_id, responder } => match wallets.create(user_id) {
                    Ok(_) => {
                        if let Err(_) = responder.send(Ok(())) {
                            eprintln!("[ERROR] responder connection closed");
                        }
                    }
                    Err(err) => {
                        if let Err(_) = responder.send(Err(err)) {
                            eprintln!("[ERROR] responder connection closed");
                        }
                    }
                },
            }
        }
    });

    // Position manager thread
    tokio::spawn(async move {
        while let Some(msg) = position_rx.recv().await {
            match msg {
                PositionManagerMsg::Open {
                    user_id,
                    order,
                    responder,
                } => {
                    let sent = match positions.open(user_id, order, wallet_tx.clone()).await {
                        Ok(position_id) => responder.send(Ok(position_id)),
                        Err(err) => responder.send(Err(err)),
                    };

                    if let Err(_) = sent {
                        eprintln!("[ERROR RESPONDING TO POSITION OPEN MSG]");
                    }
                }
                PositionManagerMsg::Close {
                    user_id,
                    position_id,
                    responder,
                } => {
                    let sent = match positions
                        .close(&user_id, position_id, wallet_tx.clone())
                        .await
                    {
                        Ok(_) => responder.send(Ok(())),
                        Err(err) => responder.send(Err(err)),
                    };

                    if let Err(_) = sent {
                        eprintln!("[ERROR RESPONDING TO POSITION CLOSE MSG]");
                    }
                }
                PositionManagerMsg::List { user_id, responder } => {
                    let sent = match positions.list(&user_id) {
                        Ok(positions_list) => responder.send(Some(positions_list)),
                        Err(_) => responder.send(None),
                    };
                    if let Err(_) = sent {
                        eprintln!("[ERROR RESPONDING TO POSITION LIST MSG]")
                    }
                }
                PositionManagerMsg::UpdateRisk => {
                    // match positions.update_risk(wallet_tx.clone()).await {
                    //     Ok(_) => {}
                    //     Err(_) => {
                    //         eprintln!("[UDPATE RISK PANIC]");
                    //     }
                    // }
                    // let prices = latest_price_.clone().load();
                    // println!("{} | {}", prices.bid, prices.ask);
                }
            }
        }
    });

    tokio::signal::ctrl_c().await.unwrap();
    println!("Shutting down");
}
