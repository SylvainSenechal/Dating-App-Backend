use crate::my_errors::service_errors::ServiceError;
use crate::requests::requests;
use crate::service_layer::auth_service::JwtClaims;
use crate::utilities::responses::{response_ok, ApiResponse};
use crate::{data_access_layer, AppState};
use axum::{extract::State, http::StatusCode, Json};
use std::sync::Arc;

pub async fn create_feedback(
    jwt_claims: JwtClaims,
    State(state): State<Arc<AppState>>,
    Json(create_feedback_request): Json<requests::CreateFeedbackRequest>,
) -> Result<(StatusCode, Json<ApiResponse<()>>), ServiceError> {
    let creation_datetime = format!("{:?}", chrono::offset::Utc::now());
    data_access_layer::feedback_dal::create_feedback(
        &state,
        create_feedback_request.feedback_message,
        jwt_claims.user_uuid,
        &creation_datetime,
    )?;

    response_ok(None::<()>)
}
