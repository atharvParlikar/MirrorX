use crate::types::{
    positions::Position,
    types::{
        AppState, CloseOrderRequest, GenericResponse, GetListResponse, OpenOrderRequest,
        OpenOrderResponse, PositionManagerMsg,
    },
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

    let (oneshot_tx, oneshot_rx) = oneshot::channel::<Result<String, String>>();

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

pub async fn close_order_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CloseOrderRequest>,
) -> (StatusCode, Json<GenericResponse>) {
    let auth_header = match headers.get("Authorization") {
        Some(v) => v.to_str().unwrap_or(""),
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(GenericResponse {
                    message: "Not authenticated".to_string(),
                }),
            );
        }
    };

    let parts: Vec<&str> = auth_header.split_whitespace().collect();
    if parts.len() != 2 || parts[0] != "Bearer" {
        return (
            StatusCode::BAD_REQUEST,
            Json(GenericResponse {
                message: "Authorization failed, include Bearer JWT token".to_string(),
            }),
        );
    }

    let user_id = parts[1].to_string();

    println!("user_id: {}", user_id);

    let (oneshot_tx, oneshot_rx) = oneshot::channel::<Result<(), String>>();

    if let Err(_) = state.position_tx.send(PositionManagerMsg::Close {
        user_id,
        position_id: payload.order_id.clone(),
        responder: oneshot_tx,
    }) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(GenericResponse {
                message: "Failed to send close order request".to_string(),
            }),
        );
    }

    match oneshot_rx.await {
        Ok(Ok(())) => (
            StatusCode::OK,
            Json(GenericResponse {
                message: format!("Position {} closed", payload.order_id),
            }),
        ),
        Ok(Err(err)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(GenericResponse { message: err }),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(GenericResponse {
                message: "Close order channel dropped".to_string(),
            }),
        ),
    }
}

pub async fn order_list_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<(StatusCode, Json<GetListResponse>), (StatusCode, Json<GenericResponse>)> {
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

    let (oneshot_tx, oneshot_rx) = oneshot::channel::<Option<Vec<Position>>>();

    match state.position_tx.send(PositionManagerMsg::List {
        user_id: user_id,
        responder: oneshot_tx,
    }) {
        Ok(_) => {}
        Err(_) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(GenericResponse {
                    message: "something went wrong".to_string(),
                }),
            ));
        }
    };

    match oneshot_rx.await {
        Ok(Some(position_list)) => {
            return Ok((
                StatusCode::OK,
                Json(GetListResponse {
                    positions: position_list,
                }),
            ));
        }
        Ok(None) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(GenericResponse {
                    message: "could not find user".to_string(),
                }),
            ))
        }
        Err(err) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(GenericResponse {
                    message: "Something went wrong".to_string(),
                }),
            ));
        }
    };
}
