use crate::my_errors::sqlite_errors::map_sqlite_error;
use crate::my_errors::sqlite_errors::SqliteError;
use crate::AppState;
use chrono;
use rusqlite::params;
use serde::Serialize;
use std::sync::Arc;
pub struct TraceRequest<'a> {
    pub trace_id: Option<usize>,
    pub ip: Option<std::net::SocketAddr>,
    pub method: &'a str, // TODO : why no string
    pub path: &'a str,
    pub query_string: &'a str,
    pub data: Option<&'a str>,
}

#[derive(Serialize)]
// todo : see optional fields
pub struct GetTracesResponse {
    trace_id: Option<usize>,
    datetime: Option<String>,
    // pub ip: Option<std::net::SocketAddr>, TODO : Decide which fields to publicly show
    method: Option<String>,
    path: Option<String>,
    query_string: Option<String>,
    // data: Option<String>,
}

pub fn create_trace(db: &Arc<AppState>, trace: TraceRequest) -> Result<(), SqliteError> {
    let binding = db.connection.get().unwrap();
    let mut statement = binding
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

// All for now, later add filters
pub fn get_traces(db: &Arc<AppState>) -> Result<Vec<GetTracesResponse>, SqliteError> {
    let binding = db.connection.get().unwrap();
    let mut statement = binding
        .prepare_cached("SELECT * FROM traces;")
        .map_err(map_sqlite_error)?;
    let result_rows = statement
        .query_map(params![], |row| {
            Ok(GetTracesResponse {
                trace_id: row.get("trace_id")?,
                datetime: row.get("datetime")?,
                method: row.get("method")?,
                path: row.get("path")?,
                query_string: row.get("query_string")?,
            })
        })
        .map_err(map_sqlite_error)?;

    let mut traces = Vec::new();
    for trace in result_rows {
        traces.push(trace.map_err(map_sqlite_error)?);
    }

    Ok(traces)
}
