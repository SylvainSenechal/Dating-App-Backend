use serde::{Deserialize, Serialize};
use actix_web::{web, HttpResponse, Result as actixResult};
use argon2::{Algorithm, Argon2, Params, PasswordHasher, Version, password_hash::SaltString};
use rand::thread_rng;

use crate::{AppState, data_access_layer};
use crate::my_errors::sqlite_errors::SqliteError;
use crate::data_access_layer::user_dal::{User};
use crate::my_errors::service_errors::ServiceError;
use crate::constants::constants::{M_COST, T_COST, P_COST, OUTPUT_LEN};
use crate::service_layer::auth_service::AuthorizationUser;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct CreateUserRequest {
    pub pseudo: String,
    pub password: String,
    pub email: String,
    pub age: u8,
    pub latitude: f32,
    pub longitude: f32,
    pub gender: String,
    pub looking_for: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateUserInfosReq {
    pub id: u32,
    pub pseudo: String,
    pub password: String,
    pub email: String,
    pub age: u8,
    pub latitude: f32,
    pub longitude: f32,
    pub gender: String,
    pub looking_for: String
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct CreateUserResponse {
    message: String
}

pub async fn create_user(
    db: web::Data<AppState>,
    mut create_user_request: web::Json<CreateUserRequest>,
) -> actixResult<HttpResponse, ServiceError> {
    println!("{:?}", create_user_request);
    let user = data_access_layer::user_dal::User::get_user_by_email(&db, create_user_request.email.to_string());
    let hasher: Argon2 = Argon2::new(
        Algorithm::Argon2id,
        Version::V0x13,
        Params::new(M_COST, T_COST, P_COST, Some(OUTPUT_LEN))
            .expect("Failed to build params for Argon2id") // TODO : clean error
    );
    
    let salt = SaltString::generate(&mut thread_rng());
    let hashed_password = hasher.hash_password(create_user_request.password.as_bytes(), &salt)
        .expect("Could not hash password"); // TODO : clean error
    let phc_string = hashed_password.to_string();
    match user {
        Err(SqliteError::NotFound) => {
            // todo : check the option age field : post with and without age, with and without impl default
            create_user_request.password = phc_string;
            match data_access_layer::user_dal::User::create_user(&db, create_user_request.into_inner()) {
                Ok(()) => Ok(HttpResponse::Ok().json(CreateUserResponse{message: "User created".to_string()})),
                Err(err) => Err(ServiceError::SqliteError(err))
            }
        },
        Ok(_) => {
            Err(ServiceError::UserAlreadyExist)
        }
        Err(err) => Err(ServiceError::SqliteError(err))
    }
}

pub async fn get_user(db: web::Data<AppState>, web::Path(userId): web::Path<u32>) -> actixResult<HttpResponse, ServiceError> {
    let user_found = data_access_layer::user_dal::User::get_user_by_id(&db, userId);
    match user_found {
        Ok(user) => Ok(HttpResponse::Ok().json(user)),
        Err(err) => Err(ServiceError::SqliteError(err))
    }
}

pub async fn get_users(db: web::Data<AppState>) -> actixResult<HttpResponse, ServiceError> {
    let users_found = data_access_layer::user_dal::User::get_users(&db);
    match users_found {
        Ok(users) => Ok(HttpResponse::Ok().json(users)),
        Err(err) => Err(ServiceError::SqliteError(err))
    }    
}

pub async fn update_user(
    authorized: AuthorizationUser,
    db: web::Data<AppState>,
    web::Path(userId): web::Path<u32>,
    mut update_user_request: web::Json<UpdateUserInfosReq>
) -> actixResult<HttpResponse, ServiceError> {
    // TODO verifier l id et l'id du jwt sont les memes
    let update_status = data_access_layer::user_dal::User::update_user_infos(&db, update_user_request.into_inner());
    match update_status {
        Ok(()) => Ok(HttpResponse::Ok().body("sucess")),
        Err(err) => Err(ServiceError::SqliteError(err))
    }    
}