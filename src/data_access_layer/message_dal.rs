use actix_web::web;
use rusqlite::params;
use serde::{Deserialize, Serialize};

use crate::my_errors::sqlite_errors::map_sqlite_error;
use crate::my_errors::sqlite_errors::SqliteError;
use crate::service_layer::message_service::CreateMessageRequest;
use crate::AppState;

#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    pub id: usize,
    pub message: String,
    pub poster_id: usize,
    pub love_id: usize,
    pub creation_datetime: String
}

pub fn create_message(
    db: &web::Data<AppState>,
    request: &CreateMessageRequest,
    creation_datetime: &String,
) -> Result<usize, SqliteError> {
    let mut statement = db
        .connection
        .prepare_cached("INSERT INTO Messages (message, poster_id, love_id, creation_datetime) VALUES (?, ?, ?, ?)")
        .map_err(map_sqlite_error)?;
    statement
        .execute(params![request.message, request.poster_id, request.love_id, creation_datetime])
        .map_err(map_sqlite_error)?;

    let id_inserted: usize = db.connection.last_insert_rowid() as usize;

    Ok(id_inserted)
}

// Get messages in one love relations
pub fn get_love_messages(
    db: &web::Data<AppState>,
    love_id: usize,
) -> Result<Vec<Message>, SqliteError> {
    let mut statement = db
        .connection
        .prepare_cached("SELECT * FROM Messages WHERE love_id = ?")
        .map_err(map_sqlite_error)?;
    let result_rows = statement
        .query_map(params![love_id], |row| {
            Ok(Message {
                id: row.get("message_id")?,
                message: row.get("message")?,
                poster_id: row.get("poster_id")?,
                love_id: row.get("love_id")?,
                creation_datetime: row.get("creation_datetime")?,
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
    user_id: usize,
) -> Result<Vec<Message>, SqliteError> {
    let mut statement = db
        .connection
        .prepare_cached("SELECT * FROM Messages WHERE love_id IN (SELECT love_id FROM Lovers WHERE Lovers.lover1 = ?)")
        .map_err(map_sqlite_error)?;
    let result_rows1 = statement
        .query_map(params![user_id], |row| {
            Ok(Message {
                id: row.get("message_id")?,
                message: row.get("message")?,
                poster_id: row.get("poster_id")?,
                love_id: row.get("love_id")?,
                creation_datetime: row.get("creation_datetime")?,
            })
        })
        .map_err(map_sqlite_error)?;

    let mut statement = db
        .connection
        .prepare_cached("SELECT * FROM Messages WHERE love_id IN (SELECT love_id FROM Lovers WHERE Lovers.lover2 = ?)")
        .map_err(map_sqlite_error)?;
    let result_rows2 = statement
        .query_map(params![user_id], |row| {
            Ok(Message {
                id: row.get("message_id")?,
                message: row.get("message")?,
                poster_id: row.get("poster_id")?,
                love_id: row.get("love_id")?,
                creation_datetime: row.get("creation_datetime")?,
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

    Ok(messages)
}
