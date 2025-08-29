use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use tokio::sync::oneshot;

use crate::types::types::{
    AppState, CreateUserMessage, GenericResponse, SignupRequest, UserManagerMsg,
};

pub async fn signup_handler(
    State(reciever): State<AppState>,
    Json(payload): Json<SignupRequest>,
) -> impl IntoResponse {
    let (oneshot_tx, mut oneshot_rx) = oneshot::channel::<Result<String, String>>();

    let sent = reciever
        .user_tx
        .send(UserManagerMsg::Create(CreateUserMessage {
            username: payload.username,
            password: payload.password,
            responder: oneshot_tx,
        }));

    if let Err(_) = sent {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(GenericResponse {
                message: "".to_string(),
            }),
        );
    }

    match oneshot_rx.await {
        Ok(Ok(msg)) => {
            return (StatusCode::CREATED, Json(GenericResponse { message: msg }));
        }
        Ok(Err(err)) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(GenericResponse { message: err }),
            );
        }
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(GenericResponse {
                    message: "".to_string(),
                }),
            );
        }
    }
}
