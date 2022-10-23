use actix_web::web;
use rusqlite::params;
use serde::{Deserialize, Serialize};

use crate::my_errors::sqlite_errors::map_sqlite_error;
use crate::my_errors::sqlite_errors::SqliteError;
use crate::AppState;

#[derive(Serialize, Deserialize, Debug)]
pub struct Lover {
    // same as user but with a love_id, and seen_by_lover1/2
    pub love_id: usize,
    pub lover1: usize,
    pub lover2: usize,
    pub seen_by_lover1: u8, // bool actually
    pub seen_by_lover2: u8, // bool actually
    pub id: usize,
    pub name: String,
    pub password: String,
    pub email: String,
    pub age: u8,
    pub latitude: f32,
    pub longitude: f32,
    pub gender: String,
    pub looking_for: String,
    pub search_radius: u16,
    pub looking_for_age_min: u8,
    pub looking_for_age_max: u8,
    pub description: String,
}

#[allow(dead_code)]
pub struct LoveRelation {
    pub love_id: usize,
    pub lover1: usize,
    pub lover2: usize,
}

pub fn create_lovers(
    db: &web::Data<AppState>,
    lover1: usize,
    lover2: usize,
) -> Result<(), SqliteError> {
    let mut statement = db
        .connection
        .prepare_cached("INSERT INTO Lovers (lover1, lover2) VALUES (?, ?)")
        .map_err(map_sqlite_error)?;
    statement
        .execute(params![lover1, lover2])
        .map_err(map_sqlite_error)?;

    Ok(())
}

#[allow(dead_code)]
// Get all the lovers of the user_id (user_id is exluded from result), a lover is a user, with an added love_id field
pub fn get_love_relation(
    db: &web::Data<AppState>,
    love_id: usize,
) -> Result<LoveRelation, SqliteError> {
    let mut statement = db
        .connection
        .prepare_cached(
            "
            SELECT * FROM Lovers WHERE love_id = ?
            ",
        )
        .map_err(map_sqlite_error)?;
    statement
        .query_row(params![love_id], |row| {
            Ok(LoveRelation {
                love_id: row.get("love_id")?,
                lover1: row.get("lover1")?,
                lover2: row.get("lovee2")?,
            })
        })
        .map_err(map_sqlite_error)
}

// Return true if user_id is in the loved_id relation
pub fn user_in_love_relation(
    db: &web::Data<AppState>,
    user_id: usize,
    love_id: usize,
) -> Result<(), SqliteError> {
    let mut statement = db
        .connection
        .prepare_cached(
            "
        SELECT * FROM Lovers WHERE love_id = ? AND (lover1 = ? OR lover2 = ?)
        ",
        )
        .map_err(map_sqlite_error)?;

    statement
        .query_row(params![love_id, user_id, user_id], |_| Ok(()))
        .map_err(map_sqlite_error)
}

// Get all the lovers of the user_id (user_id is exluded from result), a lover is a user, with an added love_id field
pub fn get_lovers(db: &web::Data<AppState>, user_id: usize) -> Result<Vec<Lover>, SqliteError> {
    let mut statement = db
        .connection
        .prepare_cached(
            "
            SELECT * FROM Users JOIN Lovers ON Users.user_id = Lovers.lover1 WHERE Lovers.lover2 = ?
            ",
        )
        .map_err(map_sqlite_error)?;
    let result_rows1 = statement
        .query_map(params![user_id], |row| {
            Ok(Lover {
                love_id: row.get("love_id")?,
                lover1: row.get("lover1")?,
                lover2: row.get("lover2")?,
                seen_by_lover1: row.get("seen_by_lover1")?,
                seen_by_lover2: row.get("seen_by_lover2")?,
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
        .prepare_cached(
            "
            SELECT * FROM Users JOIN Lovers ON Users.user_id = Lovers.lover2 WHERE Lovers.lover1 = ?
            ",
        )
        .map_err(map_sqlite_error)?;
    let result_rows2 = statement
        .query_map(params![user_id], |row| {
            Ok(Lover {
                love_id: row.get("love_id")?,
                lover1: row.get("lover1")?,
                lover2: row.get("lover2")?,
                seen_by_lover1: row.get("seen_by_lover1")?,
                seen_by_lover2: row.get("seen_by_lover2")?,
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

pub fn tick_love(db: &web::Data<AppState>, love_id: usize, lover_id: usize) -> Result<(), SqliteError> {
    let mut statement = db
        .connection
        .prepare_cached(
            "
            UPDATE Lovers
            SET 
                seen_by_lover1 = CASE WHEN lover1 = ? THEN 1 ELSE 0 END,
                seen_by_lover2 = CASE WHEN lover2 = ? THEN 1 ELSE 0 END
            WHERE love_id = ?
        ",
        )
        .map_err(map_sqlite_error)?;
    statement
        .execute(params![lover_id, lover_id, love_id])
        .map_err(map_sqlite_error)?;

    Ok(())
}


// UPDATE Lovers
// SET 
//     seen_by_lover1 = CASE WHEM lover1 = 2 THEN 1 ELSE 0 END,
//     seen_by_lover2 = CASE WHEM lover2 = 2 THEN 1 ELSE 0 END
// WHERE love_id = 1