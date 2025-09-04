use axum::{extract::State, http::StatusCode, Json};
use tokio::sync::oneshot;

use crate::email_utils::send_magic_link_email;
use crate::jwt_utils::create_jwt;
use crate::types::types::{
    AppState, Claims, CreateUserMessage, GenericResponse, MagicLinkRequest, MagicLinkResponse,
    UserManagerMsg, VerifyMagicLinkRequest, VerifyMagicLinkResponse,
};

pub async fn send_magic_link_handler(
    State(state): State<AppState>,
    Json(payload): Json<MagicLinkRequest>,
) -> Result<(StatusCode, Json<MagicLinkResponse>), (StatusCode, Json<GenericResponse>)> {
    // For now, we'll create a user if they don't exist
    // In a real app, you might want to check if the email is registered first
    let user_id = format!("user_{}", nanoid::nanoid!(8));

    // Create JWT token for magic link (expires in 15 minutes)
    let magic_token = match create_magic_link_token(&payload.email, &user_id) {
        Ok(token) => token,
        Err(_) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(GenericResponse {
                    message: "Failed to generate magic link".to_string(),
                }),
            ));
        }
    };

    // Create magic link URL (adjust the URL for your frontend)
    let magic_link = format!("http://localhost:3000/auth/verify?token={}", magic_token);

    // Send email
    match send_magic_link_email(&payload.email, &magic_link).await {
        Ok(_) => {
            // Create user in the system
            let (tx, _rx) = oneshot::channel::<Result<String, String>>();
            if let Err(_) = state.user_tx.send(UserManagerMsg::Create(CreateUserMessage {
                username: payload.email.clone(),
                password: "".to_string(), // Magic link doesn't need password
                responder: tx,
            })) {
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(GenericResponse {
                        message: "Failed to create user account".to_string(),
                    }),
                ));
            }

            // We don't wait for the user creation response since email is already sent
            // In production, you might want to handle this differently

            Ok((
                StatusCode::OK,
                Json(MagicLinkResponse {
                    message: "Magic link sent to your email".to_string(),
                    email: payload.email,
                }),
            ))
        }
        Err(_) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(GenericResponse {
                message: "Failed to send email".to_string(),
            }),
        )),
    }
}

pub async fn verify_magic_link_handler(
    Json(payload): Json<VerifyMagicLinkRequest>,
) -> Result<(StatusCode, Json<VerifyMagicLinkResponse>), (StatusCode, Json<GenericResponse>)> {
    // Verify the magic link token
    match crate::jwt_utils::verify_jwt(&payload.token) {
        Ok(claims) => {
            // Create a new JWT token for the user session (24 hours)
            match create_jwt(&claims.sub) {
                Ok(session_token) => Ok((
                    StatusCode::OK,
                    Json(VerifyMagicLinkResponse {
                        token: session_token,
                        user_id: claims.sub.clone(),
                        email: claims.sub, // In this case, we're using user_id as email for simplicity
                    }),
                )),
                Err(_) => Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(GenericResponse {
                        message: "Failed to create session token".to_string(),
                    }),
                )),
            }
        }
        Err(_) => Err((
            StatusCode::UNAUTHORIZED,
            Json(GenericResponse {
                message: "Invalid or expired magic link".to_string(),
            }),
        )),
    }
}

fn create_magic_link_token(_email: &str, user_id: &str) -> Result<String, crate::jwt_utils::JwtError> {
    use chrono::{Duration, Utc};
    use jsonwebtoken::{encode, EncodingKey, Header};
    use crate::jwt_utils::get_jwt_secret;

    let expiration = Utc::now()
        .checked_add_signed(Duration::minutes(15)) // Magic links expire in 15 minutes
        .expect("valid timestamp")
        .timestamp() as usize;

    let claims = Claims {
        sub: user_id.to_string(),
        exp: expiration,
        iat: Utc::now().timestamp() as usize,
    };

    let header = Header::new(jsonwebtoken::Algorithm::HS256);
    let encoding_key = EncodingKey::from_secret(get_jwt_secret().as_bytes());

    encode(&header, &claims, &encoding_key).map_err(|_| crate::jwt_utils::JwtError::EncodingError)
}