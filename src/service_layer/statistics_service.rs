use crate::requests::requests;
use crate::service_layer::auth_service::JwtClaims;
use crate::utilities::responses::{response_ok, ApiResponse};
use crate::{data_access_layer, AppState};
use crate::{
    data_access_layer::trace_dal::GetTracesResponse, my_errors::service_errors::ServiceError,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use std::sync::Arc;

pub async fn loved_count(
    // How many users loved you
    jwt_claims: JwtClaims,
    State(state): State<Arc<AppState>>,
    Path(user_uuid): Path<String>,
) -> Result<(StatusCode, Json<ApiResponse<usize>>), ServiceError> {
    if jwt_claims.user_uuid != user_uuid {
        return Err(ServiceError::ForbiddenQuery);
    }
    let swiped_count = data_access_layer::user_dal::swiped_count(&state, jwt_claims.user_uuid, 1)?;
    response_ok(Some(swiped_count))
}

pub async fn rejected_count(
    // How many users rejected you
    jwt_claims: JwtClaims,
    State(state): State<Arc<AppState>>,
    Path(user_uuid): Path<String>,
) -> Result<(StatusCode, Json<ApiResponse<usize>>), ServiceError> {
    if jwt_claims.user_uuid != user_uuid {
        return Err(ServiceError::ForbiddenQuery);
    }
    let swiped_count = data_access_layer::user_dal::swiped_count(&state, jwt_claims.user_uuid, 0)?;
    response_ok(Some(swiped_count))
}

pub async fn loving_count(
    // How many users you loved
    jwt_claims: JwtClaims,
    State(state): State<Arc<AppState>>,
    Path(user_uuid): Path<String>,
) -> Result<(StatusCode, Json<ApiResponse<usize>>), ServiceError> {
    if jwt_claims.user_uuid != user_uuid {
        return Err(ServiceError::ForbiddenQuery);
    }
    let swiped_count = data_access_layer::user_dal::swiping_count(&state, jwt_claims.user_uuid, 1)?;
    response_ok(Some(swiped_count))
}

pub async fn rejecting_count(
    // How many users you rejected
    jwt_claims: JwtClaims,
    State(state): State<Arc<AppState>>,
    Path(user_uuid): Path<String>,
) -> Result<(StatusCode, Json<ApiResponse<usize>>), ServiceError> {
    if jwt_claims.user_uuid != user_uuid {
        return Err(ServiceError::ForbiddenQuery);
    }
    let swiped_count = data_access_layer::user_dal::swiping_count(&state, jwt_claims.user_uuid, 0)?;
    response_ok(Some(swiped_count))
}

pub async fn matching_potential(
    jwt_claims: JwtClaims,
    State(state): State<Arc<AppState>>,
    Path(user_uuid): Path<String>,
    matching_potential_request: Query<requests::MatchingPotentialRequest>,
) -> Result<(StatusCode, Json<ApiResponse<usize>>), ServiceError> {
    if jwt_claims.user_uuid != user_uuid {
        return Err(ServiceError::ForbiddenQuery);
    }
    let potential_matches_count = data_access_layer::lover_dal::potential_matches_count(
        &state,
        jwt_claims.user_uuid,
        matching_potential_request.looking_for,
        matching_potential_request.search_radius,
        matching_potential_request.latitude,
        matching_potential_request.longitude,
        matching_potential_request.looking_for_age_min,
        matching_potential_request.looking_for_age_max,
    )?;
    response_ok(Some(potential_matches_count))
}

pub async fn backend_activity(
    _: JwtClaims,
    State(state): State<Arc<AppState>>,
) -> Result<(StatusCode, Json<ApiResponse<Vec<GetTracesResponse>>>), ServiceError> {
    let traces = data_access_layer::trace_dal::get_traces(&state)?;
    response_ok(Some(traces))
}
