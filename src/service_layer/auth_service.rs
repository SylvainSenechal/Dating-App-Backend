use actix_web::{
    dev, error::ErrorUnauthorized, web, Error, FromRequest, HttpRequest, HttpResponse,
    Result as actixResult,
};
use argon2::{
    password_hash::{PasswordHash, PasswordVerifier},
    Argon2,
};
use futures::future::{err, ok, Ready};
use jsonwebtoken::errors::ErrorKind;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::constants::constants::{
    BEARER, DEFAULT_HASH, KEY_JWT, KEY_JWT_REFRESH, TOKEN_LIFESPAN, TOKEN_REFRESH_LIFESPAN,
};
use crate::my_errors::service_errors::ServiceError;
use crate::{data_access_layer, AppState};

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
struct Claims {
    sub: usize, // ID of the user
    exp: usize,
}

#[derive(Serialize)]
struct LoginResponse {
    token: String,
    refresh_token: String,
    message: String,
}

#[derive(Serialize)]
struct RefreshResponse {
    token: String,
}

pub async fn login(
    db: web::Data<AppState>,
    login_user: web::Json<UserLoginRequest>,
) -> actixResult<HttpResponse, ServiceError> {
    let user_found = data_access_layer::user_dal::User::get_user_password_by_email(
        &db,
        login_user.email.to_string(),
    );

    match user_found {
        Ok((user_id, password)) => {
            let rehash = PasswordHash::new(&password).unwrap(); // Turning string into PHC string format type
            let valid_password =
                Argon2::default().verify_password(login_user.password.as_bytes(), &rehash);

            match valid_password {
                Ok(_) => {
                    let my_claims = Claims {
                        sub: user_id,
                        exp: SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .expect("failed getting current timestamp")
                            .as_secs() as usize
                            + TOKEN_LIFESPAN,
                    };
                    let my_refresh_claims = Claims {
                        sub: user_id,
                        exp: SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .expect("failed getting current timestamp")
                            .as_secs() as usize
                            + TOKEN_REFRESH_LIFESPAN,
                    };
                    let token = encode(
                        &Header::default(),
                        &my_claims,
                        &EncodingKey::from_secret(KEY_JWT),
                    )
                    .expect("failed token creation");
                    let refresh_token = encode(
                        &Header::default(),
                        &my_refresh_claims,
                        &EncodingKey::from_secret(KEY_JWT_REFRESH),
                    )
                    .expect("failed token creation");

                    Ok(HttpResponse::Ok().json(LoginResponse {
                        token: token,
                        refresh_token: refresh_token,
                        message: "Successfull login".to_string(),
                    }))
                }
                Err(_) => Err(ServiceError::LoginError),
            }
        }
        Err(_) => {
            // If user doesn't exists, we still hash a constant default hash to dodge timing attacks
            let rehash = PasswordHash::new(&DEFAULT_HASH).unwrap(); // Turning string into PHC string format type
            let _ = Argon2::default().verify_password("AYAYA_CUTE_PASSWORD".as_bytes(), &rehash);
            Err(ServiceError::LoginError)
        }
    }
}

pub async fn token_refresh(
    refresh_request: web::Json<TokenRefreshRequest>,
) -> actixResult<HttpResponse, ServiceError> {
    let validation = Validation {
        ..Validation::default()
    };
    let token_data = decode::<Claims>(
        &refresh_request.refresh_token,
        &DecodingKey::from_secret(KEY_JWT_REFRESH),
        &validation,
    );
    match token_data {
        Ok(data) => {
            println!("{:?}", data);
            let my_claims = Claims {
                sub: data.claims.sub,
                exp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("failed getting current timestamp")
                    .as_secs() as usize
                    + TOKEN_LIFESPAN,
            };
            let token = encode(
                &Header::default(),
                &my_claims,
                &EncodingKey::from_secret(KEY_JWT),
            )
            .expect("failed token creation");
            Ok(HttpResponse::Ok().json(RefreshResponse { token: token }))
        }
        Err(e) => match *e.kind() {
            ErrorKind::InvalidToken => {
                // Example on how to handle a specific jwt error
                println!("Token is invalid");
                return Err(ServiceError::JwtError);
            }
            _ => return Err(ServiceError::JwtError),
        },
    }
}

pub struct AuthorizationUser {
    pub id: usize,
}

impl FromRequest for AuthorizationUser {
    type Error = Error;
    type Future = Ready<Result<AuthorizationUser, Error>>;
    type Config = ();

    fn from_request(
        // db: web::Data<AppState>,
        req: &HttpRequest,
        _payload: &mut dev::Payload,
    ) -> Self::Future {
        let _auth = req.headers().get("Authorization");
        match _auth {
            Some(val) => {
                if let Ok(header) = val.to_str() {
                    println!("header {}", header);

                    if header.starts_with(BEARER) {
                        let token = header[6..header.len()].trim();
                        println!("token {}", token);
                    }
                }
                let _split: Vec<&str> = _auth.unwrap().to_str().unwrap().split("Bearer").collect();
                let token = _split[1].trim();
                let validation = Validation {
                    ..Validation::default()
                };
                let token_data =
                    decode::<Claims>(&token, &DecodingKey::from_secret(KEY_JWT), &validation);

                match token_data {
                    Ok(data) => {
                        println!("Auth accepted for {}", data.claims.sub);
                        // Anytime a user perform a protected action, we update it's lastseen field to now
                        if let Some(db) = req.app_data::<web::Data<AppState>>() {
                            match data_access_layer::user_dal::User::update_user_last_seen(
                                &db,
                                data.claims.sub,
                            ) {
                                Ok(_) => (),
                                Err(e) => {
                                    println!(
                                        "Error updating user last seen in authorization : {:?}",
                                        e
                                    );
                                }
                            }
                        }

                        return ok(AuthorizationUser {
                            id: data.claims.sub,
                        });
                    }
                    Err(e) => {
                        println!("PAS ok auth {}", e);
                        return err(ErrorUnauthorized("invalid token!"));
                    }
                }
            }
            None => {
                println!("No Authorization header found");
                err(ErrorUnauthorized("blocked!"))
            }
        }
    }
}

pub fn validate_token(token: &str) -> Option<AuthorizationUser> {
    let validation = Validation {
        ..Validation::default()
    };
    let token_data = decode::<Claims>(&token, &DecodingKey::from_secret(KEY_JWT), &validation);
    match token_data {
        Ok(data) => {
            println!("ok validate_token");
            return Some(AuthorizationUser {
                id: data.claims.sub,
            });
        }
        Err(e) => {
            println!("validate_token not ok : {}", e);
            return None;
        }
    }
}
