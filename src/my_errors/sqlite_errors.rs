use crate::my_errors::service_errors::ServiceError;
use std::fmt;

#[derive(Debug)]
pub enum SqliteError {
    NotFound,
    UnknownSqliteProblem,
    // SqliteFailure(libsqlite3_sys::Error),
    // SqliteFailureExplained(libsqlite3_sys::Error, String),
    SqliteFailureNoText,
}

impl fmt::Display for SqliteError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub fn map_sqlite_error(e: rusqlite::Error) -> SqliteError {
    println!("sqlite error encountered : {:?}", e);
    match e {
        rusqlite::Error::QueryReturnedNoRows => SqliteError::NotFound,
        // rusqlite::Error::SqliteFailure(sqlite_failure_detail, Some(explaination)) => {
        //     SqliteError::SqliteFailureExplained(sqlite_failure_detail, explaination)
        // }
        // rusqlite::Error::SqliteFailure(sqlite_failure_detail, None) => {
        //     SqliteError::SqliteFailure(sqlite_failure_detail)
        // }
        rusqlite::Error::InvalidColumnIndex(_) => SqliteError::SqliteFailureNoText,
        rusqlite::Error::InvalidColumnType(_, _, _) => SqliteError::SqliteFailureNoText,
        rusqlite::Error::InvalidColumnName(_) => SqliteError::SqliteFailureNoText,

        _ => SqliteError::UnknownSqliteProblem,
    }
}

pub fn transaction_error(e: rusqlite::Error) -> ServiceError {
    println!("Transaction error : {:?}", e);
    ServiceError::Transaction
    // match e {
    //     rusqlite::Error::QueryReturnedNoRows => SqliteError::NotFound,
    //     rusqlite::Error::SqliteFailure(sqlite_failure_detail, Some(explaination)) => {
    //         SqliteError::SqliteFailureExplained(sqlite_failure_detail, explaination)
    //     }
    //     rusqlite::Error::SqliteFailure(sqlite_failure_detail, None) => {
    //         SqliteError::SqliteFailure(sqlite_failure_detail)
    //     }
    //     rusqlite::Error::InvalidColumnIndex(_) => SqliteError::SqliteFailureNoText,
    //     rusqlite::Error::InvalidColumnType(_, _, _) => SqliteError::SqliteFailureNoText,
    //     rusqlite::Error::InvalidColumnName(_) => SqliteError::SqliteFailureNoText,

    //     _ => SqliteError::UnknownSqliteError,
    // }
}

impl From<SqliteError> for ServiceError {
    fn from(_error: SqliteError) -> Self {
        ServiceError::Internal
    }
}
