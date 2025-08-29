use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use rust_decimal::Decimal;
use tokio::sync::oneshot;

use crate::types::types::{AppState, GenericResponse, WalletManagerMsg};

pub async fn balance_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let auth_header = match headers.get("Authorization") {
        Some(v) => v.to_str().unwrap_or(""),
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(GenericResponse {
                    message: "".to_string(),
                }),
            );
        }
    };

    let parts: Vec<&str> = auth_header.split_whitespace().collect();
    if parts.len() != 2 || parts[0] != "Bearer" {
        return (
            StatusCode::BAD_REQUEST,
            Json(GenericResponse {
                message: "".to_string(),
            }),
        );
    }
    let username = parts[1].to_string();

    let (tx, rx) = oneshot::channel::<Option<Decimal>>();

    if let Err(_) = state.wallet_tx.send(WalletManagerMsg::GetBalance {
        user_id: username,
        responder: tx,
    }) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(GenericResponse {
                message: "".to_string(),
            }),
        );
    }

    // 4. Await result
    match rx.await {
        Ok(Some(balance_json)) => (
            StatusCode::OK,
            Json(GenericResponse {
                message: balance_json.to_string(),
            }),
        ),
        Ok(None) => (
            StatusCode::BAD_REQUEST,
            Json(GenericResponse {
                message: "".to_string(),
            }),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(GenericResponse {
                message: "".to_string(),
            }),
        ),
    }
}
