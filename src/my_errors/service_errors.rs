use crate::my_errors::sqlite_errors::SqliteError;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::Add;

#[derive(Serialize, Deserialize, Debug)]
pub struct ApiResponseError {
    pub error_message: String,
    pub error_code: ErrorCode,
}

#[derive(Debug)]
pub enum ServiceError {
    Internal,
    UserAlreadyExist,
    NoPotentialMatchFound,
    Sqlite(SqliteError),
    ForbiddenQuery,
    ValueNotAccepted(String, String), // (Value, Reason)
    Transaction,
    UnknownServiceProblem,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ErrorCode {
    NoError = 0,
    UnspecifiedError = 1, // TODO : more specific error code
}

impl ServiceError {
    pub fn error_message(&self) -> String {
        match self {
            Self::Internal => "Internal error".to_string(),
            Self::UserAlreadyExist => "User already exists".to_string(),
            Self::NoPotentialMatchFound => "No potential match found".to_string(),
            Self::Sqlite(_) => "Sqlite internal error".to_string(),
            Self::ForbiddenQuery => "Query forbidden error".to_string(),
            Self::ValueNotAccepted(value, reason) => "SQL provided value not accepted, value = "
                .to_string()
                .add(value)
                .add(" reason : ")
                .add(reason),
            Self::Transaction => "Transaction error".to_string(),
            Self::UnknownServiceProblem => "Unknown service layer error".to_string(),
        }
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            Self::Internal => StatusCode::INTERNAL_SERVER_ERROR,
            Self::UserAlreadyExist => StatusCode::UNPROCESSABLE_ENTITY,
            Self::NoPotentialMatchFound => StatusCode::NOT_FOUND,
            Self::Sqlite(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::ForbiddenQuery => StatusCode::FORBIDDEN,
            Self::ValueNotAccepted(_, _) => StatusCode::FORBIDDEN,
            Self::Transaction => StatusCode::INTERNAL_SERVER_ERROR,
            Self::UnknownServiceProblem => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
impl fmt::Display for ServiceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl IntoResponse for ServiceError {
    fn into_response(self) -> Response {
        let http_status = self.status_code();
        let body = Json(ApiResponseError {
            error_message: self.error_message(),
            error_code: ErrorCode::UnspecifiedError, // TODO
        });

        (http_status, body).into_response()
    }
}
