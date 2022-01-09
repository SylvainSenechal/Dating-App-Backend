use actix_web::web;
use rusqlite::params;
use serde::{Deserialize, Serialize};

use crate::my_errors::sqlite_errors::map_sqlite_error;
use crate::my_errors::sqlite_errors::SqliteError;
use crate::service_layer::user_service::{CreateUserRequest};
use crate::AppState;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct User {
    pub id: u32,
    #[serde(default)]
    pub pseudo: String,
    pub password: String,
    #[serde(default)]
    pub age: Option<u8>, // todo : voir pourquoi option..
}

impl User {
    pub fn create_user(
        db: &web::Data<AppState>,
        user: CreateUserRequest,
    ) -> Result<(), SqliteError> {
        let mut statement = db
            .connection
            .prepare("INSERT INTO users (pseudo, password, age) VALUES (?, ?, ?)")
            .map_err(map_sqlite_error)?;
        statement
            .execute(params![user.pseudo, user.password, user.age])
            .map_err(map_sqlite_error)?;

        Ok(())
    }

    pub fn get_user_by_pseudo(db: &web::Data<AppState>, pseudo: String) -> Result<User, SqliteError> {
        let mut statement = db
            .connection
            .prepare_cached("SELECT * FROM users WHERE pseudo = ?")
            .map_err(map_sqlite_error)?;

        let user_found = statement
            .query_row(params![pseudo], |row| {
                Ok(User {
                    id: row.get("person_id")?,
                    pseudo: row.get("pseudo")?,
                    // email: row.get("email")?,
                    password: row.get("password")?,
                    age: row.get("age")?,
                    ..Default::default()
                })
            })
            .map_err(map_sqlite_error)?;

        Ok(user_found)
    }

    pub fn get_user_by_id(db: &web::Data<AppState>, userId: u32) -> Result<User, SqliteError> {
        let mut statement = db
            .connection
            .prepare_cached("SELECT * FROM users WHERE person_id = ?")
            .map_err(map_sqlite_error)?;

        let user_found = statement
            .query_row(params![userId], |row| {
                Ok(User {
                    id: row.get("person_id")?,
                    pseudo: row.get("pseudo")?,
                    // email: row.get("email")?,
                    password: row.get("password")?,
                    age: row.get("age")?,
                    ..Default::default()
                })
            })
            .map_err(map_sqlite_error)?;

        Ok(user_found)
    }

    pub fn get_users(db: &web::Data<AppState>) -> Result<Vec<User>, SqliteError> {
        let mut statement = db
            .connection
            .prepare("SELECT * FROM users")
            .map_err(map_sqlite_error)?;
        let result_rows = statement
            .query_map([], |row| {
                Ok(User {
                    id: row.get("person_id")?,
                    pseudo: row.get("pseudo")?,
                    // email: row.get("email")?,
                    password: row.get("password")?,
                    age: row.get("age")?,
                    ..Default::default()
                })
            })
            .map_err(map_sqlite_error)?;

        let mut persons = Vec::new();
        for person in result_rows {
            persons.push(person.map_err(map_sqlite_error)?);
        }

        Ok(persons)
    }
}