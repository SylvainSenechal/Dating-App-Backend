use actix_web::{web, HttpResponse, Result as actixResult};

use crate::my_errors::service_errors::ServiceError;
use crate::service_layer::auth_service::AuthorizationUser;
use crate::{data_access_layer, AppState};

pub async fn get_lovers(
    authorized: AuthorizationUser,
    db: web::Data<AppState>,
    web::Path(user_id): web::Path<u32>
) -> actixResult<HttpResponse, ServiceError> {
    if authorized.id != user_id {
        return Err(ServiceError::UnknownServiceError);
    }
    let lovers_found = data_access_layer::lover_dal::get_lovers(&db, authorized.id);
    match lovers_found {
        Ok(lovers) => Ok(HttpResponse::Ok().json(lovers)),
        Err(err) => Err(ServiceError::SqliteError(err)),
    }
}
