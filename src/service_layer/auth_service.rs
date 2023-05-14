use crate::utilities::responses::{response_auth_ok, response_ok_auth_with_message, ApiResponse};
use argon2::{
    password_hash::{PasswordHash, PasswordVerifier},
    Argon2,
};
use axum::{
    async_trait,
    extract::State,
    extract::{FromRequestParts, TypedHeader},
    headers::{authorization::Bearer, Authorization},
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
    Json, RequestPartsExt,
};
use jsonwebtoken::errors::ErrorKind;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::configs::app_state::AppState;
use crate::constants::constants::{DEFAULT_HASH, TOKEN_LIFESPAN, TOKEN_REFRESH_LIFESPAN};
use crate::data_access_layer;

// JWT : https://github.com/Keats/jsonwebtoken#validation

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct UserLoginRequest {
    email: String,
    password: String,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct TokenRefreshRequest {
    refresh_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JwtClaims {
    pub user_uuid: String,
    pub private_user_uuid: String,
    exp: usize,
}

#[derive(Serialize)]
pub struct LoginResponse {
    token: String,
    refresh_token: String,
}

#[derive(Serialize)]
pub struct RefreshResponse {
    token: String,
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(login_user): Json<UserLoginRequest>,
) -> Result<(StatusCode, Json<ApiResponse<LoginResponse>>), AuthError> {
    let user_found = data_access_layer::user_dal::get_user_password_by_email(
        &state,
        login_user.email.to_string(),
    );

    match user_found {
        Ok((user_uuid, private_user_uuid, password)) => {
            let rehash = PasswordHash::new(&password).unwrap(); // Turning string into PHC string format type
            let valid_password =
                Argon2::default().verify_password(login_user.password.as_bytes(), &rehash);

            match valid_password {
                Ok(_) => {
                    let my_claims = JwtClaims {
                        user_uuid: user_uuid.clone(),
                        private_user_uuid: private_user_uuid.clone(),
                        exp: SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .expect("failed getting current timestamp")
                            .as_secs() as usize
                            + TOKEN_LIFESPAN,
                    };
                    let my_refresh_claims = JwtClaims {
                        user_uuid: user_uuid,
                        private_user_uuid: private_user_uuid,
                        exp: SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .expect("failed getting current timestamp")
                            .as_secs() as usize
                            + TOKEN_REFRESH_LIFESPAN,
                    };
                    let token = encode(
                        &Header::default(),
                        &my_claims,
                        &EncodingKey::from_secret(state.key_jwt.as_bytes()),
                    )
                    .map_err(|_| AuthError::TokenCreation)?;
                    let refresh_token = encode(
                        &Header::default(),
                        &my_refresh_claims,
                        &EncodingKey::from_secret(state.refresh_key_jwt.as_bytes()),
                    )
                    .map_err(|_| AuthError::TokenCreation)?;
                    response_ok_auth_with_message(
                        Some(LoginResponse {
                            token: token,
                            refresh_token: refresh_token,
                        }),
                        "Successfull login".to_string(),
                    )
                }
                Err(_) => Err(AuthError::WrongCredentials),
            }
        }
        Err(_) => {
            // If user doesn't exists, we still hash a constant default hash to dodge timing attacks
            let rehash = PasswordHash::new(DEFAULT_HASH).unwrap(); // Turning string into PHC string format type
            let _ = Argon2::default().verify_password("AYAYA_CUTE_PASSWORD".as_bytes(), &rehash);
            Err(AuthError::WrongCredentials)
        }
    }
}

pub async fn token_refresh(
    State(state): State<Arc<AppState>>,
    Json(refresh_request): Json<TokenRefreshRequest>,
) -> Result<(StatusCode, Json<ApiResponse<RefreshResponse>>), AuthError> {
    let validation = Validation {
        ..Validation::default()
    };
    let token_data = decode::<JwtClaims>(
        &refresh_request.refresh_token,
        &DecodingKey::from_secret(state.refresh_key_jwt.as_bytes()),
        &validation,
    );
    match token_data {
        Ok(data) => {
            let my_claims = JwtClaims {
                user_uuid: data.claims.user_uuid,
                private_user_uuid: data.claims.private_user_uuid,
                exp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("failed getting current timestamp")
                    .as_secs() as usize
                    + TOKEN_LIFESPAN,
            };
            let token = encode(
                &Header::default(),
                &my_claims,
                &EncodingKey::from_secret(state.key_jwt.as_bytes()),
            )
            .map_err(|_| AuthError::TokenCreation)?;
            response_auth_ok(Some(RefreshResponse { token: token }))
        }
        Err(e) => match *e.kind() {
            ErrorKind::InvalidToken => {
                // Example on how to handle a specific jwt error
                println!("Token is invalid");
                Err(AuthError::InvalidToken)
            }
            _ => Err(AuthError::InvalidToken),
        },
    }
}

#[derive(Debug)]
pub enum AuthError {
    WrongCredentials,
    TokenCreation,
    InvalidToken,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AuthError::WrongCredentials => (StatusCode::UNAUTHORIZED, "Wrong credentials"),
            AuthError::TokenCreation => (StatusCode::INTERNAL_SERVER_ERROR, "Token creation error"),
            AuthError::InvalidToken => (StatusCode::BAD_REQUEST, "Invalid token"),
        };
        let body = Json(json!({
            "error": error_message,
        }));
        (status, body).into_response()
    }
}

#[async_trait]
impl FromRequestParts<Arc<AppState>> for JwtClaims {
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| AuthError::InvalidToken)?;
        let token_data = decode::<JwtClaims>(
            bearer.token(),
            &DecodingKey::from_secret(state.key_jwt.as_bytes()),
            &Validation::default(),
        )
        .map_err(|_| AuthError::InvalidToken)?;

        match data_access_layer::user_dal::update_user_last_seen(
            state,
            token_data.claims.user_uuid.clone(),
        ) {
            Ok(_) => (),
            Err(e) => {
                println!("Error updating user last seen in authorization : {:?}", e);
            }
        }

        Ok(token_data.claims)
    }
}
