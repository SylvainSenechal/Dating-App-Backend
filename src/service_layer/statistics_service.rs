use crate::service_layer::auth_service::JwtClaims;
use crate::utilities::responses::{response_ok, ApiResponse};
use crate::{data_access_layer, AppState};
use crate::{
    data_access_layer::trace_dal::GetTracesResponse, my_errors::service_errors::ServiceError,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use std::sync::Arc;

// TODO struct count json ?

pub async fn loved_count(
    // How many users loved you
    jwt_claims: JwtClaims,
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<usize>,
) -> Result<(StatusCode, Json<ApiResponse<usize>>), ServiceError> {
    if jwt_claims.user_id != user_id {
        return Err(ServiceError::ForbiddenQuery);
    }
    let swiped_count =
        data_access_layer::user_dal::User::swiped_count(&state, jwt_claims.user_id, 1)?;
    response_ok(Some(swiped_count))
}

pub async fn rejected_count(
    // How many users rejected you
    jwt_claims: JwtClaims,
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<usize>,
) -> Result<(StatusCode, Json<ApiResponse<usize>>), ServiceError> {
    if jwt_claims.user_id != user_id {
        return Err(ServiceError::ForbiddenQuery);
    }
    let swiped_count =
        data_access_layer::user_dal::User::swiped_count(&state, jwt_claims.user_id, 0)?;
    response_ok(Some(swiped_count))
}

pub async fn loving_count(
    // How many users you loved
    jwt_claims: JwtClaims,
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<usize>,
) -> Result<(StatusCode, Json<ApiResponse<usize>>), ServiceError> {
    if jwt_claims.user_id != user_id {
        return Err(ServiceError::ForbiddenQuery);
    }
    let swiped_count =
        data_access_layer::user_dal::User::swiping_count(&state, jwt_claims.user_id, 1)?;
    response_ok(Some(swiped_count))
}

pub async fn rejecting_count(
    // How many users you rejected
    jwt_claims: JwtClaims,
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<usize>,
) -> Result<(StatusCode, Json<ApiResponse<usize>>), ServiceError> {
    if jwt_claims.user_id != user_id {
        return Err(ServiceError::ForbiddenQuery);
    }
    let swiped_count =
        data_access_layer::user_dal::User::swiping_count(&state, jwt_claims.user_id, 0)?;
    response_ok(Some(swiped_count))
}

pub async fn backend_activity(
    jwt_claims: JwtClaims,
    State(state): State<Arc<AppState>>,
) -> Result<(StatusCode, Json<ApiResponse<Vec<GetTracesResponse>>>), ServiceError> {
    let traces = data_access_layer::trace_dal::get_traces(&state)?;
    response_ok(Some(traces))
}
