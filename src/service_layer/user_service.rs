use actix_web::{web, HttpResponse, Result as actixResult};
use argon2::{password_hash::SaltString, Algorithm, Argon2, Params, PasswordHasher, Version};
use rand::thread_rng;
use serde::{Deserialize, Serialize};

use crate::constants::constants::{M_COST, OUTPUT_LEN, P_COST, T_COST};
use crate::my_errors::service_errors::ServiceError;
use crate::my_errors::sqlite_errors::{transaction_error, SqliteError};
use crate::service_layer::auth_service::AuthorizationUser;
use crate::utilities;
use crate::{data_access_layer, AppState};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct CreateUserRequest {
    pub name: String,
    pub password: String,
    pub email: String,
    pub age: u8,
    pub latitude: f32,
    pub longitude: f32,
    pub gender: String,
    pub looking_for: String,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct SwipeUserRequest {
    pub swiper: usize,
    pub swiped: usize,
    pub love: u8, // boolean for sqlite, 0 = dont love, 1 - love
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateUserInfosReq {
    pub id: usize,
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
    db: web::Data<AppState>,
    mut create_user_request: web::Json<CreateUserRequest>,
) -> actixResult<HttpResponse, ServiceError> {
    println!("{:?}", create_user_request);
    let user = data_access_layer::user_dal::User::get_user_by_email(
        &db,
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
            match data_access_layer::user_dal::User::create_user(
                &db,
                create_user_request.into_inner(),
            ) {
                Ok(()) => Ok(utilities::responses::response_ok_with_message(
                    None::<()>,
                    "User created".to_string(),
                )),
                Err(err) => Err(ServiceError::SqliteError(err)),
            }
        }
        Ok(_) => Err(ServiceError::UserAlreadyExist),
        Err(err) => Err(ServiceError::SqliteError(err)),
    }
}

pub async fn get_user(
    authorized: AuthorizationUser,
    db: web::Data<AppState>,
    web::Path(user_id): web::Path<usize>,
) -> actixResult<HttpResponse, ServiceError> {
    if authorized.id != user_id {
        return Err(ServiceError::ForbiddenQuery);
    }
    let user_found = data_access_layer::user_dal::User::get_user_by_id(&db, 30);
    match user_found {
        Ok(user) => Ok(utilities::responses::response_ok(Some(user))),
        Err(err) => Err(ServiceError::SqliteError(err)),
    }
}

pub async fn delete_user(
    authorized: AuthorizationUser,
    db: web::Data<AppState>,
    web::Path(user_id): web::Path<usize>,
) -> actixResult<HttpResponse, ServiceError> {
    if authorized.id != user_id {
        return Err(ServiceError::ForbiddenQuery);
    }
    let deleted = data_access_layer::user_dal::User::delete_user_by_id(&db, user_id);
    match deleted {
        Ok(()) => Ok(utilities::responses::response_ok_with_message(
            None::<()>,
            "User deleted successfully".to_string(),
        )),
        Err(err) => Err(ServiceError::SqliteError(err)),
    }
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
    authorized: AuthorizationUser,
    db: web::Data<AppState>,
    web::Path(user_id): web::Path<usize>,
    update_user_request: web::Json<UpdateUserInfosReq>,
) -> actixResult<HttpResponse, ServiceError> {
    if authorized.id != user_id {
        return Err(ServiceError::ForbiddenQuery);
    }
    if update_user_request.description.chars().count() > 1000 {
        // Warning : Be carefull when counting string chars(), this needs tests..
        return Err(ServiceError::ValueNotAccepted(
            update_user_request.description.to_string(),
            "Description string is too long".to_string(),
        ));
    }
    let update_status =
        data_access_layer::user_dal::User::update_user_infos(&db, update_user_request.into_inner());
    match update_status {
        Ok(()) => Ok(utilities::responses::response_ok_with_message(
            None::<()>,
            "User updated successfully".to_string(),
        )),
        Err(err) => Err(ServiceError::SqliteError(err)),
    }
}

pub async fn find_love(
    authorized: AuthorizationUser,
    db: web::Data<AppState>,
    web::Path(user_id): web::Path<usize>,
) -> actixResult<HttpResponse, ServiceError> {
    if authorized.id != user_id {
        return Err(ServiceError::ForbiddenQuery);
    }
    let user = data_access_layer::user_dal::User::get_user_by_id(&db, authorized.id);
    let user = match user {
        Ok(user) => user,
        Err(err) => return Err(ServiceError::SqliteError(err)),
    };

    let potential_lover = data_access_layer::user_dal::User::find_love_target(
        &db,
        user_id,
        user.looking_for,
        user.gender,
        user.search_radius,
        user.latitude,
        user.longitude,
        user.looking_for_age_min,
        user.looking_for_age_max,
    );

    match potential_lover {
        Ok(user) => Ok(HttpResponse::Ok().json(user)),
        Err(err) => {
            match err {
                SqliteError::NotFound => {
                    Ok(HttpResponse::NotFound().json("Could not find a fitting potential lover"))
                } // TODO response for this
                _ => Err(ServiceError::SqliteError(err)),
            }
        }
    }
}

// Todo : swiper should only be able to swipe a user given by the backend (here you can set swiped_id to be anybody, inclduing yourself..)
pub async fn swipe_user(
    authorized: AuthorizationUser,
    db: web::Data<AppState>,
    web::Path((swiper_id, swiped_id)): web::Path<(usize, usize)>,
    swipe_user_request: web::Json<SwipeUserRequest>,
) -> actixResult<HttpResponse, ServiceError> {
    if authorized.id != swiper_id {
        return Err(ServiceError::ForbiddenQuery);
    }
    println!("{:?}", swipe_user_request);

    db.connection
        .execute("BEGIN TRANSACTION", [])
        .map_err(transaction_error)?;
    match data_access_layer::user_dal::User::swipe_user(
        &db,
        swipe_user_request.swiper,
        swipe_user_request.swiped,
        swipe_user_request.love,
    ) {
        Ok(()) => {
            match data_access_layer::user_dal::User::check_mutual_love(&db, swiper_id, swiped_id) {
                Ok(2) => {
                    match data_access_layer::lover_dal::create_lovers(&db, swiper_id, swiped_id) {
                        Ok(_) => {
                            db.connection
                                .execute("END TRANSACTION", [])
                                .map_err(transaction_error)?;
                            Ok(utilities::responses::response_ok_with_message(
                                None::<()>,
                                "You matched !".to_string(),
                            ))
                        }
                        Err(err) => {
                            db.connection
                                .execute("END TRANSACTION", [])
                                .map_err(transaction_error)?;
                            Err(ServiceError::SqliteError(err))
                        }
                    }
                }
                Ok(_) => {
                    db.connection
                        .execute("END TRANSACTION", [])
                        .map_err(transaction_error)?;
                    Ok(utilities::responses::response_ok_with_message(
                        None::<()>,
                        "You love that person !".to_string(),
                    ))
                }
                Err(err) => {
                    db.connection
                        .execute("END TRANSACTION", [])
                        .map_err(transaction_error)?;
                    Err(ServiceError::SqliteError(err))
                }
            }
        }
        Err(err) => {
            db.connection
                .execute("END TRANSACTION", [])
                .map_err(transaction_error)?;
            Err(ServiceError::SqliteError(err))
        }
    }
}
