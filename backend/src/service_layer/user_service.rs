use serde::{Deserialize, Serialize};
use actix_web::{middleware, web, App, HttpResponse, HttpServer, Result as actixResult};
use argon2::{
    Algorithm, Argon2, Error, Params, ParamsBuilder, PasswordHash, PasswordHasher,
    PasswordVerifier, Version, password_hash::SaltString,
};
use rand::thread_rng;

use crate::{AppState, data_access_layer};
use crate::my_errors::sqlite_errors::SqliteError;
use crate::my_errors::service_errors::ServiceError;

// https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html
// ==> "Use Argon2id with a minimum configuration of 15 MiB of memory, an iteration count of 2, and 1 degree of parallelism."
const M_COST: u32 = 15_000;// m_cost is the memory size, expressed in kilobytes
const T_COST: u32 = 2; // t_cost is the number of iterations;
const P_COST: u32 = 1; //p_cost is the degree of parallelism.
const OUTPUT_LEN: usize = 32; // determines the length of the returned hash in bytes

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct CreateUser {
    pseudo: String,
    email: String,
    password: String,
    #[serde(default)]
    age: u8,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct GetUser {
    pseudo: String,
}

// todo tester plein de requete pour voir comportement async
pub async fn create_user(
    db: web::Data<AppState>,
    create_user_request: web::Json<CreateUser>,
) -> actixResult<HttpResponse, ServiceError> {

    let user = data_access_layer::user_dal::User::get_user(&db, create_user_request.pseudo.to_string()).await;

    let hasher: Argon2 = Argon2::new(
        Algorithm::Argon2id,
        Version::V0x13,
        Params::new(M_COST, T_COST, P_COST, Some(OUTPUT_LEN))
            .expect("Failed to build params for Argon2id") // TODO : clean error
    );
    
    let salt = SaltString::generate(&mut thread_rng());

    // TODO : This hashing is expensive and blocking, compute in async function
    let hashed_password = hasher.hash_password(create_user_request.password.as_bytes(), &salt)
        .expect("Could not hash password"); // TODO : clean error

    let phc_string = hashed_password.to_string();
    println!("hash strinnnnggg {}", hashed_password);
    println!("hash strinnnnggg {}", phc_string);
    let rehash = PasswordHash::new(&phc_string).unwrap();

    Argon2::default().verify_password("nulPass".as_bytes(), &hashed_password).expect("could not verify");
    Argon2::default().verify_password("nulPass".as_bytes(), &rehash).expect("could not verify");

    match user {
        Err(SqliteError::NotFound) => {
            let user = data_access_layer::user_dal::User{
                pseudo: create_user_request.pseudo.to_string(),
                email: create_user_request.email.to_string(),
                password: phc_string,
                age: Some(create_user_request.age) // todo : check this some thing
            };
            match data_access_layer::user_dal::User::create_user(&db, user).await {
                Ok(()) => Ok(HttpResponse::Ok().body("User created")),
                Err(err) => Err(ServiceError::SqliteError(err))
            }
             
        },
        Ok(_) => Err(ServiceError::ServiceError("This user already exists".to_string())),
        Err(err) => Err(ServiceError::SqliteError(err))
    }
}

pub async fn get_user(db: web::Data<AppState>, web::Path(pseudo): web::Path<String>) -> actixResult<HttpResponse, ServiceError> {
    let user_found = data_access_layer::user_dal::User::get_user(&db, pseudo).await;
    match user_found {
        Ok(user) => Ok(HttpResponse::Ok().json(user)),
        Err(err) => Err(ServiceError::SqliteError(err))
    }
}

pub async fn get_users(db: web::Data<AppState>) -> actixResult<HttpResponse, ServiceError> {
    let users_found = data_access_layer::user_dal::User::get_users(&db).await;
    match users_found {
        Ok(users) => Ok(HttpResponse::Ok().json(users)),
        Err(err) => Err(ServiceError::SqliteError(err))
    }    
}