use crate::configs::app_state::AppState;
use crate::my_errors::sqlite_errors::map_sqlite_error;
use crate::my_errors::sqlite_errors::SqliteError;
use rusqlite::params;
use std::sync::Arc;
use uuid::Uuid;

pub fn create_feedback(
    db: &Arc<AppState>,
    feedback_message: String,
    poster_uuid: String,
    creation_datetime: &String,
) -> Result<(), SqliteError> {
    let feedback_uuid = Uuid::now_v7().to_string();
    let binding = db.connection.get().unwrap();
    binding
        .prepare_cached(
            "INSERT INTO Feedbacks (feedback_uuid, poster_uuid, feedback_message, creation_datetime) VALUES (?, ?, ?, ?)",
        )
        .map_err(map_sqlite_error)?
        .execute(params![feedback_uuid, feedback_message, poster_uuid, creation_datetime])
        .map_err(map_sqlite_error)?;

    Ok(())
}
