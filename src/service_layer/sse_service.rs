use crate::configs::app_state::AppState;
use crate::data_access_layer;
use axum::{
    extract::{Path, State},
    response::sse::{Event, Sse},
};
use futures::stream::Stream;
use serde::Serialize;
use std::sync::{Arc, Mutex};
use std::{collections::HashMap, convert::Infallible};
use tokio::sync::broadcast;

#[derive(Serialize, Clone)]
pub struct SseMessage {
    pub message_type: SseMessageType,
    pub data: MessageData,
}

#[derive(Serialize, Clone)]
pub enum SseMessageType {
    ChatMessage,
    GreenTickMessage,
}

#[derive(Serialize, Clone, Debug)]
pub enum MessageData {
    ChatMessage {
        uuid_love_room: String,
        uuid_message: String,
        message: String,
        poster_uuid: String,
        creation_datetime: String,
    },
    GreenTickMessage {
        uuid_love_room: String,
    },
}

struct Guard<'a> {
    channels: &'a Mutex<HashMap<String, broadcast::Sender<SseMessage>>>,
    user_uuid: String,
}

impl Drop for Guard<'_> {
    fn drop(&mut self) {
        self.channels.lock().unwrap().remove(&self.user_uuid);
    }
}

pub async fn server_side_event_handler(
    State(state): State<Arc<AppState>>,
    Path(user_private_uuid): Path<String>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let user_uuid =
        data_access_layer::user_dal::get_user_uuid_by_private_uuid(&state, user_private_uuid)
            .unwrap();

    let (tx, mut red) = broadcast::channel::<SseMessage>(1);
    state.txs.lock().unwrap().insert(user_uuid.clone(), tx);
    let stream = async_stream::stream! {
        let _guard = Guard {
            channels: &state.txs,
            user_uuid: user_uuid
        };

        while let Ok(msg) = red.recv().await {
            yield Ok(Event::default().event("update").json_data(msg).unwrap())
        }
    };

    Sse::new(stream)
}
