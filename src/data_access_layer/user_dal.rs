use chrono;
use rusqlite::params;
use serde::{Deserialize, Serialize};

use crate::configs::app_state::AppState;
use crate::my_errors::sqlite_errors::map_sqlite_error;
use crate::my_errors::sqlite_errors::SqliteError;
use crate::requests::requests;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    pub uuid: String,
    pub private_uuid: String,
    pub name: String,
    pub password: String,
    pub email: String,
    pub last_seen: String,
    pub age: u8,
    pub latitude: f32,
    pub longitude: f32,
    pub gender: String,
    pub looking_for: String,
    pub search_radius: u16,
    pub looking_for_age_min: u8,
    pub looking_for_age_max: u8,
    pub description: String,
    pub photo_urls: Option<String>,
    pub photo_display_orders: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PotentialLover {
    pub uuid: String,
    pub name: String,
    pub last_seen: String,
    pub age: u8,
    pub gender: String,
    pub description: String,
    pub distance: f32,
    pub photo_urls: Option<String>,
    pub photo_display_orders: Option<String>,
}

pub fn create_user(
    db: &Arc<AppState>,
    user: requests::CreateUserRequest,
) -> Result<(), SqliteError> {
    let binding = db.connection.get().unwrap();
    let mut statement = binding
            .prepare_cached("INSERT INTO Users (user_uuid, private_user_uuid, name, password, email, last_seen, age, latitude, longitude, gender, looking_for) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")
            .map_err(map_sqlite_error)?;
    statement
        .execute(params![
            Uuid::now_v7().to_string(),
            Uuid::now_v7().to_string(),
            user.name,
            user.password,
            user.email,
            format!("{:?}", chrono::offset::Utc::now()), // Last seen = now
            user.age,
            user.latitude * std::f32::consts::PI / 180.,
            user.longitude * std::f32::consts::PI / 180.,
            user.gender,
            user.looking_for
        ])
        .map_err(map_sqlite_error)?;

    Ok(())
}

// todo : replace by user_exists ?
pub fn get_user_by_email(db: &Arc<AppState>, email: String) -> Result<User, SqliteError> {
    let binding = db.connection.get().unwrap();
    let mut statement = binding
        .prepare_cached("SELECT * FROM Users WHERE email = ? LIMIT 1")
        .map_err(map_sqlite_error)?;

    statement
        .query_row(params![email], |row| {
            Ok(User {
                uuid: row.get("user_uuid")?,
                private_uuid: row.get("private_user_uuid")?,
                name: row.get("name")?,
                email: row.get("email")?,
                password: "Have fun with this password bro".to_string(),
                last_seen: row.get("last_seen")?,
                age: row.get("age")?,
                latitude: row.get("latitude")?,
                longitude: row.get("longitude")?,
                gender: row.get("gender")?,
                looking_for: row.get("looking_for")?,
                search_radius: row.get("search_radius")?,
                looking_for_age_min: row.get("looking_for_age_min")?,
                looking_for_age_max: row.get("looking_for_age_max")?,
                description: row.get("description")?,
                photo_urls: Some("".to_string()),
                photo_display_orders: Some("".to_string()),
            })
        })
        .map_err(map_sqlite_error)
}

// This route is for internal use as the password returned is the real Argon-hashed one
pub fn get_user_password_by_email(
    db: &Arc<AppState>,
    email: String,
) -> Result<(String, String, String), SqliteError> {
    let binding = db.connection.get().unwrap();
    let mut statement = binding
        .prepare_cached("SELECT * FROM Users WHERE email = ? LIMIT 1")
        .map_err(map_sqlite_error)?;

    statement
        .query_row(params![email], |row| {
            Ok((
                row.get("user_uuid")?,
                row.get("private_user_uuid")?,
                row.get("password")?,
            ))
        })
        .map_err(map_sqlite_error)
}

// This route is for internal use as the password returned is the real Argon-hashed one
pub fn get_user_password_by_user_uuid(
    db: &Arc<AppState>,
    user_uuid: String,
) -> Result<String, SqliteError> {
    let binding = db.connection.get().unwrap();
    let mut statement = binding
        .prepare_cached("SELECT password FROM Users WHERE user_uuid = ?")
        .map_err(map_sqlite_error)?;

    statement
        .query_row(params![user_uuid], |row| row.get("password"))
        .map_err(map_sqlite_error)
}

pub fn get_user_uuid_by_private_uuid(
    db: &Arc<AppState>,
    private_uuid: String,
) -> Result<String, SqliteError> {
    let binding = db.connection.get().unwrap();
    let mut statement = binding
        .prepare_cached("SELECT user_uuid FROM Users WHERE private_user_uuid = ? LIMIT 1")
        .map_err(map_sqlite_error)?;

    statement
        .query_row(params![private_uuid], |row| row.get("user_uuid"))
        .map_err(map_sqlite_error)
}

pub fn get_user_by_uuid(db: &Arc<AppState>, user_uuid: String) -> Result<User, SqliteError> {
    let binding = db.connection.get().unwrap();
    let mut statement = binding
        .prepare_cached(
            "
        SELECT *,
        GROUP_CONCAT(Photos.url, ',') as photo_urls,
        GROUP_CONCAT(Photos.display_order, ',') as photo_display_orders
        
        FROM Users 
        LEFT JOIN Photos ON Users.user_uuid = Photos.user_uuid 
        WHERE Users.user_uuid = ?
        LIMIT 1
        ",
        )
        .map_err(map_sqlite_error)?;
    statement
        .query_row(params![user_uuid], |row| {
            Ok(User {
                uuid: row.get("user_uuid")?,
                private_uuid: row.get("private_user_uuid")?,
                name: row.get("name")?,
                password: "".to_string(), // todo : fix
                email: row.get("email")?,
                last_seen: row.get("last_seen")?,
                age: row.get("age")?,
                latitude: row.get("latitude")?,
                longitude: row.get("longitude")?,
                gender: row.get("gender")?,
                looking_for: row.get("looking_for")?,
                search_radius: row.get("search_radius")?,
                looking_for_age_min: row.get("looking_for_age_min")?,
                looking_for_age_max: row.get("looking_for_age_max")?,
                description: row.get("description")?,
                photo_urls: row.get("photo_urls")?,
                photo_display_orders: row.get("photo_display_orders")?,
            })
        })
        .map_err(map_sqlite_error)
}

pub fn delete_user_by_uuid(db: &Arc<AppState>, user_uuid: String) -> Result<(), SqliteError> {
    let binding = db.connection.get().unwrap();
    let mut statement = binding
        .prepare_cached("DELETE FROM Users WHERE user_uuid = ?")
        .map_err(map_sqlite_error)?;

    statement
        .execute(params![user_uuid])
        .map_err(map_sqlite_error)?;

    Ok(())
}

pub fn update_user_infos(
    db: &Arc<AppState>,
    user: requests::UpdateUserInfosReq,
) -> Result<(), SqliteError> {
    let binding = db.connection.get().unwrap();
    let mut statement = binding
        .prepare_cached(
            "UPDATE Users
                SET name = ?,
                email = ?,
                age = ?,
                last_seen = ?,
                latitude = ?,
                longitude = ?,
                gender = ?,
                looking_for = ?,
                search_radius = ?,
                looking_for_age_min = ?,
                looking_for_age_max = ?,
                description = ?
                WHERE user_uuid = ?",
        )
        .map_err(map_sqlite_error)?;

    statement
        .execute(params![
            user.name,
            user.email,
            user.age,
            format!("{:?}", chrono::offset::Utc::now()), // Last seen = now
            user.latitude,
            user.longitude,
            user.gender,
            user.looking_for,
            user.search_radius,
            user.looking_for_age_min,
            user.looking_for_age_max,
            user.description,
            user.uuid
        ])
        .map_err(map_sqlite_error)?;

    Ok(())
}

pub fn update_user_last_seen(db: &Arc<AppState>, user_uuid: String) -> Result<(), SqliteError> {
    let binding = db.connection.get().unwrap();
    let mut statement = binding
        .prepare_cached(
            "UPDATE Users
                SET last_seen = ?
                WHERE user_uuid = ?",
        )
        .map_err(map_sqlite_error)?;

    statement
        .execute(params![
            format!("{:?}", chrono::offset::Utc::now()), // Last seen = now
            user_uuid
        ])
        .map_err(map_sqlite_error)?;

    Ok(())
}

pub fn find_love_target(
    db: &Arc<AppState>,
    user_uuid: String,
    looking_for: String,
    search_radius: u16,
    latitude: f32,
    longitude: f32,
    age_min: u8,
    age_max: u8,
) -> Result<PotentialLover, SqliteError> {
    let binding = db.connection.get().unwrap();
    let mut statement = binding
        .prepare_cached(
            "
                SELECT *, 
                6371 * acos(
                    sin(?) * sin(latitude) +
                    cos(?) * cos(latitude) * cos(? - longitude)
                ) as distance,
                GROUP_CONCAT(Photos.url, ',') as photo_urls,
                GROUP_CONCAT(Photos.display_order, ',') as photo_display_orders

                FROM Users
                LEFT JOIN Photos ON Users.user_uuid = Photos.user_uuid 
                WHERE Users.user_uuid <> ?
                AND Users.gender = ?
                AND Users.age <= ?
                AND Users.age >= ?
                AND Users.user_uuid NOT IN ( -- don't pick someone that the user has already swipped
                    SELECT swiped as user_uuid
                    FROM MatchingResults
                    WHERE swiper = ?
                )
                AND distance < ?
                ORDER BY datetime(Users.last_seen) DESC -- Getting the most recently active user
                LIMIT 1
               ",
        )
        .map_err(map_sqlite_error)?;

    let result = statement.query_row(
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
        |row| {
            Ok(PotentialLover {
                uuid: row.get("user_uuid")?,
                name: row.get("name")?,
                last_seen: row.get("last_seen")?,
                age: row.get("age")?,
                distance: row.get("distance")?,
                gender: row.get("gender")?,
                description: row.get("description")?,
                photo_urls: row.get("photo_urls")?,
                photo_display_orders: row.get("photo_display_orders")?,
            })
        },
    );
    // Do no use ? to handle error here :
    // Because of the group_concat, when there is no user found, we will still get a row with null
    // value, and then when trying to access values "row.get("user_uuid")?," we will get an error that
    // is different than just "QueryReturnedNoRows" : InvalidColumnType(1, "user_uuid", Null)
    match result {
        Ok(potential_lover) => Ok(potential_lover),
        Err(rusqlite::Error::InvalidColumnType(_, _, _)) => Err(SqliteError::NotFound),
        Err(rusqlite::Error::QueryReturnedNoRows) => Err(SqliteError::NotFound),
        Err(_) => Err(SqliteError::UnknownSqliteProblem),
    }
}

pub fn swipe_user(
    db: &Arc<AppState>,
    swiper: String,
    swiped: String,
    love: u8, // 0 : swiper dont like swiped, 1 : swiper like swiped
) -> Result<(), SqliteError> {
    let binding = db.connection.get().unwrap();
    let mut statement = binding
        .prepare_cached(
            "INSERT INTO MatchingResults (match_uuid, swiper, swiped, love) VALUES (?, ?, ?, ?)",
        )
        .map_err(map_sqlite_error)?;
    statement
        .execute(params![Uuid::now_v7().to_string(), swiper, swiped, love,])
        .map_err(map_sqlite_error)?;

    Ok(())
}

pub fn check_mutual_love(
    db: &Arc<AppState>,
    lover1: String,
    lover2: String,
) -> Result<usize, SqliteError> {
    let binding = db.connection.get().unwrap();
    let mut statement = binding
        .prepare_cached(
            "
            SELECT COUNT(*) as count 
            FROM MatchingResults
            WHERE (swiper = ? AND swiped = ? AND love = 1) 
            OR (swiper = ? AND swiped = ? AND love = 1)",
        )
        .map_err(map_sqlite_error)?;
    let mutual_love_count: usize = statement
        .query_row(params![lover1, lover2, lover2, lover1], |row| {
            row.get("count")
        })
        .map_err(map_sqlite_error)?;

    Ok(mutual_love_count)
}

pub fn swiped_count(
    db: &Arc<AppState>,
    user_uuid: String,
    loved: u8,
) -> Result<usize, SqliteError> {
    let binding = db.connection.get().unwrap();
    let mut statement = binding
        .prepare_cached(
            "
            SELECT COUNT(*) as count 
            FROM MatchingResults
            WHERE swiped = ? AND love = ?
            ",
        )
        .map_err(map_sqlite_error)?;
    let swiped_count: usize = statement
        .query_row(params![user_uuid, loved], |row| row.get("count"))
        .map_err(map_sqlite_error)?;

    Ok(swiped_count)
}

pub fn swiping_count(
    db: &Arc<AppState>,
    user_uuid: String,
    loved: u8,
) -> Result<usize, SqliteError> {
    let binding = db.connection.get().unwrap();
    let mut statement = binding
        .prepare_cached(
            "
            SELECT COUNT(*) as count
            FROM MatchingResults
            WHERE swiper = ? AND love = ?
            ",
        )
        .map_err(map_sqlite_error)?;
    let swiping_count: usize = statement
        .query_row(params![user_uuid, loved], |row| row.get("count"))
        .map_err(map_sqlite_error)?;

    Ok(swiping_count)
}
