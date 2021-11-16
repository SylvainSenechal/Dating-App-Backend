use std::{fmt, ops::Add};
use actix_web::{
    http::{StatusCode},
};
use serde::{Serialize};
use actix_web::{HttpResponse};


#[derive(Debug)]
pub enum CustomError {
    NotFound,
    UnknownSqliteError,
    SqliteFailure(libsqlite3_sys::Error),
    SqliteFailureExplained(libsqlite3_sys::Error, String),
    SqliteFailureNoText,
}
impl CustomError {
    pub fn name(&self) -> String {
        match self {
            Self::NotFound => "Ressource not found".to_string(),
            Self::SqliteFailureExplained(sqliteFailureDetail, explaination) => sqliteFailureDetail.to_string().add(" : ").add(explaination),
            Self::SqliteFailure(sqliteFailureDetail) => sqliteFailureDetail.to_string(),
            Self::SqliteFailureNoText => {
                "Something fucked up in the database, it's not your fault dud".to_string()
            }
            Self::UnknownSqliteError => "Unknown sqlite error".to_string(),
        }
    }
}
impl fmt::Display for CustomError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
#[derive(Serialize)]

struct ErrorResponse {
    code: u16,
    error: String,
    message: String,
}
impl actix_web::ResponseError for CustomError {
    fn status_code(&self) -> StatusCode {
        match *self {
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::SqliteFailure(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::SqliteFailureExplained(_, _) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::SqliteFailureNoText => StatusCode::INTERNAL_SERVER_ERROR,
            Self::UnknownSqliteError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let status_code = self.status_code();
        let error_response = ErrorResponse {
            code: status_code.as_u16(),
            message: self.to_string(),
            error: self.name(),
        };
        HttpResponse::build(status_code).json(error_response)
    }
}

pub fn map_sqlite_error(e: rusqlite::Error) -> CustomError {
    // Todo : Add this to the logger
    println!("map sqlite error found : {:?}", e);

    match e {
        rusqlite::Error::QueryReturnedNoRows => CustomError::NotFound,
        rusqlite::Error::SqliteFailure(sqliteFailureDetail, Some(explaination)) => CustomError::SqliteFailureExplained(sqliteFailureDetail, explaination),
        rusqlite::Error::SqliteFailure(sqliteFailureDetail, None) => CustomError::SqliteFailure(sqliteFailureDetail),
        rusqlite::Error::InvalidColumnIndex(_) => CustomError::SqliteFailureNoText,
        rusqlite::Error::InvalidColumnType(_, _, _) => CustomError::SqliteFailureNoText,
        rusqlite::Error::InvalidColumnName(_) => CustomError::SqliteFailureNoText,

        _ => CustomError::UnknownSqliteError,
    }
}