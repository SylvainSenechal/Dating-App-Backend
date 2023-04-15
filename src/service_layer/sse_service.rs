use crate::{data_access_layer, AppState};
use axum::Json;
use axum::{
    extract::Query,
    extract::State,
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

#[derive(Serialize, Clone, Debug)]
pub enum SseMessage {
    ChatMessage {
        uuid_love_room: String,
        uuid_message: String,
        message: String,
        poster_uuid: String,
        creation_datetime: String,
    },
}

pub async fn server_side_event_handler(
    State(state): State<Arc<AppState>>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    struct Guard {
        // whatever state you need here
    }

    impl Drop for Guard {
        fn drop(&mut self) {
            println!("stream closed");
        }
    }

    let (tx, mut red) = broadcast::channel::<SseMessage>(1);
    // state.txs.lock().unwrap().insert(pagination.id, tx);
    state.txs.lock().unwrap().insert(0, tx);

    let stream = async_stream::stream! {
        // let _guard = Guard {};
        // let mut rx = state.tx.subscribe();
        while let Ok(msg) = red.recv().await {
            println!("sending");
            // yield Ok(Event::default().data(msg))
            // yield Ok(Event::default().event("update").data(msg))
            yield Ok(Event::default().event("update").json_data(msg).unwrap())
        }
    };

    Sse::new(stream)
}
