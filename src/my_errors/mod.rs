use serde::Serialize;

pub mod sqlite_errors;
pub mod service_errors;

#[derive(Serialize)]
struct ErrorResponse {
    code: u16,
    message: String,
    detailed_error: String,
}