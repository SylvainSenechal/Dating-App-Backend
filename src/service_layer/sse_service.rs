use crate::{data_access_layer, AppState};
use axum::Json;
use axum::{
    extract::Query,
    extract::{Path, State},
    http::{HeaderValue, Method},
    response::sse::{Event, Sse},
    routing::get,
    Router,
};
use futures::stream::{self, Stream};
use r2d2::Pool;
use rusqlite::Connection;
use serde::Deserialize;
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::Mutex;
use std::{convert::Infallible, path::PathBuf, time::Duration};
use tokio::sync::broadcast;
use tower_http::cors::Any;
use tower_http::cors::CorsLayer;
use tower_http::{services::ServeDir, trace::TraceLayer};

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

pub async fn server_side_event_handler(
    State(state): State<Arc<AppState>>,
    Path(user_private_uuid): Path<String>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    struct Guard {
        // whatever state you need here
    }

    impl Drop for Guard {
        fn drop(&mut self) {
            println!("stream closed");
        }
    }

    // todo : return error
    let user_uuid =
        data_access_layer::user_dal::User::get_user_uuid_by_private_uuid(&state, user_private_uuid)
            .unwrap();

    let (tx, mut red) = broadcast::channel::<SseMessage>(1);
    state.txs.lock().unwrap().insert(user_uuid, tx);

    let stream = async_stream::stream! {
        // let _guard = Guard {};
        while let Ok(msg) = red.recv().await {
            println!("sending");
            yield Ok(Event::default().event("update").json_data(msg).unwrap())
        }
    };

    Sse::new(stream)
}
