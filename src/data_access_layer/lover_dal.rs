use rusqlite::params;
use serde::{Deserialize, Serialize};

use crate::my_errors::sqlite_errors::map_sqlite_error;
use crate::my_errors::sqlite_errors::SqliteError;
use crate::requests::requests::Gender;
use crate::AppState;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct LoveWithLover {
    pub love_uuid: String,
    pub lover1: String,
    pub lover2: String,
    pub seen_by_lover1: u8, // bool actually
    pub seen_by_lover2: u8, // bool actually
    pub lover_uuid: String,
    pub last_seen: String,
    pub name: String,
    pub age: u8,
    pub gender: String,
    pub description: String,
}

pub fn create_lovers(
    db: &Arc<AppState>,
    lover1: String,
    lover2: String,
) -> Result<(), SqliteError> {
    let binding = db.connection.get().unwrap();
    let mut statement = binding
        .prepare_cached("INSERT INTO Lovers (love_uuid, lover1, lover2) VALUES (?, ?, ?)")
        .map_err(map_sqlite_error)?;
    statement
        .execute(params![Uuid::now_v7().to_string(), lover1, lover2])
        .map_err(map_sqlite_error)?;

    Ok(())
}

// Return true if user_uuid is in the loved_id relation
pub fn user_in_love_relation(
    db: &Arc<AppState>,
    user_uuid: String,
    love_uuid: String,
) -> Result<(), SqliteError> {
    let binding = db.connection.get().unwrap();
    let mut statement = binding
        .prepare_cached(
            "
        SELECT * FROM Lovers WHERE love_uuid = ? AND (lover1 = ? OR lover2 = ?)
        ",
        )
        .map_err(map_sqlite_error)?;

    statement
        .query_row(params![love_uuid, user_uuid, user_uuid], |_| Ok(()))
        .map_err(map_sqlite_error)
}

// Get all the lovers of the user_uuid (user_uuid is exluded from result)
pub fn get_lovers(
    db: &Arc<AppState>,
    user_uuid: String,
) -> Result<Vec<LoveWithLover>, SqliteError> {
    let binding = db.connection.get().unwrap();
    let mut statement = binding
        .prepare_cached(
            "
            SELECT * FROM Users JOIN Lovers ON Users.user_uuid = Lovers.lover1 WHERE Lovers.lover2 = ?
            ",
        )
        .map_err(map_sqlite_error)?;
    let result_rows1 = statement
        .query_map(params![user_uuid], |row| {
            Ok(LoveWithLover {
                love_uuid: row.get("love_uuid")?,
                lover1: row.get("lover1")?,
                lover2: row.get("lover2")?,
                seen_by_lover1: row.get("seen_by_lover1")?,
                seen_by_lover2: row.get("seen_by_lover2")?,
                lover_uuid: row.get("user_uuid")?,
                name: row.get("name")?,
                last_seen: row.get("last_seen")?,
                age: row.get("age")?,
                gender: row.get("gender")?,
                description: row.get("description")?,
            })
        })
        .map_err(map_sqlite_error)?;

    let binding = db.connection.get().unwrap();
    let mut statement = binding
        .prepare_cached(
            "
            SELECT * FROM Users JOIN Lovers ON Users.user_uuid = Lovers.lover2 WHERE Lovers.lover1 = ?
            ",
        )
        .map_err(map_sqlite_error)?;
    let result_rows2 = statement
        .query_map(params![user_uuid], |row| {
            Ok(LoveWithLover {
                love_uuid: row.get("love_uuid")?,
                lover1: row.get("lover1")?,
                lover2: row.get("lover2")?,
                seen_by_lover1: row.get("seen_by_lover1")?,
                seen_by_lover2: row.get("seen_by_lover2")?,
                lover_uuid: row.get("user_uuid")?,
                name: row.get("name")?,
                last_seen: row.get("last_seen")?,
                age: row.get("age")?,
                gender: row.get("gender")?,
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

pub fn tick_love(
    db: &Arc<AppState>,
    love_uuid: String,
    lover_uuid: String,
) -> Result<(), SqliteError> {
    let binding = db.connection.get().unwrap();
    let mut statement = binding
        .prepare_cached(
            "
            UPDATE Lovers
            SET
                seen_by_lover1 = CASE WHEN lover1 = ? THEN 1 ELSE seen_by_lover1 END,
                seen_by_lover2 = CASE WHEN lover2 = ? THEN 1 ELSE seen_by_lover2 END
            WHERE love_uuid = ?
        ",
        )
        .map_err(map_sqlite_error)?;
    statement
        .execute(params![lover_uuid, lover_uuid, love_uuid])
        .map_err(map_sqlite_error)?;

    Ok(())
}

pub fn potential_matches_count(
    db: &Arc<AppState>,
    user_uuid: String,
    looking_for: Gender,
    search_radius: u16,
    latitude: f32,
    longitude: f32,
    age_min: u8,
    age_max: u8,
) -> Result<usize, SqliteError> {
    let binding = db.connection.get().unwrap();
    // todo : potential sql optimization, selecting from MatchingResults ?
    let mut statement = binding
        .prepare_cached(
            "
                SELECT count(*) as count, 
                6371 * acos(
                    sin(?) * sin(latitude) +
                    cos(?) * cos(latitude) * cos(? - longitude)
                ) as distance
                FROM Users
                WHERE user_uuid <> ?
                AND gender = ?
                AND age <= ?
                AND age >= ?
                AND user_uuid NOT IN ( -- don't pick someone that the user has already swipped
                    SELECT swiped as user_uuid
                    FROM MatchingResults
                    WHERE swiper = ?
                )
                AND distance < ?
               ",
        )
        .map_err(map_sqlite_error)?;
    let potential_matches_count = statement
        .query_row(
            params![
                latitude,
                latitude,
                longitude,
                user_uuid,
                looking_for,
                age_max,
                age_min,
                user_uuid,
                search_radius
            ],
            |row| row.get("count"),
        )
        .map_err(map_sqlite_error)?;

    Ok(potential_matches_count)
}
