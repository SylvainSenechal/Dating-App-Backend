use argon2::{
    password_hash::{PasswordHash, PasswordVerifier, SaltString},
    Algorithm, Argon2, Params, PasswordHasher, Version,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use rand::thread_rng;
use std::sync::Arc;

use crate::configs::app_state::AppState;
use crate::data_access_layer;
use crate::data_access_layer::user_dal::User;
use crate::my_errors::service_errors::ServiceError;
use crate::my_errors::sqlite_errors::{transaction_error, SqliteError};
use crate::requests::requests;
use crate::responses::responses;
use crate::service_layer::auth_service::JwtClaims;
use crate::utilities::responses::{response_ok, response_ok_with_message, ApiResponse};
use crate::{
    constants::constants::{M_COST, OUTPUT_LEN, P_COST, T_COST},
    data_access_layer::user_dal::PotentialLover,
};

pub async fn create_user(
    State(state): State<Arc<AppState>>,
    Json(mut create_user_request): Json<requests::CreateUserRequest>,
) -> Result<(StatusCode, Json<ApiResponse<()>>), ServiceError> {
    let user = data_access_layer::user_dal::get_user_by_email(
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
            data_access_layer::user_dal::create_user(&state, create_user_request)?;
            response_ok_with_message(None::<()>, "user created".to_string())
        }
        Ok(_) => Err(ServiceError::UserAlreadyExist),
        Err(err) => Err(ServiceError::Sqlite(err)),
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
    let mut user_found = data_access_layer::user_dal::get_user_by_uuid(&state, user_uuid)?;
    user_found.latitude = user_found.latitude / std::f32::consts::PI * 180.;
    user_found.longitude = user_found.longitude / std::f32::consts::PI * 180.;
    response_ok(Some(user_found))
}

pub async fn delete_user(
    jwt_claims: JwtClaims,
    State(state): State<Arc<AppState>>,
    Path(user_uuid): Path<String>,
    Json(delete_user_request): Json<requests::DeleteUserRequest>,
) -> Result<(StatusCode, Json<ApiResponse<responses::MessageResponse>>), ServiceError> {
    if jwt_claims.user_uuid != user_uuid {
        return Err(ServiceError::ForbiddenQuery);
    }

    let password =
        data_access_layer::user_dal::get_user_password_by_user_uuid(&state, jwt_claims.user_uuid)?;
    let rehash = PasswordHash::new(&password).unwrap(); // Turning string into PHC string format type
    let valid_password =
        Argon2::default().verify_password(delete_user_request.password.as_bytes(), &rehash);

    match valid_password {
        Ok(_) => {
            data_access_layer::user_dal::delete_user_by_uuid(&state, user_uuid)?;
            response_ok_with_message(
                Some(responses::MessageResponse {
                    message: "user deleted successfully".to_string(),
                }),
                "user deleted successfully".to_string(),
            )
        }
        Err(_) => response_ok(Some(responses::MessageResponse {
            message: "wrong password".to_string(),
        })),
    }
}

pub async fn update_user(
    jwt_claims: JwtClaims,
    State(state): State<Arc<AppState>>,
    Path(user_uuid): Path<String>,
    Json(mut update_user_request): Json<requests::UpdateUserInfosReq>,
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
    update_user_request.longitude = update_user_request.longitude * std::f32::consts::PI / 180.;
    update_user_request.latitude = update_user_request.latitude * std::f32::consts::PI / 180.;
    // todo : check updated email is not taken
    data_access_layer::user_dal::update_user_infos(&state, update_user_request)?;
    response_ok_with_message(None::<()>, "user updated successfully".to_string())
}

pub async fn find_lover(
    jwt_claims: JwtClaims,
    State(state): State<Arc<AppState>>,
) -> Result<(StatusCode, Json<ApiResponse<PotentialLover>>), ServiceError> {
    let user = data_access_layer::user_dal::get_user_by_uuid(&state, jwt_claims.user_uuid.clone())?;
    let potential_lover = data_access_layer::user_dal::find_love_target(
        &state,
        jwt_claims.user_uuid,
        user.looking_for,
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
                _ => Err(ServiceError::Sqlite(err)),
            }
        }
    }
}

pub async fn swipe_user(
    jwt_claims: JwtClaims,
    State(state): State<Arc<AppState>>,
    Json(swipe_user_request): Json<requests::SwipeUserRequest>,
) -> Result<(StatusCode, Json<ApiResponse<responses::SwipeUserResponse>>), ServiceError> {
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
    match data_access_layer::user_dal::swipe_user(
        &state,
        jwt_claims.user_uuid.clone(),
        swipe_user_request.swiped_uuid.clone(),
        u8::from(swipe_user_request.love),
    ) {
        Ok(()) => {
            match data_access_layer::user_dal::check_mutual_love(
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
                            response_ok_with_message(
                                Some(responses::SwipeUserResponse::Matched),
                                "you matched !".to_string(),
                            )
                        }
                        Err(err) => {
                            state
                                .connection
                                .get()
                                .unwrap()
                                .execute("END TRANSACTION", [])
                                .map_err(transaction_error)?;
                            Err(ServiceError::Sqlite(err))
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
                    response_ok_with_message(
                        Some(responses::SwipeUserResponse::NotMatched),
                        "you love that person !".to_string(),
                    )
                }
                Err(err) => {
                    state
                        .connection
                        .get()
                        .unwrap()
                        .execute("END TRANSACTION", [])
                        .map_err(transaction_error)?;
                    Err(ServiceError::Sqlite(err))
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
            Err(ServiceError::Sqlite(err))
        }
    }
}
