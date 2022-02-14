use std::ops::Deref;

use crate::my_errors::service_errors::ServiceError;
use crate::my_errors::sqlite_errors::{transaction_error, SqliteError};
use crate::service_layer::auth_service::AuthorizationUser;
use crate::service_layer::websocket_service::{ChatMessage, Server};
use crate::service_layer::MessageServiceResponse;
use crate::{data_access_layer, AppState};
use actix::Addr;
use actix_web::{web, HttpResponse, Result as actixResult};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct CreateMessageRequest {
    pub message: String,
    pub poster_id: usize,
    pub love_id: usize,
    // TODO : add date
}

// Post a message by poster_id in the love_id relation
pub async fn create_message(
    authorized: AuthorizationUser,
    db: web::Data<AppState>,
    create_message_request: web::Json<CreateMessageRequest>,
    server: web::Data<Addr<Server>>,
) -> actixResult<HttpResponse, ServiceError> {
    println!("{:?}", create_message_request);
    if authorized.id != create_message_request.poster_id {
        return Err(ServiceError::ForbiddenQuery);
    }
    match data_access_layer::lover_dal::user_in_love_relation(
        &db,
        create_message_request.poster_id,
        create_message_request.love_id,
    ) {
        Ok(_) => (),
        Err(err) => match err {
            SqliteError::NotFound => return Err(ServiceError::ForbiddenQuery),
            _ => return Err(ServiceError::UnknownServiceError),
        },
    }
    if create_message_request.message.chars().count() > 1000 {
        // Warning : Be carefull when counting string chars(), this needs tests..
        return Err(ServiceError::ValueNotAccepted(
            create_message_request.message.to_string(),
            "Message content string is too long".to_string(),
        ));
    }
    db.connection
        .execute("BEGIN TRANSACTION", [])
        .map_err(transaction_error)?;
    let result_creation_message =
        data_access_layer::message_dal::create_message(&db, create_message_request.deref());
    db.connection
        .execute("END TRANSACTION", [])
        .map_err(transaction_error)?;
    match result_creation_message {
        Ok(id_message) => {
            println!("message {} created", id_message);
            server.do_send(ChatMessage {
                id_love_room: create_message_request.love_id,
                id_message: id_message,
                message: create_message_request.message.to_string(),
                poster_id: create_message_request.poster_id,
            });
            Ok(HttpResponse::Ok().json(MessageServiceResponse {
                message: "Message created".to_string(),
            }))
        }
        Err(err) => Err(ServiceError::SqliteError(err)),
    }
}

// Get messages of one "love_id" love relations
pub async fn get_love_messages(
    authorized: AuthorizationUser,
    db: web::Data<AppState>,
    web::Path(love_id): web::Path<usize>,
) -> actixResult<HttpResponse, ServiceError> {
    match data_access_layer::lover_dal::user_in_love_relation(&db, authorized.id, love_id) {
        Ok(_) => println!(
            "{} user allowed to get messages of {} love relationship",
            authorized.id, love_id
        ),
        Err(err) => match err {
            SqliteError::NotFound => return Err(ServiceError::ForbiddenQuery),
            _ => return Err(ServiceError::UnknownServiceError),
        },
    }
    let messages_found = data_access_layer::message_dal::get_love_messages(&db, love_id);
    match messages_found {
        Ok(messages) => Ok(HttpResponse::Ok().json(messages)),
        Err(err) => Err(ServiceError::SqliteError(err)),
    }
}

// Get all the messages of all the love relation of "user_id"
pub async fn get_lover_messages(
    authorized: AuthorizationUser,
    db: web::Data<AppState>,
    web::Path(user_id): web::Path<usize>,
) -> actixResult<HttpResponse, ServiceError> {
    if authorized.id != user_id {
        return Err(ServiceError::ForbiddenQuery);
    }
    let messages_found = data_access_layer::message_dal::get_lover_messages(&db, user_id);
    match messages_found {
        Ok(messages) => Ok(HttpResponse::Ok().json(messages)),
        Err(err) => Err(ServiceError::SqliteError(err)),
    }
}
