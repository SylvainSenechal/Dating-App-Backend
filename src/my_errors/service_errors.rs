use std::fmt;
use std::ops::Add;
use actix_web::http::StatusCode;
use actix_web::HttpResponse;

use crate::my_errors::sqlite_errors::SqliteError;
use crate::my_errors::ErrorResponse;

#[derive(Debug)]

pub enum ServiceError {
    ServiceError(String),
    UserAlreadyExist,
    SqliteError(SqliteError),
    LoginError,
    JwtError,
    ForbiddenQuery,
    ValueNotAccepted(String, String), // (Value, Reason)
    TransactionError,
    UnknownServiceError
}
impl ServiceError {
    pub fn error_message(&self) -> String {
        match self {
            Self::ServiceError(_) => "Service layer error".to_string(),
            Self::UserAlreadyExist => "User already exists".to_string(),
            Self::SqliteError(_) => "Sqlite internal error".to_string(),
            Self::LoginError => "Login error".to_string(),
            Self::JwtError => "Jwt internal error".to_string(),
            Self::ForbiddenQuery => "Query forbidden error".to_string(),
            Self::ValueNotAccepted(value, reason) => "SQL provided value not accepted, value = ".to_string().add(value).add(" reason : ").add(reason),
            Self::TransactionError => "Transaction error".to_string(),
            Self::UnknownServiceError => "Unknown service layer error".to_string(),
        }
    }
}
impl fmt::Display for ServiceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl actix_web::ResponseError for ServiceError {
    fn status_code(&self) -> StatusCode {
        match *self {
            Self::ServiceError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::UserAlreadyExist => StatusCode::UNPROCESSABLE_ENTITY,
            Self::SqliteError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::LoginError => StatusCode::INTERNAL_SERVER_ERROR,
            Self::JwtError => StatusCode::INTERNAL_SERVER_ERROR,
            Self::ForbiddenQuery => StatusCode::FORBIDDEN,
            Self::ValueNotAccepted(_, _) => StatusCode::FORBIDDEN,
            Self::TransactionError => StatusCode::INTERNAL_SERVER_ERROR,
            Self::UnknownServiceError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let status_code = self.status_code();
        let error_response = ErrorResponse {
            code: status_code.as_u16(),
            message: self.error_message(),
            detailed_error: self.to_string(),
        };
        HttpResponse::build(status_code).json(error_response)
    }
}