use crate::my_errors::sqlite_errors::map_sqlite_error;
use crate::my_errors::sqlite_errors::SqliteError;
use crate::AppState;
use actix_web::web;
use chrono;
use rusqlite::params;

pub struct TraceRequest<'a> {
    pub trace_id: Option<usize>,
    pub ip: Option<std::net::SocketAddr>,
    pub method: &'a str,
    pub path: &'a str,
    pub query_string: &'a str,
    pub data: Option<&'a str>,
}

pub fn create_trace(db: &web::Data<AppState>, trace: TraceRequest) -> Result<(), SqliteError> {
    let mut statement = db
        .connection
        .prepare_cached("INSERT INTO Traces (trace_id, datetime, ip, method, path, query_string, data) VALUES (?, ?, ?, ?, ?, ?, ?)")
        .map_err(map_sqlite_error)?;
    statement
        .execute(params![
            trace.trace_id,
            format!("{:?}", chrono::offset::Utc::now()),
            trace.ip.expect("should have ip").to_string(), // TODO avoid expect here
            trace.method,
            trace.path,
            trace.query_string,
            trace.data
        ])
        .map_err(map_sqlite_error)?;

    Ok(())
}
