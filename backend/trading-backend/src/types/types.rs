use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc::UnboundedSender, oneshot};

use crate::types::positions::Position;

//
// === Requests & Responses ===
//

#[derive(Deserialize)]
pub struct SignupRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct GenericResponse {
    pub message: String,
}

#[derive(Clone)]
pub struct AppState {
    pub user_tx: UnboundedSender<UserManagerMsg>,
    pub wallet_tx: UnboundedSender<WalletManagerMsg>,
    pub position_tx: UnboundedSender<PositionManagerMsg>,
}

//
// === Domain Models ===
//

#[derive(Deserialize, Clone)]
pub struct OpenOrderRequest {
    pub r#type: String, // "buy" | "sell"
    pub qty: Decimal,
    pub asset: String,
    pub stop_loss: Option<Decimal>,
    pub take_profit: Option<Decimal>,
    pub leverage: Option<Decimal>,
}

#[derive(Serialize, Clone)]
pub struct OpenOrderResponse {
    pub order_id: String,
}

//
// === Actor Messages ===
//

pub struct CreateUserMessage {
    pub username: String,
    pub password: String,
    pub responder: oneshot::Sender<Result<String, String>>, // returns user_id or error
}

pub enum UserManagerMsg {
    Create(CreateUserMessage),
    // Delete {
    //     username: String,
    //     responder: oneshot::Sender<Result<(), String>>,
    // },
    // Lookup {
    //     username: String,
    //     responder: oneshot::Sender<Option<User>>,
    // },
}

pub enum WalletManagerMsg {
    GetBalance {
        user_id: String,
        responder: oneshot::Sender<Option<Decimal>>,
    },
    Credit {
        user_id: String,
        amount: Decimal,
        responder: oneshot::Sender<Result<(), String>>,
    },
    Debit {
        user_id: String,
        amount: Decimal,
        responder: oneshot::Sender<Result<(), String>>,
    },
    Create {
        user_id: String,
        responder: oneshot::Sender<Result<(), String>>,
    },
}

// --- PositionManager messages ---
pub enum PositionManagerMsg {
    Open {
        user_id: String,
        order: OpenOrderRequest,
        responder: oneshot::Sender<Result<String, String>>,
    },
    Close {
        user_id: String,
        position_id: String,
        responder: oneshot::Sender<Result<(), String>>,
    },
    List {
        user_id: String,
        responder: oneshot::Sender<Option<Vec<Position>>>,
    },
}

#[derive(Deserialize)]
pub struct PriceUpdates {
    pub buy: Decimal,
    pub sell: Decimal,
    pub symbol: String,
}

#[derive(Debug)]
pub struct CurrentPrice {
    pub bid: Decimal,
    pub ask: Decimal,
}
