use std::ops::Deref;

use crate::my_errors::service_errors::ServiceError;
use crate::my_errors::sqlite_errors::{transaction_error, SqliteError};
use crate::service_layer::auth_service::AuthorizationUser;
use crate::service_layer::websocket_service::{ChatMessage, GreenTickMessage, Server};
use crate::{data_access_layer, utilities, AppState};
use actix::Addr;
use actix_web::{web, HttpResponse, Result as actixResult};
use chrono;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct CreateMessageRequest {
    pub message: String,
    pub poster_id: usize,
    pub love_id: usize,
}

#[derive(Deserialize)]
pub struct GreenTickMessagesRequest {
    pub messages: Vec<GreenTickMessageRequest>,
}

#[derive(Deserialize)]
pub struct GreenTickMessageRequest {
    pub message_id: usize,
    pub love_id: usize,
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
    if create_message_request.message.len() == 0 {
        return Err(ServiceError::ValueNotAccepted(
            create_message_request.message.to_string(),
            "Empty messages not accepted".to_string(),
        ));
    }
    if create_message_request.message.chars().count() > 1000 {
        // Warning : Be carefull when counting string chars(), this needs tests..
        return Err(ServiceError::ValueNotAccepted(
            create_message_request.message.to_string(),
            "Message content string is too long".to_string(),
        ));
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

    let creation_datetime = format!("{:?}", chrono::offset::Utc::now());
    db.connection
        .execute("BEGIN TRANSACTION", [])
        .map_err(transaction_error)?;
    let result_creation_message = data_access_layer::message_dal::create_message(
        &db,
        create_message_request.deref(),
        &creation_datetime,
    );
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
                creation_datetime: creation_datetime,
            });
            Ok(utilities::responses::response_ok_with_message(
                None::<()>,
                "Message created".to_string(),
            ))
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
        Ok(messages) => Ok(utilities::responses::response_ok(Some(messages))),
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
        Ok(messages) => Ok(utilities::responses::response_ok(Some(messages))),
        Err(err) => Err(ServiceError::SqliteError(err)),
    }
}

// Green tick a viewed message
pub async fn green_tick_messages(
    authorized: AuthorizationUser,
    db: web::Data<AppState>,
    green_tick_messages_request: web::Json<GreenTickMessagesRequest>,
    server: web::Data<Addr<Server>>,
) -> actixResult<HttpResponse, ServiceError> {
    // Verify if the message you are green ticking is from a discussion that you "own"
    // TODO
    // Green tick the message
    db.connection
        .execute("BEGIN TRANSACTION", [])
        .map_err(transaction_error)?;

    for message in &green_tick_messages_request.messages {
        match data_access_layer::message_dal::green_tick_message(&db, &message.message_id) {
            Ok(()) => {
                server.do_send(GreenTickMessage {
                    id_love_room: message.love_id,
                    id_message: message.message_id,
                });
            },
            Err(err) => return Err(ServiceError::SqliteError(err)),
            // !!!!! TODO : For every transaction, if error, unlock db by ending transaction..
        }
    }

    db.connection
        .execute("END TRANSACTION", [])
        .map_err(transaction_error)?;

    Ok(utilities::responses::response_ok(None::<()>))
}
