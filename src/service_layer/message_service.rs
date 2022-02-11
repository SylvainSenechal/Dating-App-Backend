use std::ops::Deref;

use crate::my_errors::service_errors::ServiceError;
use crate::service_layer::auth_service::AuthorizationUser;
use crate::service_layer::websocket_service::{ChatMessage, Server};
use crate::{data_access_layer, AppState};
use actix::Addr;
use actix_web::{web, HttpResponse, Result as actixResult};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct CreateMessageRequest {
    pub message: String,
    pub poster_id: u32,
    pub love_id: u32,
    // TODO : add date
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct CreateMessageResponse {
    // TODO : this might be redundant with CreateUserResponse
    message: String,
}

pub async fn get_love_messages(
    authorized: AuthorizationUser,
    db: web::Data<AppState>,
    web::Path(love_id): web::Path<u32>,
) -> actixResult<HttpResponse, ServiceError> {
    // TODO !!! : check that the requester is present in the love relation
    let messages_found = data_access_layer::message_dal::get_love_messages(&db, love_id);
    match messages_found {
        Ok(messages) => Ok(HttpResponse::Ok().json(messages)),
        Err(err) => Err(ServiceError::SqliteError(err)),
    }
}

pub async fn get_lover_messages(
    authorized: AuthorizationUser,
    db: web::Data<AppState>,
    web::Path(user_id): web::Path<u32>,
) -> actixResult<HttpResponse, ServiceError> {
    // TODO !!! : check that the requester is present in the love relation
    let messages_found = data_access_layer::message_dal::get_lover_messages(&db, user_id);
    println!("{:?}", messages_found);
    match messages_found {
        Ok(messages) => Ok(HttpResponse::Ok().json(messages)),
        Err(err) => Err(ServiceError::SqliteError(err)),
    }
}

pub async fn create_message(
    db: web::Data<AppState>,
    create_message_request: web::Json<CreateMessageRequest>,
    server: web::Data<Addr<Server>>,
) -> actixResult<HttpResponse, ServiceError> {
    println!("{:?}", create_message_request);
    // TODO !!! : check that the poster is present in the love relation
    match data_access_layer::message_dal::create_message(&db, create_message_request.deref()) {
        Ok(id_message) => {
            println!("message {} created", id_message);
            server.do_send(ChatMessage {
                id_love_room: create_message_request.love_id,
                id_message: id_message,
                message: create_message_request.message.to_string(),
                poster_id: create_message_request.poster_id,
            });
            Ok(HttpResponse::Ok().json(CreateMessageResponse {
                message: "Message created".to_string(),
            }))
        }
        Err(err) => Err(ServiceError::SqliteError(err)),
    }
}
