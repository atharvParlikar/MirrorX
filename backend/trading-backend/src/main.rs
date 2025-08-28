use axum::{
    routing::{get, post},
    Router,
};
use std::collections::HashMap;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::{
    handlers::{balance::balance_handler, order::open_order_handler, signup::signup_handler},
    types::{
        types::{AppState, Position, PositionManagerMsg, UserManagerMsg, WalletManagerMsg},
        users::Users,
        wallet::Wallets,
    },
};

mod handlers;
mod types;

#[tokio::main]
async fn main() {
    let users: Users = Users::new();
    let wallets: Wallets = Wallets::new();
    let position_map: HashMap<String, Vec<Position>> = HashMap::new();

    let (user_tx, mut user_rx) = mpsc::unbounded_channel::<UserManagerMsg>();
    let (wallet_tx, mut wallet_rx) = mpsc::unbounded_channel::<WalletManagerMsg>();
    let (position_tx, mut position_rx) = mpsc::unbounded_channel::<PositionManagerMsg>();

    let app: Router = Router::new()
        .route("/signup", post(signup_handler))
        .route("/balance", get(balance_handler))
        .route("/order/open", post(open_order_handler))
        .with_state(AppState {
            user_tx: user_tx.clone(),
            wallet_tx: wallet_tx.clone(),
            position_tx: position_tx.clone(),
        });

    // Manages users
    tokio::spawn(async move {
        while let Some(msg) = user_rx.recv().await {
            match msg {
                UserManagerMsg::Create(create_msg) => {}
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
                        wallets
                            .update_balance(user_id, current_balance + amount)
                            .unwrap();
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
                        wallets
                            .update_balance(user_id, current_balance - amount)
                            .unwrap();
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
                    match wallets.get_balance(&user_id) {
                        Some(balance) => responder.send(Some(balance)),
                        None => responder.send(None),
                    }
                    .unwrap();
                }
                WalletManagerMsg::Create { user_id, responder } => match wallets.create(user_id) {
                    Some(_) => {
                        responder.send(Ok(()));
                    }
                    None => {
                        responder.send(Err("Could not create wallet".to_string()));
                    }
                },
            }
        }
    });

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
