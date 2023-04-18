use axum::{http::StatusCode, Json};
use serde::Serialize;

use crate::my_errors::service_errors::ServiceError;
use crate::service_layer::auth_service::AuthError;

#[derive(Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub message: Option<String>,
    pub code: u16,
    pub data: Option<T>,
}

// Error type : Generic that implements intoResponse ?
pub fn response_ok<T: Serialize>(
    data: Option<T>,
) -> Result<(StatusCode, Json<ApiResponse<T>>), ServiceError> {
    Ok((
        StatusCode::OK,
        Json(ApiResponse {
            message: None,
            code: StatusCode::OK.as_u16(),
            data: data,
        }),
    ))
}

pub fn response_ok_with_message<T: Serialize>(
    data: Option<T>,
    message: String,
) -> Result<(StatusCode, Json<ApiResponse<T>>), ServiceError> {
    Ok((
        StatusCode::OK,
        Json(ApiResponse {
            message: Some(message),
            code: StatusCode::OK.as_u16(),
            data: data,
        }),
    ))
}

pub fn response_auth_ok<T: Serialize>(
    data: Option<T>,
) -> Result<(StatusCode, Json<ApiResponse<T>>), AuthError> {
    Ok((
        StatusCode::OK,
        Json(ApiResponse {
            message: None,
            code: StatusCode::OK.as_u16(),
            data: data,
        }),
    ))
}

pub fn response_ok_auth_with_message<T: Serialize>(
    data: Option<T>,
    message: String,
) -> Result<(StatusCode, Json<ApiResponse<T>>), AuthError> {
    Ok((
        StatusCode::OK,
        Json(ApiResponse {
            message: Some(message),
            code: StatusCode::OK.as_u16(),
            data: data,
        }),
    ))
}

// pub fn response_ok_with_message<T: Serialize>(data: Option<T>, message: String) -> (StatusCode, Json<ApiResponse<T>>) {
//     (StatusCode::OK, Json(ApiResponse {
//         message: Some(message),
//         code: StatusCode::OK.as_u16(),
//         data: data,
//     }))
// }
