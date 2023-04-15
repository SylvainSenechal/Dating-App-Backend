use crate::my_errors::service_errors::ServiceError;
use crate::my_errors::sqlite_errors::SqliteError;
use crate::service_layer::auth_service::JwtClaims;
// use crate::service_layer::websocket_service::{ChatMessage, GreenTickMessage, Server};
use crate::data_access_layer::message_dal::Message;
use crate::service_layer::sse_service::SseMessage;
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
    pub messages: Vec<GreenTickMessageRequest>,
}

#[derive(Deserialize)]
pub struct GreenTickMessageRequest {
    pub message_uuid: String,
    pub love_uuid: String,
}

// Post a message by poster_id in the love_id relation
pub async fn create_message(
    jwt_claims: JwtClaims,
    State(state): State<Arc<AppState>>,
    Json(create_message_request): Json<CreateMessageRequest>,
    // server: web::Data<Addr<Server>>,
) -> Result<(StatusCode, Json<ApiResponse<()>>), ServiceError> {
    println!("{:?}", create_message_request);
    if jwt_claims.user_uuid != create_message_request.poster_uuid {
        return Err(ServiceError::ForbiddenQuery);
    }
    if create_message_request.message.is_empty() {
        return Err(ServiceError::SqlValueNotAccepted(
            create_message_request.message,
            "Empty messages not accepted".to_string(),
        ));
    }
    if create_message_request.message.chars().count() > 1000 {
        // Warning : Be carefull when counting string chars(), this needs tests..
        return Err(ServiceError::SqlValueNotAccepted(
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

    let message = SseMessage::ChatMessage {
        uuid_love_room: create_message_request.love_uuid,
        uuid_message: uuid_message,
        message: create_message_request.message.to_string(),
        poster_uuid: create_message_request.poster_uuid,
        creation_datetime: creation_datetime,
    };

    // todo : handle result ?
    state.txs.lock().unwrap().get(&0).unwrap().send(message);
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
    _: JwtClaims,
    State(state): State<Arc<AppState>>,
    Json(green_tick_messages_request): Json<GreenTickMessagesRequest>,
    // server: web::Data<Addr<Server>>,
) -> Result<(StatusCode, Json<ApiResponse<()>>), ServiceError> {
    // TODO
    // Verify if the message you are green ticking is from a discussion that you "own",
    // and also that it's not your own message

    // todo : see how to handle partial errors for this, maybe errors dont matter for message tick
    for message in &green_tick_messages_request.messages {
        match data_access_layer::message_dal::green_tick_message(
            &state,
            message.message_uuid.clone(),
        ) {
            Ok(()) => {
                // server.do_send(GreenTickMessage {
                //     id_love_room: message.love_id,
                //     id_message: message.message_id,
                // });

                // let oo = state
                // .txs
                // .lock()
                // .unwrap()
                // .get(&0)
                // .unwrap()
                // .send("proute".to_owned()); // .send("proute".to_owned());
                // println!("SSSS {:?}", oo);
            }
            Err(err) => return Err(ServiceError::SqliteError(err)),
            // !!!!! TODO : For every transaction, if error, unlock db by ending transaction..
        }
    }

    response_ok(None::<()>)
}
