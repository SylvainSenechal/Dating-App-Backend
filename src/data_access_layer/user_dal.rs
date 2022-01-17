use actix_web::web;
use rusqlite::params;
use serde::{Deserialize, Serialize};

use crate::my_errors::sqlite_errors::map_sqlite_error;
use crate::my_errors::sqlite_errors::SqliteError;
use crate::service_layer::user_service::{CreateUserRequest, UpdateUserInfosReq};
use crate::AppState;

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    pub id: u32,
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

impl User {
    pub fn create_user(
        db: &web::Data<AppState>,
        user: CreateUserRequest,
    ) -> Result<(), SqliteError> {
        let mut statement = db
            .connection
            .prepare("INSERT INTO Users (name, password, email, age, latitude, longitude, gender, looking_for) VALUES (?, ?, ?, ?, ?, ?, ?, ?)")
            .map_err(map_sqlite_error)?;
        statement
            .execute(params![
                user.name,
                user.password,
                user.email,
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

        let user_found = statement
            .query_row(params![email], |row| {
                Ok(User {
                    id: row.get("user_id")?,
                    name: row.get("name")?,
                    email: row.get("email")?,
                    password: row.get("password")?, // TODO : DO NOT SEND BACK THE PASSWORD
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

        Ok(user_found)
    }

    pub fn get_user_by_id(db: &web::Data<AppState>, userId: u32) -> Result<User, SqliteError> {
        let mut statement = db
            .connection
            .prepare_cached("SELECT * FROM Users WHERE user_id = ?")
            .map_err(map_sqlite_error)?;

        let user_found = statement
            .query_row(params![userId], |row| {
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

        Ok(user_found)
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

    pub fn get_users(db: &web::Data<AppState>) -> Result<Vec<User>, SqliteError> {
        let mut statement = db
            .connection
            .prepare("SELECT * FROM Users")
            .map_err(map_sqlite_error)?;
        let result_rows = statement
            .query_map([], |row| {
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
        for person in result_rows {
            persons.push(person.map_err(map_sqlite_error)?);
        }

        Ok(persons)
    }

    pub fn find_love_target(
        db: &web::Data<AppState>,
        userId: u32,
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
               ",
            )
            .map_err(map_sqlite_error)?;

        let user_found = statement
            .query_row(params![userId, looking_for, gender], |row| {
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

        Ok(user_found)
    }

    pub fn swipe_user(
        db: &web::Data<AppState>,
        swiper: u32,
        swiped: u32,
        love: u8, // 0 : swiper dont like swiped, 1 : swiper like swiped
    ) -> Result<(), SqliteError> {
        let mut statement = db
            .connection
            .prepare("INSERT INTO MatchingResults (swiper, swiped, love) VALUES (?, ?, ?)")
            .map_err(map_sqlite_error)?;
        statement
            .execute(params![swiper, swiped, love,])
            .map_err(map_sqlite_error)?;

        Ok(())
    }

    pub fn check_mutual_love(
        db: &web::Data<AppState>,
        lover1: u32,
        lover2: u32,
    ) -> Result<usize, SqliteError> {
        let mut statement = db
            .connection
            .prepare(
                "
            SELECT * 
            FROM MatchingResults
            WHERE (swiper = ? AND swiped = ?) 
            OR (swiper = ? AND swiped = ?)
            AND love = 1
            ",
            )
            .map_err(map_sqlite_error)?;
        let rows_found = statement
            .query_map(params![lover1, lover2, lover2, lover1], |row| Ok(()))
            .map_err(map_sqlite_error)?;

        Ok(rows_found.count())
    }

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
}
