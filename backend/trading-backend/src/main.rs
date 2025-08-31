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
    handlers::{
        balance::balance_handler,
        order::{close_order_handler, open_order_handler, order_list_handler},
        signup::signup_handler,
    },
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

    let app: Router = Router::new()
        .route("/signup", post(signup_handler))
        .route("/balance", get(balance_handler))
        .route("/order/open", post(open_order_handler))
        .route("/order/close", post(close_order_handler))
        .route("/order/list", get(order_list_handler))
        .with_state(AppState {
            user_tx: user_tx.clone(),
            wallet_tx: wallet_tx.clone(),
            position_tx: position_tx.clone(),
        });

    tokio::task::spawn_blocking(move || {
        let mut pubsub = con.as_pubsub();

        pubsub.subscribe("priceUpdates").unwrap();

        loop {
            let msg = pubsub.get_message().unwrap();
            let payload: String = msg.get_payload().unwrap();

            let prices: PriceUpdates = serde_json::from_str(payload.as_str()).unwrap();

            let new_prices = Arc::new(CurrentPrice {
                bid: prices.buy,
                ask: prices.sell,
            });

            latest_price.store(new_prices.clone());

            if let Err(err) = position_tx.send(PositionManagerMsg::UpdateRisk) {
                eprintln!("[ERRORT UPDATRE RISK MESSAGE] {}", err);
            }
        }
    });

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

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
