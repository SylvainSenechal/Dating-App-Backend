use crate::data_access_layer::lover_dal::Lover; // todo : refactor into dto/dal logic
use crate::my_errors::service_errors::ServiceError;
use crate::my_errors::sqlite_errors::transaction_error;
use crate::service_layer::auth_service::JwtClaims;
use crate::utilities::responses::{response_ok, ApiResponse};
use crate::{data_access_layer, AppState};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use std::sync::Arc;

pub async fn get_lovers(
    jwt_claims: JwtClaims,
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<usize>,
) -> Result<(StatusCode, Json<ApiResponse<Vec<Lover>>>), ServiceError> {
    if jwt_claims.user_id != user_id {
        return Err(ServiceError::ForbiddenQuery);
    }
    let lovers_found = data_access_layer::lover_dal::get_lovers(&state, user_id);
    match lovers_found {
        Ok(lovers) => response_ok(Some(lovers)),
        Err(err) => Err(ServiceError::SqliteError(err)),
    }
}

pub async fn tick_love(
    jwt_claims: JwtClaims,
    State(state): State<Arc<AppState>>,
    Path(love_id): Path<usize>,
) -> Result<(StatusCode, Json<ApiResponse<()>>), ServiceError> {
    // TODO : verify that authorized.id is in the love_id relation

    data_access_layer::lover_dal::tick_love(&state, love_id, jwt_claims.user_id)?;
    response_ok(None::<()>)
}
