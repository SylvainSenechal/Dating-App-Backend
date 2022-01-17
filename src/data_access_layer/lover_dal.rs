use actix_web::web;
use rusqlite::params;

use crate::data_access_layer::user_dal::User;
use crate::my_errors::sqlite_errors::map_sqlite_error;
use crate::my_errors::sqlite_errors::SqliteError;
use crate::AppState;

pub fn create_lovers(
    db: &web::Data<AppState>,
    lover1: u32,
    lover2: u32,
) -> Result<(), SqliteError> {
    let mut statement = db
        .connection
        .prepare("INSERT INTO Lovers (lover1, lover2) VALUES (?, ?)")
        .map_err(map_sqlite_error)?;
    statement
        .execute(params![lover1, lover2])
        .map_err(map_sqlite_error)?;

    Ok(())
}

pub fn get_lovers(db: &web::Data<AppState>, user_id: u32) -> Result<Vec<User>, SqliteError> {
    // TODO : do this in a transaction or use a Union..
    let mut statement = db
        .connection
        .prepare(
            "
            SELECT * FROM Users JOIN Lovers ON Users.user_id = Lovers.lover1 WHERE Lovers.lover2 = ?
            ",
        )
        .map_err(map_sqlite_error)?;
    let result_rows1 = statement
        .query_map(params![user_id], |row| {
            Ok(User {
                id: row.get("user_id")?,
                name: row.get("name")?,
                password: row.get("password")?,
                email: row.get("email")?,
                age: row.get("age")?,
                latitude: row.get("latitude")?,
                longitude: row.get("longitude")?,
                gender: row.get("gender")?,
                looking_for: row.get("looking_for")?,
                search_radius: row.get("search_radius")?,
                looking_for_age_min: row.get("looking_for_age_min")?,
                looking_for_age_max: row.get("looking_for_age_max")?,
                description: row.get("description")?,
            })
        })
        .map_err(map_sqlite_error)?;

    let mut statement = db
        .connection
        .prepare(
            "
            SELECT * FROM Users JOIN Lovers ON Users.user_id = Lovers.lover2 WHERE Lovers.lover1 = ?
            ",
        )
        .map_err(map_sqlite_error)?;
    let result_rows2 = statement
        .query_map(params![user_id], |row| {
            Ok(User {
                id: row.get("user_id")?,
                name: row.get("name")?,
                password: row.get("password")?,
                email: row.get("email")?,
                age: row.get("age")?,
                latitude: row.get("latitude")?,
                longitude: row.get("longitude")?,
                gender: row.get("gender")?,
                looking_for: row.get("looking_for")?,
                search_radius: row.get("search_radius")?,
                looking_for_age_min: row.get("looking_for_age_min")?,
                looking_for_age_max: row.get("looking_for_age_max")?,
                description: row.get("description")?,
            })
        })
        .map_err(map_sqlite_error)?;

    let mut persons = Vec::new();
    for person in result_rows1 {
        persons.push(person.map_err(map_sqlite_error)?);
    }
    for person in result_rows2 {
        persons.push(person.map_err(map_sqlite_error)?);
    }

    Ok(persons)
}
