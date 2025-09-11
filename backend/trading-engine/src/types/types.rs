use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use tokio::sync::oneshot;

use crate::types::positions::Position;

//
// === Domain Models ===
//

#[derive(Deserialize, Clone, Debug)]
pub struct OpenOrderRequest {
    pub order_id: String,
    pub user_id: String,
    pub qty: Decimal,
    pub asset: String,
    pub margin: Option<Decimal>,
    pub stop_loss: Option<Decimal>,
    pub take_profit: Option<Decimal>,
    pub leverage: Option<Decimal>,
}

#[derive(Deserialize, Clone)]
pub struct CloseOrderRequest {
    pub order_id: String,
}

#[derive(Serialize, Clone)]
pub struct OpenOrderResponse {
    pub order_id: String,
}

#[derive(Serialize, Clone)]
pub struct CloseOrderResponse {
    pub message: String,
}

#[derive(Serialize, Clone)]
pub struct GetListResponse {
    pub positions: Vec<Position>,
}

#[derive(Deserialize, Clone)]
pub struct IncomingPrices {
    pub btc: CurrentPrice,
    pub eth: CurrentPrice,
    pub sol: CurrentPrice,
}
pub struct SignUpRequest {
    pub email: String,
}

pub enum KafkaMessages {
    IncomingPrices(IncomingPrices),
    Order(OpenOrderRequest),
    CreateUser(SignUpRequest),
    InvalidMessage,
}

//
// === Actor Messages ===
//

pub struct CreateUserMessage {
    pub username: String,
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
    UpdateRisk,
}

#[derive(Deserialize)]
pub struct PriceUpdates {
    pub buy: Decimal,
    pub sell: Decimal,
    pub symbol: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CurrentPrice {
    pub bid: Decimal,
    pub ask: Decimal,
}
