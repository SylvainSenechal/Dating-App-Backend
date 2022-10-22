use actix_web::{web, HttpResponse, Result as actixResult};

use crate::my_errors::service_errors::ServiceError;
use crate::service_layer::auth_service::AuthorizationUser;
use crate::utilities;
use crate::{data_access_layer, AppState};

pub async fn loved_count(
    // How many users loved you
    authorized: AuthorizationUser,
    db: web::Data<AppState>,
    web::Path(user_id): web::Path<usize>,
) -> actixResult<HttpResponse, ServiceError> {
    if authorized.id != user_id {
        return Err(ServiceError::ForbiddenQuery);
    }
    let swiped_count = data_access_layer::user_dal::User::swiped_count(&db, authorized.id, 1);
    match swiped_count {
        Ok(count) => Ok(utilities::responses::response_ok(Some(count))),
        Err(err) => Err(ServiceError::SqliteError(err)),
    }
}

pub async fn rejected_count(
    // How many users rejected you
    authorized: AuthorizationUser,
    db: web::Data<AppState>,
    web::Path(user_id): web::Path<usize>,
) -> actixResult<HttpResponse, ServiceError> {
    if authorized.id != user_id {
        return Err(ServiceError::ForbiddenQuery);
    }
    let swiped_count = data_access_layer::user_dal::User::swiped_count(&db, authorized.id, 0);
    match swiped_count {
        Ok(count) => Ok(utilities::responses::response_ok(Some(count))),
        Err(err) => Err(ServiceError::SqliteError(err)),
    }
}

pub async fn loving_count(
    // How many users you loved
    authorized: AuthorizationUser,
    db: web::Data<AppState>,
    web::Path(user_id): web::Path<usize>,
) -> actixResult<HttpResponse, ServiceError> {
    if authorized.id != user_id {
        return Err(ServiceError::ForbiddenQuery);
    }
    let swiped_count = data_access_layer::user_dal::User::swiping_count(&db, authorized.id, 1);
    match swiped_count {
        Ok(count) => Ok(utilities::responses::response_ok(Some(count))),
        Err(err) => Err(ServiceError::SqliteError(err)),
    }
}

pub async fn rejecting_count(
    // How many users you rejected
    authorized: AuthorizationUser,
    db: web::Data<AppState>,
    web::Path(user_id): web::Path<usize>,
) -> actixResult<HttpResponse, ServiceError> {
    if authorized.id != user_id {
        return Err(ServiceError::ForbiddenQuery);
    }
    let swiped_count = data_access_layer::user_dal::User::swiping_count(&db, authorized.id, 0);
    match swiped_count {
        Ok(count) => Ok(utilities::responses::response_ok(Some(count))),
        Err(err) => Err(ServiceError::SqliteError(err)),
    }
}

pub async fn backend_activity(
    authorized: AuthorizationUser,
    db: web::Data<AppState>,
) -> actixResult<HttpResponse, ServiceError> {
    let traces = data_access_layer::trace_dal::get_traces(&db);
    match traces {
        Ok(t) => Ok(utilities::responses::response_ok(Some(t))),
        Err(err) => Err(ServiceError::SqliteError(err)),
    }
}
