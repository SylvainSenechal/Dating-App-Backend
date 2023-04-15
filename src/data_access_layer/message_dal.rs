use rusqlite::params;
use serde::{Deserialize, Serialize};

use crate::my_errors::sqlite_errors::map_sqlite_error;
use crate::my_errors::sqlite_errors::SqliteError;
use crate::service_layer::message_service::CreateMessageRequest;
use crate::AppState;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    pub uuid: String,
    pub message: String,
    pub poster_uuid: String,
    pub love_uuid: String,
    pub seen: u8,
    pub creation_datetime: String,
}

pub fn create_message(
    db: &Arc<AppState>,
    request: &CreateMessageRequest,
    creation_datetime: &String,
) -> Result<String, SqliteError> {
    let uuid_message = Uuid::now_v7().to_string();
    let binding = db.connection.get().unwrap();
    binding
        .prepare_cached("INSERT INTO Messages (message_uuid, message, poster_uuid, love_uuid, seen, creation_datetime) VALUES (?, ?, ?, ?, ?, ?)")
        .map_err(map_sqlite_error)?
        .execute(params![
            uuid_message,
            request.message,
            request.poster_uuid,
            request.love_uuid,
            0, // message is not seen
            creation_datetime
        ])
        .map_err(map_sqlite_error)?;

    Ok(uuid_message)
}

// pub fn create_message(
//     db: &Arc<AppState>,
//     request: &CreateMessageRequest,
//     creation_datetime: &String,
// ) -> Result<usize, SqliteError> {
//     let mut binding = db.connection.get().unwrap();
//     let tx = binding.transaction().map_err(map_sqlite_error)?;
//     tx
//         .prepare_cached("INSERT INTO Messages (message_uuid, message, poster_id, love_id, seen, creation_datetime) VALUES (?, ?, ?, ?, ?, ?)")
//         .map_err(map_sqlite_error)?
//         .execute(params![
//             Uuid::now_v7().to_string(),
//             request.message,
//             request.poster_uuid,
//             request.love_uuid,
//             0, // message is not seen
//             creation_datetime
//         ])
//         .map_err(map_sqlite_error)?;

//     let id_inserted: usize = tx.last_insert_rowid() as usize;
//     tx.commit().map_err(map_sqlite_error)?;

//     Ok(id_inserted)
// }

// Get messages in one love relations
pub fn get_love_messages(
    db: &Arc<AppState>,
    love_uuid: String,
) -> Result<Vec<Message>, SqliteError> {
    let binding = db.connection.get().unwrap();
    let mut statement = binding
        .prepare_cached("SELECT * FROM Messages WHERE love_uuid = ?")
        .map_err(map_sqlite_error)?;
    let result_rows = statement
        .query_map(params![love_uuid], |row| {
            Ok(Message {
                uuid: row.get("message_uuid")?,
                message: row.get("message")?,
                poster_uuid: row.get("poster_uuid")?,
                love_uuid: row.get("love_uuid")?,
                seen: row.get("seen")?,
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

// Get all the messages of all the love relation of user_uuid
pub fn get_lover_messages(
    db: &Arc<AppState>,
    user_uuid: String,
) -> Result<Vec<Message>, SqliteError> {
    let binding = db.connection.get().unwrap();
    let mut statement = binding
        .prepare_cached("SELECT * FROM Messages WHERE love_uuid IN (SELECT love_uuid FROM Lovers WHERE Lovers.lover1 = ?)")
        .map_err(map_sqlite_error)?;
    let result_rows1 = statement
        .query_map(params![user_uuid], |row| {
            Ok(Message {
                uuid: row.get("message_uuid")?,
                message: row.get("message")?,
                poster_uuid: row.get("poster_uuid")?,
                love_uuid: row.get("love_uuid")?,
                seen: row.get("seen")?,
                creation_datetime: row.get("creation_datetime")?,
            })
        })
        .map_err(map_sqlite_error)?;

    let binding = db.connection.get().unwrap();
    let mut statement = binding
        .prepare_cached("SELECT * FROM Messages WHERE love_uuid IN (SELECT love_uuid FROM Lovers WHERE Lovers.lover2 = ?)")
        .map_err(map_sqlite_error)?;
    let result_rows2 = statement
        .query_map(params![user_uuid], |row| {
            Ok(Message {
                uuid: row.get("message_uuid")?,
                message: row.get("message")?,
                poster_uuid: row.get("poster_uuid")?,
                love_uuid: row.get("love_uuid")?,
                seen: row.get("seen")?,
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

pub fn green_tick_message(db: &Arc<AppState>, message_uuid: String) -> Result<(), SqliteError> {
    let binding = db.connection.get().unwrap();
    let mut statement = binding
        .prepare_cached("UPDATE Messages SET seen = ? WHERE message_uuid = ?")
        .map_err(map_sqlite_error)?;

    statement
        .execute(params![1, message_uuid])
        .map_err(map_sqlite_error)?;
    Ok(())
}
