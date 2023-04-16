use argon2::{password_hash::SaltString, Algorithm, Argon2, Params, PasswordHasher, Version};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use rand::thread_rng;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::data_access_layer::user_dal::User;
use crate::my_errors::service_errors::ServiceError;
use crate::my_errors::sqlite_errors::{transaction_error, SqliteError};
use crate::service_layer::auth_service::JwtClaims;
use crate::utilities::responses::{response_ok, response_ok_with_message, ApiResponse};
use crate::{
    constants::constants::{M_COST, OUTPUT_LEN, P_COST, T_COST},
    my_errors::service_errors,
};
use crate::{data_access_layer, AppState}; // todo : refactor into dto/dal logic

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct CreateUserRequest {
    pub name: String,
    pub password: String,
    pub email: String,
    pub age: u8,
    pub latitude: f32,
    pub longitude: f32,
    pub gender: String, // TODO add enum constraint
    pub looking_for: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SwipeUserRequest {
    pub swiped_uuid: String,
    pub love: bool, // boolean for sqlite, 0 = dont love, 1 - love
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateUserInfosReq {
    pub uuid: String,
    pub name: String,
    pub password: String,
    pub email: String,
    pub age: u8,
    pub latitude: f32,
    pub longitude: f32,
    pub gender: String,
    pub looking_for: String,
    pub search_radius: u16,
    pub looking_for_age_min: u8,
    pub looking_for_age_max: u8,
    pub description: String,
}

pub async fn create_user(
    State(state): State<Arc<AppState>>,
    Json(mut create_user_request): Json<CreateUserRequest>,
) -> Result<(StatusCode, Json<ApiResponse<()>>), ServiceError> {
    println!("{:?}", create_user_request);
    let user = data_access_layer::user_dal::User::get_user_by_email(
        &state,
        create_user_request.email.to_string(),
    );
    let hasher: Argon2 = Argon2::new(
        Algorithm::Argon2id,
        Version::V0x13,
        Params::new(M_COST, T_COST, P_COST, Some(OUTPUT_LEN))
            .expect("Failed to build params for Argon2id"),
    );

    let salt = SaltString::generate(&mut thread_rng());
    let hashed_password = hasher
        .hash_password(create_user_request.password.as_bytes(), &salt)
        .expect("Could not hash password");
    let phc_string = hashed_password.to_string();
    match user {
        Err(SqliteError::NotFound) => {
            create_user_request.password = phc_string;
            data_access_layer::user_dal::User::create_user(&state, create_user_request)?;
            response_ok_with_message(None::<()>, "user created".to_string())
        }
        Ok(_) => Err(ServiceError::UserAlreadyExist),
        Err(err) => Err(ServiceError::SqliteError(err)),
    }
}

pub async fn get_user(
    jwt_claims: JwtClaims,
    State(state): State<Arc<AppState>>,
    Path(user_uuid): Path<String>,
) -> Result<(StatusCode, Json<ApiResponse<User>>), ServiceError> {
    if jwt_claims.user_uuid != user_uuid {
        return Err(ServiceError::ForbiddenQuery);
    }
    let user_found = data_access_layer::user_dal::User::get_user_by_uuid(&state, user_uuid)?;
    response_ok(Some(user_found))
}

pub async fn delete_user(
    jwt_claims: JwtClaims,
    State(state): State<Arc<AppState>>,
    Path(user_uuid): Path<String>,
) -> Result<(StatusCode, Json<ApiResponse<()>>), ServiceError> {
    if jwt_claims.user_uuid != user_uuid {
        return Err(ServiceError::ForbiddenQuery);
    }
    data_access_layer::user_dal::User::delete_user_by_uuid(&state, user_uuid)?;
    response_ok_with_message(None::<()>, "user deleted successfully".to_string())
}

// This route is useless and dangerous..
// pub async fn get_users(db: web::Data<AppState>) -> actixResult<HttpResponse, ServiceError> {
//     let users_found = data_access_layer::user_dal::User::get_users(&db);
//     match users_found {
//         Ok(users) => Ok(HttpResponse::Ok().json(users)),
//         Err(err) => Err(ServiceError::SqliteError(err)),
//     }
// }

pub async fn update_user(
    jwt_claims: JwtClaims,
    State(state): State<Arc<AppState>>,
    Path(user_uuid): Path<String>,
    Json(update_user_request): Json<UpdateUserInfosReq>,
) -> Result<(StatusCode, Json<ApiResponse<()>>), ServiceError> {
    if jwt_claims.user_uuid != user_uuid {
        return Err(ServiceError::ForbiddenQuery);
    }
    if update_user_request.description.chars().count() > 1000 {
        // Warning : Be carefull when counting string chars(), this needs tests..
        return Err(ServiceError::ValueNotAccepted(
            update_user_request.description,
            "Description string is too long".to_string(),
        ));
    }
    if update_user_request.latitude < -90.0 || update_user_request.latitude > 90.0 {
        return Err(ServiceError::ValueNotAccepted(
            update_user_request.latitude.to_string(),
            "latitude should be between -90 and +90".to_string(),
        ));
    }
    if update_user_request.longitude < -180.0 || update_user_request.longitude > 180.0 {
        return Err(ServiceError::ValueNotAccepted(
            update_user_request.longitude.to_string(),
            "longitude should be between -180 and +180".to_string(),
        ));
    }
    data_access_layer::user_dal::User::update_user_infos(&state, update_user_request)?;
    response_ok_with_message(None::<()>, "user updated successfully".to_string())
}

pub async fn find_lover(
    jwt_claims: JwtClaims,
    State(state): State<Arc<AppState>>,
) -> Result<(StatusCode, Json<ApiResponse<User>>), ServiceError> {
    let user =
        data_access_layer::user_dal::User::get_user_by_uuid(&state, jwt_claims.user_uuid.clone());
    let user = match user {
        Ok(user) => user,
        Err(err) => return Err(ServiceError::SqliteError(err)),
    };

    let potential_lover = data_access_layer::user_dal::User::find_love_target(
        &state,
        jwt_claims.user_uuid,
        user.looking_for,
        user.gender,
        user.search_radius,
        user.latitude,
        user.longitude,
        user.looking_for_age_min,
        user.looking_for_age_max,
    );

    match potential_lover {
        Ok(user) => {
            response_ok_with_message(Some(user), "you found a potential lover !".to_string())
        }
        Err(err) => {
            match err {
                SqliteError::NotFound => Err(ServiceError::NoPotentialMatchFound), // TODO response for this
                _ => Err(ServiceError::SqliteError(err)),
            }
        }
    }
}

// Todo : swiper should only be able to swipe a user given by the backend (here you can set swiped_id to be anybody, inclduing yourself..)
pub async fn swipe_user(
    jwt_claims: JwtClaims,
    State(state): State<Arc<AppState>>,
    Json(swipe_user_request): Json<SwipeUserRequest>,
) -> Result<(StatusCode, Json<ApiResponse<()>>), ServiceError> {
    println!("{:?}", swipe_user_request);
    if jwt_claims.user_uuid == swipe_user_request.swiped_uuid {
        // Cannot swipe yourself..
        return Err(ServiceError::ForbiddenQuery);
    }

    // todo : refactor transaction here
    state
        .connection
        .get()
        .unwrap()
        .execute("BEGIN TRANSACTION", [])
        .map_err(transaction_error)?;
    match data_access_layer::user_dal::User::swipe_user(
        &state,
        jwt_claims.user_uuid.clone(),
        swipe_user_request.swiped_uuid.clone(),
        u8::from(swipe_user_request.love),
    ) {
        Ok(()) => {
            match data_access_layer::user_dal::User::check_mutual_love(
                &state,
                jwt_claims.user_uuid.clone(),
                swipe_user_request.swiped_uuid.clone(),
            ) {
                Ok(2) => {
                    match data_access_layer::lover_dal::create_lovers(
                        &state,
                        jwt_claims.user_uuid,
                        swipe_user_request.swiped_uuid,
                    ) {
                        Ok(_) => {
                            state
                                .connection
                                .get()
                                .unwrap()
                                .execute("END TRANSACTION", [])
                                .map_err(transaction_error)?;
                            response_ok_with_message(None::<()>, "you matched !".to_string())
                        }
                        Err(err) => {
                            state
                                .connection
                                .get()
                                .unwrap()
                                .execute("END TRANSACTION", [])
                                .map_err(transaction_error)?;
                            Err(ServiceError::SqliteError(err))
                        }
                    }
                }
                Ok(_) => {
                    state
                        .connection
                        .get()
                        .unwrap()
                        .execute("END TRANSACTION", [])
                        .map_err(transaction_error)?;
                    response_ok_with_message(None::<()>, "you love that person !".to_string())
                }
                Err(err) => {
                    state
                        .connection
                        .get()
                        .unwrap()
                        .execute("END TRANSACTION", [])
                        .map_err(transaction_error)?;
                    Err(ServiceError::SqliteError(err))
                }
            }
        }
        Err(err) => {
            state
                .connection
                .get()
                .unwrap()
                .execute("END TRANSACTION", [])
                .map_err(transaction_error)?;
            Err(ServiceError::SqliteError(err))
        }
    }
}
