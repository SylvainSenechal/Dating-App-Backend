use actix_web::HttpResponse;
use serde::Serialize;

use actix_web::http::StatusCode;

#[derive(Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub message: Option<String>,
    pub code: u16,
    pub data: Option<T>,
}

// "User found".to_string() TODO : MESSAGE IN CONST

pub fn response_ok<T: Serialize>(data: Option<T>) -> HttpResponse {
    HttpResponse::Ok().json(ApiResponse {
        message: None,
        code: StatusCode::OK.as_u16(),
        data: data,
    })
}

pub fn response_ok_with_message<T: Serialize>(data: Option<T>, message: String) -> HttpResponse {
    HttpResponse::Ok().json(ApiResponse {
        message: Some(message),
        code: StatusCode::OK.as_u16(),
        data: data,
    })
}
