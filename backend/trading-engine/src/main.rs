use std::sync::Arc;

use arc_swap::ArcSwap;
use rust_decimal_macros::dec;
use tokio::sync::mpsc;

use crate::types::{
    positions::Positions,
    types::{CurrentPrice, PositionManagerMsg, UserManagerMsg, WalletManagerMsg},
    users::Users,
    wallet::Wallets,
};

mod handlers;
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

    //  HACK:
    let wallet_tx_ = wallet_tx.clone();

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
                    match positions.update_risk(wallet_tx.clone()).await {
                        Ok(_) => {}
                        Err(x) => {
                            eprintln!("[UDPATE RISK PANIC]");
                        }
                    }
                }
            }
        }
    });

    tokio::signal::ctrl_c().await.unwrap();
    println!("Shutting down");
}
