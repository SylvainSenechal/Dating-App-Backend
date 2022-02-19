use actix_web::web;
use chrono;
use rusqlite::params;
use serde::{Deserialize, Serialize};

use crate::my_errors::sqlite_errors::map_sqlite_error;
use crate::my_errors::sqlite_errors::SqliteError;
use crate::service_layer::user_service::{CreateUserRequest, UpdateUserInfosReq};
use crate::AppState;

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    pub id: usize,
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
}

impl User {
    pub fn create_user(
        db: &web::Data<AppState>,
        user: CreateUserRequest,
    ) -> Result<(), SqliteError> {
        let mut statement = db
            .connection
            .prepare_cached("INSERT INTO Users (name, password, email, last_seen, age, latitude, longitude, gender, looking_for) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)")
            .map_err(map_sqlite_error)?;
        statement
            .execute(params![
                user.name,
                user.password,
                user.email,
                format!("{:?}", chrono::offset::Utc::now()), // Last seen = now
                user.age,
                user.latitude,
                user.longitude,
                user.gender,
                user.looking_for
            ])
            .map_err(map_sqlite_error)?;

        Ok(())
    }

    pub fn get_user_by_email(db: &web::Data<AppState>, email: String) -> Result<User, SqliteError> {
        let mut statement = db
            .connection
            .prepare_cached("SELECT * FROM Users WHERE email = ?")
            .map_err(map_sqlite_error)?;

        statement
            .query_row(params![email], |row| {
                Ok(User {
                    id: row.get("user_id")?,
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
                })
            })
            .map_err(map_sqlite_error)
    }

    // This route is for internal use as the password returned is the real Argon-hashed one
    pub fn get_user_password_by_email(
        db: &web::Data<AppState>,
        email: String,
    ) -> Result<(usize, String), SqliteError> {
        let mut statement = db
            .connection
            .prepare_cached("SELECT * FROM Users WHERE email = ?")
            .map_err(map_sqlite_error)?;

        statement
            .query_row(params![email], |row| {
                Ok((row.get("user_id")?, row.get("password")?))
            })
            .map_err(map_sqlite_error)
    }

    pub fn get_user_by_id(db: &web::Data<AppState>, user_id: usize) -> Result<User, SqliteError> {
        let mut statement = db
            .connection
            .prepare_cached("SELECT * FROM Users WHERE user_id = ?")
            .map_err(map_sqlite_error)?;

        statement
            .query_row(params![user_id], |row| {
                Ok(User {
                    id: row.get("user_id")?,
                    name: row.get("name")?,
                    password: "Have fun with this password bro".to_string(),
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
                })
            })
            .map_err(map_sqlite_error)
    }

    pub fn update_user_infos(
        db: &web::Data<AppState>,
        user: UpdateUserInfosReq,
    ) -> Result<(), SqliteError> {
        println!("{:?}", user);
        let mut statement = db
            .connection
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
                WHERE user_id = ?",
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
                user.id
            ])
            .map_err(map_sqlite_error)?;

        Ok(())
    }

    pub fn update_user_last_seen(
        db: &web::Data<AppState>,
        user_id: usize,
    ) -> Result<(), SqliteError> {
        let mut statement = db
            .connection
            .prepare_cached(
                "UPDATE Users
                SET last_seen = ?
                WHERE user_id = ?",
            )
            .map_err(map_sqlite_error)?;

        statement
            .execute(params![
                format!("{:?}", chrono::offset::Utc::now()), // Last seen = now
                user_id
            ])
            .map_err(map_sqlite_error)?;

        Ok(())
    }

    pub fn get_users(db: &web::Data<AppState>) -> Result<Vec<User>, SqliteError> {
        let mut statement = db
            .connection
            .prepare_cached("SELECT * FROM Users")
            .map_err(map_sqlite_error)?;
        let result_rows = statement
            .query_map([], |row| {
                Ok(User {
                    id: row.get("user_id")?,
                    name: row.get("name")?,
                    password: "Have fun with this password bro".to_string(),
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
                })
            })
            .map_err(map_sqlite_error)?;

        let mut persons = Vec::new();
        for person in result_rows {
            persons.push(person.map_err(map_sqlite_error)?);
        }

        Ok(persons)
    }

    pub fn find_love_target(
        db: &web::Data<AppState>,
        user_id: usize,
        looking_for: String,
        gender: String,
        search_radius: u16,
        latitude: f32,
        longitude: f32,
        age_min: u8,
        age_max: u8,
    ) -> Result<User, SqliteError> {
        let mut statement = db
            .connection
            .prepare_cached(
                "
                SELECT *
                FROM Users
                WHERE user_id <> ?
                AND gender = ?
                AND looking_for = ?
                AND age <= ?
                AND age >= ?
                AND user_id NOT IN (
                    SELECT swiped as user_id
                    FROM MatchingResults
                    WHERE swiper = ?
                )
                ORDER BY datetime(last_seen) DESC -- Getting the most recently active user
               ",
            )
            .map_err(map_sqlite_error)?;
        println!("{}", age_max);
        println!("{}", age_min);
        statement
            .query_row(
                params![user_id, looking_for, gender,  age_max, age_min, user_id],
                |row| {
                    Ok(User {
                        id: row.get("user_id")?,
                        name: row.get("name")?,
                        password: "Have fun with this password bro".to_string(),
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
                    })
                },
            )
            .map_err(map_sqlite_error)
    }

    pub fn swipe_user(
        db: &web::Data<AppState>,
        swiper: usize,
        swiped: usize,
        love: u8, // 0 : swiper dont like swiped, 1 : swiper like swiped
    ) -> Result<(), SqliteError> {
        let mut statement = db
            .connection
            .prepare_cached("INSERT INTO MatchingResults (swiper, swiped, love) VALUES (?, ?, ?)")
            .map_err(map_sqlite_error)?;
        statement
            .execute(params![swiper, swiped, love,])
            .map_err(map_sqlite_error)?;

        Ok(())
    }

    pub fn check_mutual_love(
        db: &web::Data<AppState>,
        lover1: usize,
        lover2: usize,
    ) -> Result<usize, SqliteError> {
        let mut statement = db
            .connection
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
                Ok(row.get("count")?)
            })
            .map_err(map_sqlite_error)?;

        Ok(mutual_love_count)
    }

    pub fn swiped_count(
        db: &web::Data<AppState>,
        user_id: usize,
        loved: u8,
    ) -> Result<usize, SqliteError> {
        let mut statement = db
            .connection
            .prepare_cached(
                "
            SELECT COUNT(*) as count 
            FROM MatchingResults
            WHERE swiped = ? AND love = ?
            ",
            )
            .map_err(map_sqlite_error)?;
        let swiped_count: usize = statement
            .query_row(params![user_id, loved], |row| Ok(row.get("count")?))
            .map_err(map_sqlite_error)?;

        Ok(swiped_count)
    }

    pub fn swiping_count(
        db: &web::Data<AppState>,
        user_id: usize,
        loved: u8,
    ) -> Result<usize, SqliteError> {
        let mut statement = db
            .connection
            .prepare_cached(
                "
            SELECT COUNT(*) as count
            FROM MatchingResults
            WHERE swiper = ? AND love = ?
            ",
            )
            .map_err(map_sqlite_error)?;
        let swiping_count: usize = statement
            .query_row(params![user_id, loved], |row| Ok(row.get("count")?))
            .map_err(map_sqlite_error)?;

        Ok(swiping_count)
    }
}
