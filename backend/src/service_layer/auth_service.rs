
use jsonwebtoken::{encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey};
use jsonwebtoken::errors::ErrorKind;

use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString
    },
    Argon2
};
use std::{time, time::{SystemTime, UNIX_EPOCH}, ops::Add};
use serde::{Deserialize, Serialize};
use actix_web::{middleware, web, App, HttpResponse, HttpServer, Result as actixResult};
use crate::my_errors::service_errors::ServiceError;
use crate::{AppState, data_access_layer};

const BEARER: &str = "Bearer ";
const KEY_JWT: &[u8] = b"badObviousTestKey";
const KEY_JWT_REFRESH: &str = "ohohoho";
const TOKEN_LIFESPAN: usize = 30;
const TOKEN_REFRESH_LIFESPAN: &str = "3600sec";
const DEFAULT_HASH: &str = "$argon2id$v=19$m=15000,t=2,p=1$SZZVht0nCXacXAJU1dYJ8w$QwpNt6gUQ2K+dHQVDTf5H1mkkA0yTkXXKwZ6vHkKClQ";

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct UserLoginRequest {
    pseudo: String,
    password: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: u32, // ID of the user
    exp: usize,
}

#[derive(Serialize)]
struct LoginResponse {
    jwt_token: String,
    message: String
}

pub async fn login(
    db: web::Data<AppState>,
    login_user: web::Json<UserLoginRequest>
) -> actixResult<HttpResponse, ServiceError>{
    let user_found = data_access_layer::user_dal::User::get_user(&db, login_user.pseudo.to_string()).await;

    match user_found {
        Ok(user) => {
            let rehash = PasswordHash::new(&user.password).unwrap(); // Turning string into PHC string format type
            let valid_password = Argon2::default().verify_password(login_user.password.as_bytes(), &rehash); //.expect("could not verify");

            match valid_password {
                Ok(_) => {
                    let my_claims = Claims{
                        sub: user.id,
                        exp: SystemTime::now().duration_since(UNIX_EPOCH).expect("failed getting current timestamp").as_secs() as usize + TOKEN_LIFESPAN
                    };
                    let token = encode(&Header::default(), &my_claims, &EncodingKey::from_secret(KEY_JWT.as_ref())).expect("failed token creation");               
                    print!("oui");
                    let validation = Validation { ..Validation::default() };
                    let token_data = match decode::<Claims>(&token, &DecodingKey::from_secret(KEY_JWT.as_ref()), &validation) {
                        Ok(c) => c,
                        Err(err) => match *err.kind() {
                            ErrorKind::InvalidToken => panic!("Token is invalid"), // Example on how to handle a specific error
                            ErrorKind::InvalidIssuer => panic!("Issuer is invalid"), // Example on how to handle a specific error
                            _ => panic!("Some other errors"),
                        },
                    };
                    println!("{:?}", token_data.claims);
                    println!("{:?}", token_data.header);
                    
                    Ok(HttpResponse::Ok().json(LoginResponse{jwt_token: token, message: "Successfull login".to_string()}))
                }
                Err(_) => Err(ServiceError::LoginError) // TODO use auth error perso,
            }
        },
        Err(_) => {
            // If user doesn't exists, we still hash a constant default hash to dodge timing attacks
            let rehash = PasswordHash::new(&DEFAULT_HASH).unwrap(); // Turning string into PHC string format type
            let _ = Argon2::default().verify_password("AYAYA_CUTE_PASSWORD".as_bytes(), &rehash);
            Err(ServiceError::LoginError)
        }
    }
}