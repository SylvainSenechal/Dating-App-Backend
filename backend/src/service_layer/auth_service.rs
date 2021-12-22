
use jsonwebtoken::{encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey};
use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString
    },
    Argon2
};
use serde::{Deserialize, Serialize};
use actix_web::{middleware, web, App, HttpResponse, HttpServer, Result as actixResult};
use crate::my_errors::service_errors::ServiceError;
use crate::{AppState, data_access_layer};

const BEARER: &str = "Bearer ";
const KEY_JWT: &[u8] = b"badObviousTestKey";
const KEY_JWT_REFRESH: &str = "ohohoho";
const TOKEN_LIFESPAN: &str = "30sec";
const TOKEN_REFRESH_LIFESPAN: &str = "3600sec";

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct UserLoginRequest {
    pseudo: String,
    password: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    company: String,
    exp: usize,
}

// todo revoir le guide pour pas asvoir de faille sur les attaques par timing 
pub async fn login(
    db: web::Data<AppState>,
    login_user: web::Json<UserLoginRequest>
) -> actixResult<HttpResponse, ServiceError>{
    let user_found = data_access_layer::user_dal::User::get_user(&db, login_user.pseudo.to_string()).await;

    match user_found {
        Ok(user) => {
            let rehash = PasswordHash::new(&user.password).unwrap();
            let pass = Argon2::default().verify_password(login_user.password.as_bytes(), &rehash); //.expect("could not verify");
            match pass {
                Ok(_) => {
                    let my_claims = Claims{
                        sub: "hehe".to_string(),
                        company: "hoho".to_string(),
                        exp: 44
                    };
                    let token = encode(&Header::default(), &my_claims, &EncodingKey::from_secret(KEY_JWT.as_ref())).expect("failed token creation");
                    
                    // Ok(HttpResponse::Ok().body("User created"))
                    Ok(HttpResponse::Ok().json(token))
                    
                }
                Err(err) => Err(ServiceError::UnknownServiceError) // TODO use auth error perso,
            }
        },
        Err(err) => Err(ServiceError::SqliteError(err)) // TODO use auth error perso
    }

}