use crate::my_errors::service_errors::ServiceError;
use crate::my_errors::sqlite_errors::{transaction_error, SqliteError};
use crate::service_layer::auth_service::JwtClaims;
// use crate::service_layer::websocket_service::{ChatMessage, GreenTickMessage, Server};
use crate::data_access_layer::message_dal::Message;
use crate::utilities::responses::{response_ok, response_ok_with_message, ApiResponse};
use crate::{data_access_layer, AppState};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

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
    jwt_claims: JwtClaims,
    State(state): State<Arc<AppState>>,
    Json(create_message_request): Json<CreateMessageRequest>,
    // server: web::Data<Addr<Server>>,
) -> Result<(StatusCode, Json<ApiResponse<()>>), ServiceError> {
    println!("{:?}", create_message_request);
    if jwt_claims.user_id != create_message_request.poster_id {
        return Err(ServiceError::ForbiddenQuery);
    }
    if create_message_request.message.len() == 0 {
        return Err(ServiceError::SqlValueNotAccepted(
            create_message_request.message.to_string(),
            "Empty messages not accepted".to_string(),
        ));
    }
    if create_message_request.message.chars().count() > 1000 {
        // Warning : Be carefull when counting string chars(), this needs tests..
        return Err(ServiceError::SqlValueNotAccepted(
            create_message_request.message.to_string(),
            "Message content string is too long".to_string(),
        ));
    }
    match data_access_layer::lover_dal::user_in_love_relation(
        &state,
        create_message_request.poster_id,
        create_message_request.love_id,
    ) {
        Ok(_) => (),
        Err(err) => match err {
            SqliteError::NotFound => return Err(ServiceError::ForbiddenQuery), // user have not matched, cannot send message
            _ => return Err(ServiceError::UnknownServiceError),
        },
    }

    let creation_datetime = format!("{:?}", chrono::offset::Utc::now());

    let id_message = data_access_layer::message_dal::create_message(
        &state,
        &create_message_request,
        &creation_datetime,
    )?;
    println!("message {} created", id_message);

    // server.do_send(ChatMessage {
    //     id_love_room: create_message_request.love_id,
    //     id_message: id_message,
    //     message: create_message_request.message.to_string(),
    //     poster_id: create_message_request.poster_id,
    //     creation_datetime: creation_datetime,
    // });
    response_ok_with_message(None::<()>, "message created".to_string())
}

// Get messages of one "love_id" love relations
pub async fn get_love_messages(
    jwt_claims: JwtClaims,
    State(state): State<Arc<AppState>>,
    Path(love_id): Path<usize>,
) -> Result<(StatusCode, Json<ApiResponse<Vec<Message>>>), ServiceError> {
    match data_access_layer::lover_dal::user_in_love_relation(&state, jwt_claims.user_id, love_id) {
        Ok(_) => println!(
            "{} user allowed to get messages of {} love relationship",
            jwt_claims.user_id, love_id
        ),
        Err(err) => match err {
            SqliteError::NotFound => return Err(ServiceError::ForbiddenQuery),
            _ => return Err(ServiceError::UnknownServiceError),
        },
    }
    let messages_found = data_access_layer::message_dal::get_love_messages(&state, love_id)?;
    response_ok(Some(messages_found))
}

// Get all the messages of all the love relation of "user_id"
pub async fn get_lover_messages(
    jwt_claims: JwtClaims,
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<usize>,
) -> Result<(StatusCode, Json<ApiResponse<Vec<Message>>>), ServiceError> {
    if jwt_claims.user_id != user_id {
        return Err(ServiceError::ForbiddenQuery);
    }
    let messages_found = data_access_layer::message_dal::get_lover_messages(&state, user_id)?;
    response_ok(Some(messages_found))
}

// Green tick a viewed message
pub async fn green_tick_messages(
    _: JwtClaims, // todo : check if still protected when not using the variable
    State(state): State<Arc<AppState>>,
    Json(green_tick_messages_request): Json<GreenTickMessagesRequest>,
    // server: web::Data<Addr<Server>>,
) -> Result<(StatusCode, Json<ApiResponse<()>>), ServiceError> {
    // TODO
    // Verify if the message you are green ticking is from a discussion that you "own",
    // and also that it's not your own message

    // todo : see how to handle partial errors for this, maybe errors dont matter for message tick
    for message in &green_tick_messages_request.messages {
        match data_access_layer::message_dal::green_tick_message(&state, &message.message_id) {
            Ok(()) => {
                // server.do_send(GreenTickMessage {
                //     id_love_room: message.love_id,
                //     id_message: message.message_id,
                // });
            }
            Err(err) => return Err(ServiceError::SqliteError(err)),
            // !!!!! TODO : For every transaction, if error, unlock db by ending transaction..
        }
    }

    response_ok(None::<()>)
}
