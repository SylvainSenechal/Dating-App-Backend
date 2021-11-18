use actix_web::{web, HttpRequest, HttpResponse, Result as actixResult};
use rusqlite::params;
use serde::{Deserialize, Serialize};

use crate::my_errors::sqlite_errors::map_sqlite_error;
use crate::my_errors::sqlite_errors::SqliteError;
use crate::AppState;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct User {
    // todo : add ID
    #[serde(default)]
    pub pseudo: String,
    #[serde(default)]
    pub email: String, // use option instead, see what returning json option does (maybe serde remove option none ?)
    pub password: String,
    #[serde(default)]
    pub age: Option<u8>, // todo : voir pourquoi option..
}

impl User {
    pub async fn create_user(
        db: &web::Data<AppState>,
        user: User,
    ) -> actixResult<HttpResponse, SqliteError> {
        let mut statement = db
            .connection
            .prepare("INSERT INTO users (pseudo, email, password, age) VALUES (?, ?, ?, ?)")
            .map_err(map_sqlite_error)?;
        let nb_inserted = statement
            .execute(params![user.pseudo, user.email, user.password, user.age])
            .map_err(map_sqlite_error)?;

        Ok(HttpResponse::Ok().body("User created"))
    }

    // pub async fn get_user(
    //     // TODO : get with id/pseudo instead of json
    //     db: web::Data<AppState>,
    //     web::Path(id): web::Path<u32>,
    //     user: web::Json<User>,
    //     req: HttpRequest,
    // ) -> actixResult<HttpResponse, SqliteError> {
    //     println!("{:?}", db.connection);
    //     println!("{:?}", user);
    //     println!("{:?}", req);
    //     println!("{:?}", id);

    //     let mut statement = db
    //         .connection
    //         .prepare("SELECT * FROM persons WHERE pseudo = ?")
    //         .map_err(map_sqlite_error)?;

    //     let res = statement
    //         .query_row(params![user.pseudo], |row| {
    //             Ok(User {
    //                 pseudo: row.get("pseudo")?,
    //                 age: row.get("age")?,
    //                 email: row.get("email")?,
    //                 ..Default::default()
    //             })
    //         })
    //         .map_err(map_sqlite_error)?;

    //     Ok(HttpResponse::Ok().json(res))
    // }

    pub async fn get_user(db: &web::Data<AppState>, pseudo: String) -> Result<User, SqliteError> {
        let mut statement = db
            .connection
            .prepare("SELECT * FROM users WHERE pseudo = ?")
            .map_err(map_sqlite_error)?;

        let user_found = statement
            .query_row(params![pseudo], |row| {
                Ok(User {
                    pseudo: row.get("pseudo")?,
                    age: row.get("age")?,
                    email: row.get("email")?,
                    ..Default::default()
                })
            })
            .map_err(map_sqlite_error)?;

        Ok(user_found)
    }

    pub async fn get_users(db: &web::Data<AppState>) -> Result<Vec<User>, SqliteError> {
        let mut statement = db
            .connection
            .prepare("SELECT * FROM users")
            .map_err(map_sqlite_error)?;
        let result_rows = statement
            .query_map([], |row| {
                Ok(User {
                    pseudo: row.get("pseudo")?,
                    age: row.get("age")?,
                    email: row.get("email")?,
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
