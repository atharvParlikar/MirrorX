use std::sync::Arc;

use arc_swap::ArcSwap;
use axum::{
    routing::{get, post},
    Router,
};
use redis::Client;
use rust_decimal_macros::dec;
use tokio::sync::mpsc;

use crate::{
    handlers::{balance::balance_handler, order::open_order_handler, signup::signup_handler},
    types::{
        positions::Positions,
        types::{
            AppState, CurrentPrice, PositionManagerMsg, PriceUpdates, UserManagerMsg,
            WalletManagerMsg,
        },
        users::Users,
        wallet::Wallets,
    },
};

mod handlers;
mod types;

#[tokio::main]
async fn main() {
    let redis_client = Client::open("redis://127.0.0.1/").unwrap();
    let mut con = redis_client.get_connection().unwrap();

    let mut latestPrice = Arc::new(ArcSwap::from(Arc::new(CurrentPrice {
        bid: dec!(0),
        ask: dec!(0),
    })));

    let users: Users = Users::new();
    let wallets: Wallets = Wallets::new();
    let mut positions: Positions = Positions::new(latestPrice.clone());

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

    tokio::task::spawn_blocking(move || {
        let mut pubsub = con.as_pubsub();

        loop {
            let msg = pubsub.get_message().unwrap();
            let payload: String = msg.get_payload().unwrap();

            let prices: PriceUpdates = serde_json::from_str(payload.as_str()).unwrap();

            let newPrices = Arc::new(CurrentPrice {
                bid: prices.buy,
                ask: prices.sell,
            });

            latestPrice.store(newPrices);
        }
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
                        if let Err(_) = responder.send(Ok(())) {
                            eprintln!("[ERROR] responder connection closed");
                        }
                    }
                    None => {
                        if let Err(_) = responder.send(Err("Could not create wallet".to_string())) {
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
                    match positions.open(user_id, order, wallet_tx.clone()).await {
                        Ok(position_id) => responder.send(Ok(position_id)).unwrap(),
                        Err(err) => responder.send(Err(err)).unwrap(),
                    };
                }
                PositionManagerMsg::Close {
                    user_id,
                    position_id,
                    responder,
                } => {
                    match positions
                        .close(&user_id, position_id, wallet_tx.clone())
                        .await
                    {
                        Ok(_) => responder.send(Ok(())).unwrap(),
                        Err(err) => responder.send(Err(err)).unwrap(),
                    };
                }
                PositionManagerMsg::List { user_id, responder } => {
                    match positions.list(&user_id) {
                        Ok(positions_list) => responder.send(Some(positions_list)).unwrap(),
                        Err(_) => responder.send(None).unwrap(),
                    };
                }
            }
        }
    });

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
