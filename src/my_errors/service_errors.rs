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
    InternalError,
    ServiceError(String),
    UserAlreadyExist,
    NoPotentialMatchFound,
    SqliteError(SqliteError),
    LoginError,
    JwtError,
    ForbiddenQuery,
    ValueNotAccepted(String, String), // (Value, Reason)
    TransactionError,
    UnknownServiceError,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ErrorCode {
    NoError = 0,
    UnspecifiedError = 1, // TODO : more specific error code
}

impl ServiceError {
    pub fn error_message(&self) -> String {
        match self {
            Self::InternalError => "Internal error".to_string(),
            Self::ServiceError(_) => "Service layer error".to_string(),
            Self::UserAlreadyExist => "User already exists".to_string(),
            Self::NoPotentialMatchFound => "No potential match found".to_string(),
            Self::SqliteError(_) => "Sqlite internal error".to_string(),
            Self::LoginError => "Login error".to_string(),
            Self::JwtError => "Jwt internal error".to_string(),
            Self::ForbiddenQuery => "Query forbidden error".to_string(),
            Self::ValueNotAccepted(value, reason) => "SQL provided value not accepted, value = "
                .to_string()
                .add(value)
                .add(" reason : ")
                .add(reason),
            Self::TransactionError => "Transaction error".to_string(),
            Self::UnknownServiceError => "Unknown service layer error".to_string(),
        }
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            Self::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
            Self::ServiceError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::UserAlreadyExist => StatusCode::UNPROCESSABLE_ENTITY,
            Self::NoPotentialMatchFound => StatusCode::NOT_FOUND,
            Self::SqliteError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::LoginError => StatusCode::INTERNAL_SERVER_ERROR,
            Self::JwtError => StatusCode::INTERNAL_SERVER_ERROR,
            Self::ForbiddenQuery => StatusCode::FORBIDDEN,
            Self::ValueNotAccepted(_, _) => StatusCode::FORBIDDEN,
            Self::TransactionError => StatusCode::INTERNAL_SERVER_ERROR,
            Self::UnknownServiceError => StatusCode::INTERNAL_SERVER_ERROR,
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
