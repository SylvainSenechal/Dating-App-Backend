use crate::my_errors::sqlite_errors::map_sqlite_error;
use crate::my_errors::sqlite_errors::SqliteError;
use crate::AppState;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Serialize, Deserialize, Debug)]
pub struct Photo {
    pub photo_id: usize,
    pub photo_uuid: String,
    pub user_uuid: String,
    pub url: String,
    pub display_order: usize,
}

pub fn create_user_photo(
    db: &Arc<AppState>,
    photo_uuid: String,
    user_uuid: String,
    url: String,
    display_order: usize,
) -> Result<(), SqliteError> {
    let binding = db.connection.get().unwrap();
    binding
        .prepare_cached(
            "INSERT INTO Photos (photo_uuid, user_uuid, url, display_order) VALUES (?, ?, ?, ?)",
        )
        .map_err(map_sqlite_error)?
        .execute(params![photo_uuid, user_uuid, url, display_order])
        .map_err(map_sqlite_error)?;

    Ok(())
}

pub fn delete_photo(db: &Arc<AppState>, photo_uuid: String) -> Result<(), SqliteError> {
    let binding = db.connection.get().unwrap();
    binding
        .prepare_cached("DELETE FROM Photos WHERE photo_uuid = ?")
        .map_err(map_sqlite_error)?
        .execute(params![photo_uuid])
        .map_err(map_sqlite_error)?;
    Ok(())
}

pub fn get_user_photos(db: &Arc<AppState>, user_uuid: String) -> Result<Vec<Photo>, SqliteError> {
    let binding = db.connection.get().unwrap();
    let mut statement = binding
        .prepare_cached("SELECT * FROM Photos WHERE user_uuid = ?")
        .map_err(map_sqlite_error)?;
    let result_rows = statement
        .query_map(params![user_uuid], |row| {
            Ok(Photo {
                photo_id: row.get("photo_id")?,
                photo_uuid: row.get("photo_uuid")?,
                user_uuid: row.get("user_uuid")?,
                url: row.get("url")?,
                display_order: row.get("display_order")?,
            })
        })
        .map_err(map_sqlite_error)?;

    let mut photos = Vec::new();
    for photo in result_rows {
        photos.push(photo.map_err(map_sqlite_error)?)
    }

    Ok(photos)
}

pub fn shift_order_photos(
    db: &Arc<AppState>,
    user_uuid: String,
    order_shift: usize,
) -> Result<(), SqliteError> {
    let binding = db.connection.get().unwrap();
    let mut statement = binding
        .prepare_cached(
            "
        UPDATE Photos
        SET display_order = display_order -1
        WHERE user_uuid = ?
        AND display_order > ?
        ",
        )
        .map_err(map_sqlite_error)?;

    statement
        .execute(params![user_uuid, order_shift])
        .map_err(map_sqlite_error)?;

    Ok(())
}

pub fn switch_order_photos(
    db: &Arc<AppState>,
    order1: usize,
    order2: usize,
    photo_uuid1: String,
    photo_uuid2: String,
) -> Result<(), SqliteError> {
    let mut binding = db.connection.get().unwrap();
    let tx = binding.transaction().map_err(map_sqlite_error)?;
    tx.prepare_cached(
        "
        UPDATE Photos
        SET display_order = ?
        WHERE photo_uuid = ?
        ",
    )
    .map_err(map_sqlite_error)?
    .execute(params![order1, photo_uuid2])
    .map_err(map_sqlite_error)?;

    tx.prepare_cached(
        "
        UPDATE Photos
        SET display_order = ?
        WHERE photo_uuid = ?
        ",
    )
    .map_err(map_sqlite_error)?
    .execute(params![order2, photo_uuid1])
    .map_err(map_sqlite_error)?;

    tx.commit().map_err(map_sqlite_error)?;

    Ok(())
}
