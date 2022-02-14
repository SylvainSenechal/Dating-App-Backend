use actix_web::{web, HttpResponse, Result as actixResult};

use crate::my_errors::service_errors::ServiceError;
use crate::my_errors::sqlite_errors::transaction_error;
use crate::service_layer::auth_service::AuthorizationUser;
use crate::{data_access_layer, AppState};

pub async fn get_lovers(
    authorized: AuthorizationUser,
    db: web::Data<AppState>,
    web::Path(user_id): web::Path<usize>
) -> actixResult<HttpResponse, ServiceError> {
    if authorized.id != user_id {
        return Err(ServiceError::ForbiddenQuery);
    }
    db.connection
        .execute("BEGIN TRANSACTION", [])
        .map_err(transaction_error)?;
    let lovers_found = data_access_layer::lover_dal::get_lovers(&db, user_id);
    db.connection
        .execute("END TRANSACTION", [])
        .map_err(transaction_error)?;
    match lovers_found {
        Ok(lovers) => Ok(HttpResponse::Ok().json(lovers)),
        Err(err) => Err(ServiceError::SqliteError(err)),
    }
}
