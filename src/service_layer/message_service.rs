use crate::data_access_layer::message_dal::Message;
use crate::my_errors::service_errors::ServiceError;
use crate::my_errors::sqlite_errors::SqliteError;
use crate::service_layer::auth_service::JwtClaims;
use crate::service_layer::sse_service::{MessageData, SseMessage, SseMessageType};
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
    pub poster_uuid: String,
    pub love_uuid: String,
}

#[derive(Deserialize)]
pub struct GreenTickMessagesRequest {
    pub love_uuid: String,
    pub lover_ticked_uuid: String,
}

// Post a message by poster_id in the love_id relation
pub async fn create_message(
    jwt_claims: JwtClaims,
    State(state): State<Arc<AppState>>,
    Json(create_message_request): Json<CreateMessageRequest>,
) -> Result<(StatusCode, Json<ApiResponse<()>>), ServiceError> {
    println!("{:?}", create_message_request);
    if jwt_claims.user_uuid != create_message_request.poster_uuid {
        return Err(ServiceError::ForbiddenQuery);
    }
    if create_message_request.message.is_empty() {
        return Err(ServiceError::ValueNotAccepted(
            create_message_request.message,
            "Empty messages not accepted".to_string(),
        ));
    }
    if create_message_request.message.chars().count() > 1000 {
        return Err(ServiceError::ValueNotAccepted(
            create_message_request.message,
            "Message content string is too long".to_string(),
        ));
    }
    match data_access_layer::lover_dal::user_in_love_relation(
        &state,
        create_message_request.poster_uuid.clone(),
        create_message_request.love_uuid.clone(),
    ) {
        Ok(_) => (),
        Err(err) => match err {
            SqliteError::NotFound => return Err(ServiceError::ForbiddenQuery), // user have not matched, cannot send message
            _ => return Err(ServiceError::UnknownServiceError),
        },
    }

    let creation_datetime = format!("{:?}", chrono::offset::Utc::now());

    let uuid_message = data_access_layer::message_dal::create_message(
        &state,
        &create_message_request,
        &creation_datetime,
    )?;
    println!("message {} created", uuid_message);

    let message = SseMessage {
        message_type: SseMessageType::ChatMessage,
        data: MessageData::ChatMessage {
            uuid_love_room: create_message_request.love_uuid,
            uuid_message: uuid_message.clone(),
            message: create_message_request.message.to_string(),
            poster_uuid: create_message_request.poster_uuid,
            creation_datetime: creation_datetime,
        },
    };

    let (uuid1, uuid2) =
        data_access_layer::message_dal::get_lovers_uuids_from_message_uuid(&state, uuid_message)?;

    let rooms = state.txs.lock().unwrap();
    if let Some(sender) = rooms.get(&uuid1) {
        match sender.send(message.clone()) {
            Ok(_) => (),
            Err(e) => println!("send sse message failed : {}", e),
        }
    }
    if let Some(sender) = rooms.get(&uuid2) {
        match sender.send(message) {
            Ok(_) => (),
            Err(e) => println!("send sse message failed : {}", e),
        }
    }

    response_ok_with_message(None::<()>, "message created".to_string())
}

// Get messages of one "love_uuid" love relations
pub async fn get_love_messages(
    jwt_claims: JwtClaims,
    State(state): State<Arc<AppState>>,
    Path(love_uuid): Path<String>,
) -> Result<(StatusCode, Json<ApiResponse<Vec<Message>>>), ServiceError> {
    match data_access_layer::lover_dal::user_in_love_relation(
        &state,
        jwt_claims.user_uuid.clone(),
        love_uuid.clone(),
    ) {
        Ok(_) => println!(
            "{} user allowed to get messages of {} love relationship",
            jwt_claims.user_uuid, love_uuid
        ),
        Err(err) => match err {
            SqliteError::NotFound => return Err(ServiceError::ForbiddenQuery),
            _ => return Err(ServiceError::UnknownServiceError),
        },
    }
    let messages_found = data_access_layer::message_dal::get_love_messages(&state, love_uuid)?;
    response_ok(Some(messages_found))
}

// Get all the messages of all the love relation of "user_uuid"
pub async fn get_lover_messages(
    jwt_claims: JwtClaims,
    State(state): State<Arc<AppState>>,
    Path(user_uuid): Path<String>,
) -> Result<(StatusCode, Json<ApiResponse<Vec<Message>>>), ServiceError> {
    if jwt_claims.user_uuid != user_uuid {
        return Err(ServiceError::ForbiddenQuery);
    }
    let messages_found = data_access_layer::message_dal::get_lover_messages(&state, user_uuid)?;
    response_ok(Some(messages_found))
}

// Green tick a viewed message
pub async fn green_tick_messages(
    jwt_claims: JwtClaims,
    State(state): State<Arc<AppState>>,
    Json(green_tick_messages_request): Json<GreenTickMessagesRequest>,
) -> Result<(StatusCode, Json<ApiResponse<()>>), ServiceError> {
    data_access_layer::message_dal::green_tick_messages(
        &state,
        green_tick_messages_request.love_uuid.clone(),
        green_tick_messages_request.lover_ticked_uuid.clone(),
        jwt_claims.user_uuid,
    )?;

    let message = SseMessage {
        message_type: SseMessageType::GreenTickMessage,
        data: MessageData::GreenTickMessage {
            uuid_love_room: green_tick_messages_request.love_uuid,
        },
    };

    let rooms = state.txs.lock().unwrap();
    if let Some(sender) = rooms.get(&green_tick_messages_request.lover_ticked_uuid) {
        println!("Sending a sse message : {:?} ", message.data);
        match sender.send(message) {
            Ok(_) => (),
            Err(e) => println!("send sse message failed : {}", e),
        }
    }

    response_ok(None::<()>)
}
