use crate::types::types::{AppState, GenericResponse, OpenOrderRequest};
use axum::{
    extract::{Json, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use rust_decimal_macros::dec;

pub async fn open_order_handler(
    State(_state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<OpenOrderRequest>,
) -> impl IntoResponse {
    let auth_header = match headers.get("Authorization") {
        Some(v) => v.to_str().unwrap_or(""),
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(GenericResponse {
                    success: false,
                    message: "".to_string(),
                    error: "Missing Authorization header".to_string(),
                }),
            );
        }
    };

    let parts: Vec<&str> = auth_header.split_whitespace().collect();
    if parts.len() != 2 || parts[0] != "Bearer" {
        return (
            StatusCode::BAD_REQUEST,
            Json(GenericResponse {
                success: false,
                message: "".to_string(),
                error: "Invalid Authorization header format".to_string(),
            }),
        );
    }
    let username = parts[1].to_string();

    if payload.r#type != "buy" && payload.r#type != "sell" {
        return (
            StatusCode::BAD_REQUEST,
            Json(GenericResponse {
                success: false,
                message: "".to_string(),
                error: "Invalid order type".to_string(),
            }),
        );
    }

    if payload.qty <= dec!(0.0) {
        return (
            StatusCode::BAD_REQUEST,
            Json(GenericResponse {
                success: false,
                message: "".to_string(),
                error: "Quantity must be > 0".to_string(),
            }),
        );
    }

    (
        StatusCode::OK,
        Json(GenericResponse {
            success: true,
            message: format!(
                "Order received: {} {} of {} (SL: {:?}, TP: {:?})",
                payload.r#type, payload.qty, payload.asset, payload.stop_loss, payload.take_profit
            ),
            error: "".to_string(),
        }),
    )
}
