use crate::types::types::{
    AppState, GenericResponse, OpenOrderRequest, OpenOrderResponse, PositionManagerMsg,
};
use axum::{
    extract::{Json, State},
    http::{HeaderMap, StatusCode},
};
use tokio::sync::oneshot;

pub async fn open_order_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<OpenOrderRequest>,
) -> Result<(StatusCode, Json<OpenOrderResponse>), (StatusCode, Json<GenericResponse>)> {
    println!("Got a request");
    let auth_header = match headers.get("Authorization") {
        Some(v) => v.to_str().unwrap_or(""),
        None => {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(GenericResponse {
                    message: "Not authenticated".to_string(),
                }),
            ));
        }
    };

    let parts: Vec<&str> = auth_header.split_whitespace().collect();
    if parts.len() != 2 || parts[0] != "Bearer" {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(GenericResponse {
                message: "Authorization failed, include Bearer JWT token".to_string(),
            }),
        ));
    }

    let user_id = parts[1].to_string();

    if payload.asset != "BTC" {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(GenericResponse {
                message: format!(
                    "Invalid order, {} is not traded on MirrorX, place an order on a valid traded asset like BTC.",
                    payload.asset
                ),
            }),
        ));
    }

    // for now just trust the jwt (it ain't even jwt)
    //  TODO: We shall do the proper auth on Sat afternoon

    let (oneshot_tx, mut oneshot_rx) = oneshot::channel::<Result<String, String>>();

    if let Err(_) = state.position_tx.send(PositionManagerMsg::Open {
        user_id: user_id,
        order: payload.clone(),
        responder: oneshot_tx,
    }) {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(GenericResponse {
                message: "Order not processed, transaction failed!".to_string(),
            }),
        ));
    }

    let order_id = match oneshot_rx.await {
        Ok(Ok(position_id)) => position_id,
        Ok(Err(err)) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(GenericResponse { message: err }),
            ));
        }
        Err(err) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(GenericResponse {
                    message: err.to_string(),
                }),
            ));
        }
    };

    Ok((
        StatusCode::OK,
        Json(OpenOrderResponse { order_id: order_id }),
    ))
}
