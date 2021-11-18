use serde::{Deserialize, Serialize};
use actix_web::{middleware, web, App, HttpResponse, HttpServer, Result as actixResult};
use argon2::{
    Algorithm, Argon2, Error, Params, ParamsBuilder, PasswordHash, PasswordHasher,
    PasswordVerifier, Version, password_hash::SaltString,
};
use rand::thread_rng;

use crate::{AppState, data_access_layer};
use crate::my_errors::sqlite_errors::SqliteError;
// use crate::data_access_layer::users;

const M_COST: u32 = 15_000;// m_cost is the memory size, expressed in kilobytes
const T_COST: u32 = 4; // t_cost is the number of iterations;
const P_COST: u32 = 1; //p_cost is the degree of parallelism.
const OUTPUT_LEN: usize = 32; // determines the length of the returned hash in bytes

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct CreateUser {
    #[serde(default)]
    pseudo: String,
    #[serde(default)]
    email: String,
    #[serde(default)]
    age: u8,
}

pub async fn create_user(
    db: web::Data<AppState>,
    create_user_request: web::Json<CreateUser>,
) -> actixResult<HttpResponse, SqliteError> {

    let user = data_access_layer::users::User::get_user(&db, create_user_request.pseudo.to_string()).await;
    println!("eeeee : {:?}", user);

    let hasher: Argon2 = Argon2::new(
        Algorithm::Argon2id,
        Version::V0x13,
        Params::new(M_COST, T_COST, P_COST, Some(OUTPUT_LEN))
            .expect("Failed to build params for Argon2id") // TODO : clean error
    );
    
    let salt = SaltString::generate(&mut thread_rng());
    let hashed_password = hasher.hash_password("nulPass".as_bytes(), &salt)
        .expect("Could not hash password"); // TODO : clean error

    let phc_string = hashed_password.to_string();
    // println!("hash strinnnnggg {}", hashed_password);
    // println!("hash strinnnnggg {}", phc_string);
    // let rehash = PasswordHash::new(&phc_string).unwrap();

    // Argon2::default().verify_password("nulPass".as_bytes(), &hashed_password).expect("could not verify");
    // Argon2::default().verify_password("nulPass".as_bytes(), &rehash).expect("could not verify");

    match user {
        Ok(_) => println!("found user"), // todo return err
        Err(SqliteError::NotFound) => {
            println!("user not found");
            let user = data_access_layer::users::User{
                pseudo: create_user_request.pseudo.to_string(),
                email: create_user_request.email.to_string(),
                password: phc_string,
                age: Some(create_user_request.age)
            };
            data_access_layer::users::User::create_user(&db, user).await?;
            return Ok(HttpResponse::Ok().body("User created"))
        },
        _ => println!("Sqlite error"), // todo return err
    }
    Ok(HttpResponse::Ok().body("User created"))
}