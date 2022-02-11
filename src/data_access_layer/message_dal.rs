use actix_web::web;
use rusqlite::params;
use serde::{Deserialize, Serialize};

use crate::my_errors::sqlite_errors::map_sqlite_error;
use crate::my_errors::sqlite_errors::SqliteError;
use crate::service_layer::message_service::CreateMessageRequest;
use crate::{data_access_layer, AppState};

#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    pub id: u32,
    pub message: String,
    pub poster_id: u32,
    pub love_id: u32,
    // TODO : add date
}

pub fn create_message(
    db: &web::Data<AppState>,
    request: &CreateMessageRequest,
) -> Result<i64, SqliteError> {
    let mut statement = db
        .connection
        .prepare("INSERT INTO Messages (message, poster_id, love_id) VALUES (?, ?, ?)")
        .map_err(map_sqlite_error)?;
    statement
        .execute(params![request.message, request.poster_id, request.love_id])
        .map_err(map_sqlite_error)?;

    Ok(db.connection.last_insert_rowid()) // TODO : This whole function in a transaction or last id could be wrong
}

// Get messages in one love relations
pub fn get_love_messages(
    db: &web::Data<AppState>,
    love_id: u32,
) -> Result<Vec<Message>, SqliteError> {
    let mut statement = db
        .connection
        .prepare("SELECT * FROM Messages WHERE love_id = ?")
        .map_err(map_sqlite_error)?;
    let result_rows = statement
        .query_map(params![love_id], |row| {
            Ok(Message {
                id: row.get("message_id")?,
                message: row.get("message")?,
                poster_id: row.get("poster_id")?,
                love_id: row.get("love_id")?,
            })
        })
        .map_err(map_sqlite_error)?;

    let mut messages = Vec::new();
    for message in result_rows {
        messages.push(message.map_err(map_sqlite_error)?);
    }

    Ok(messages)
}

// Get all the messages of all the love relation of user_id
pub fn get_lover_messages(
    db: &web::Data<AppState>,
    user_id: u32,
) -> Result<Vec<Message>, SqliteError> {
    let mut statement = db
        .connection
        .prepare("SELECT * FROM Messages WHERE love_id IN (SELECT love_id FROM Lovers WHERE Lovers.lover1 = ?)")
        .map_err(map_sqlite_error)?;
    let result_rows1 = statement
        .query_map(params![user_id], |row| {
            Ok(Message {
                id: row.get("message_id")?,
                message: row.get("message")?,
                poster_id: row.get("poster_id")?,
                love_id: row.get("love_id")?,
            })
        })
        .map_err(map_sqlite_error)?;

    let mut statement = db
        .connection
        .prepare("SELECT * FROM Messages WHERE love_id IN (SELECT love_id FROM Lovers WHERE Lovers.lover2 = ?)")
        .map_err(map_sqlite_error)?;
    let result_rows2 = statement
        .query_map(params![user_id], |row| {
            Ok(Message {
                id: row.get("message_id")?,
                message: row.get("message")?,
                poster_id: row.get("poster_id")?,
                love_id: row.get("love_id")?,
            })
        })
        .map_err(map_sqlite_error)?;

    let mut messages = Vec::new();
    for message in result_rows1 {
        messages.push(message.map_err(map_sqlite_error)?);
    }
    for message in result_rows2 {
        messages.push(message.map_err(map_sqlite_error)?);
    }
    println!("{:?}", messages);

    Ok(messages)
}
