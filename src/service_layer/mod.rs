use serde::{Deserialize, Serialize};

pub mod user_service;
pub mod auth_service;
pub mod photos_service;
pub mod lover_service;
pub mod statistics_service;
pub mod message_service;
pub mod websocket_service;

#[derive(Serialize, Deserialize, Debug, Default)]
struct MessageServiceResponse {
    message: String,
}