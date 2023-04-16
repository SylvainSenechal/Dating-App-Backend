use crate::my_errors::sqlite_errors::map_sqlite_error;
use crate::my_errors::sqlite_errors::SqliteError;
use crate::AppState;
use chrono;
use rusqlite::params;
use serde::Serialize;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct TraceRequest {
    pub trace_id: Option<usize>,
    pub method: String,
    pub uri: String,
    pub user_agent: Option<String>,
}

#[derive(Serialize)]
pub struct GetTracesResponse {
    trace_id: Option<usize>,
    datetime: Option<String>,
    // pub ip: Option<std::net::SocketAddr>, TODO : Decide which fields to publicly show
    method: Option<String>,
    uri: Option<String>,
    user_agent: Option<String>,
    // data: Option<String>,
}

pub fn create_trace(db: &Arc<AppState>, trace: TraceRequest) -> Result<(), SqliteError> {
    let binding = db.connection.get().unwrap();
    let mut statement = binding
        .prepare_cached("INSERT INTO Traces (trace_uuid, trace_id, datetime, method, uri, user_agent) VALUES (?, ?, ?, ?, ?, ?)")
        .map_err(map_sqlite_error)?;
    statement
        .execute(params![
            Uuid::now_v7().to_string(),
            trace.trace_id,
            format!("{:?}", chrono::offset::Utc::now()),
            trace.method,
            trace.uri,
            trace.user_agent
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
                uri: row.get("uri")?,
                user_agent: row.get("user_agent")?,
            })
        })
        .map_err(map_sqlite_error)?;

    let mut traces = Vec::new();
    for trace in result_rows {
        traces.push(trace.map_err(map_sqlite_error)?);
    }

    Ok(traces)
}
