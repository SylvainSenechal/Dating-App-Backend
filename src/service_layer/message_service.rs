use actix_web::{web, HttpResponse, Result as actixResult};
use serde::{Deserialize, Serialize};

use crate::my_errors::service_errors::ServiceError;
use crate::service_layer::auth_service::AuthorizationUser;
use crate::{data_access_layer, AppState};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct CreateMessageRequest {
    pub message: String,
    pub poster_id: u32,
    pub love_id: u32,
    // TODO : add date
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct CreateMessageResponse { // TODO : this might be redundant with CreateUserResponse 
    message: String,
}

pub async fn get_messages(
    authorized: AuthorizationUser,
    db: web::Data<AppState>,
    web::Path(love_id): web::Path<u32>
) -> actixResult<HttpResponse, ServiceError> {
    // TODO !!! : check that the requester is present in the love relation
    let messages_found = data_access_layer::message_dal::get_messages(&db, love_id);
    match messages_found {
        Ok(messages) => Ok(HttpResponse::Ok().json(messages)),
        Err(err) => Err(ServiceError::SqliteError(err)),
    }
}

pub async fn create_message(
    db: web::Data<AppState>,
    create_message_request: web::Json<CreateMessageRequest>,
) -> actixResult<HttpResponse, ServiceError> {
    println!("{:?}", create_message_request);
    // TODO !!! : check that the poster is present in the love relation
    match data_access_layer::message_dal::create_message(&db, create_message_request.into_inner()) {
        Ok(()) => Ok(HttpResponse::Ok().json(CreateMessageResponse {
            message: "Message created".to_string(),
        })),
        Err(err) => Err(ServiceError::SqliteError(err)),
    }
}
