use crate::data_access_layer::photo_dal;
use crate::my_errors::service_errors::ServiceError;
use crate::requests::requests;
use crate::service_layer::auth_service::JwtClaims;
use crate::utilities::responses::{response_ok, ApiResponse};
use axum::{http::StatusCode, Json};

use crate::{data_access_layer, AppState};
use aws_smithy_http;
use axum::extract::{Multipart, Path, State};
use std::fmt;
use std::str::FromStr;
use std::sync::Arc;
use uuid::Uuid;

enum PhotoType {
    Png,
    Jpg,
    Jpeg,
}

impl FromStr for PhotoType {
    type Err = ServiceError;

    fn from_str(input: &str) -> Result<PhotoType, Self::Err> {
        match input {
            // todo : check now that we use canvas image
            "image/png" => Ok(PhotoType::Png),
            "image/jpg" => Ok(PhotoType::Jpg),
            "image/jpeg" => Ok(PhotoType::Jpeg),
            format => Err(ServiceError::ValueNotAccepted(
                format.to_string(),
                "Chose between png, jpg and jpeg.".to_string(),
            )),
        }
    }
}

impl fmt::Display for PhotoType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PhotoType::Png => write!(f, "image/png"),
            PhotoType::Jpg => write!(f, "image/jpg"),
            PhotoType::Jpeg => write!(f, "image/jpeg"),
        }
    }
}

pub async fn save_photo(
    jwt_claims: JwtClaims,
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<ApiResponse<()>>), ServiceError> {
    let field = multipart
        .next_field()
        .await
        .or(Err(ServiceError::ValueNotAccepted(
            "no photo in the form data multipart".to_string(),
            "no photo provided in the form data multipart".to_string(),
        )))?
        .ok_or(ServiceError::Internal)?;

    let user = data_access_layer::user_dal::get_user_by_uuid(&state, jwt_claims.user_uuid.clone())?;
    let display_order = if let Some(existing_photos) = user.photo_urls {
        existing_photos.split(",").collect::<Vec<&str>>().len() + 1
    } else {
        1
    };

    if display_order > 6 {
        return Err(ServiceError::ValueNotAccepted(
            display_order.to_string(),
            "You can only have up to 6 photos".to_string(),
        ));
    }

    let image_key = Uuid::now_v7().to_string();
    let content_type = PhotoType::from_str(field.content_type().ok_or(ServiceError::Internal)?)?;
    let image_data: aws_smithy_http::byte_stream::ByteStream =
        field.bytes().await.or(Err(ServiceError::Internal))?.into();

    if let Err(e) = state
        .aws_client
        .upload_object(&image_key, &content_type.to_string(), image_data)
        .await
    {
        println!("aws upload object error: {:?}", e);
        return Err(ServiceError::Internal);
    }

    let url = format!("{}{}", state.aws_client.r2_image_domain, image_key);
    data_access_layer::photo_dal::create_user_photo(
        &state,
        image_key,
        jwt_claims.user_uuid,
        url.to_string(),
        display_order,
    )?;

    response_ok(None::<()>)
}

pub async fn delete_photo(
    jwt_claims: JwtClaims,
    State(state): State<Arc<AppState>>,
    Path(photo_uuid): Path<String>,
) -> Result<(StatusCode, Json<ApiResponse<()>>), ServiceError> {
    let user_photos = photo_dal::get_user_photos(&state, jwt_claims.user_uuid.clone())?;

    let mut photo_owned_by_user = false;
    let mut order_shift = 0;
    for photo in user_photos {
        if photo.photo_uuid == photo_uuid {
            photo_owned_by_user = true;
            order_shift = photo.display_order;
            break;
        }
    }
    if photo_owned_by_user {
        let deletion_result = state.aws_client.delete_object(&photo_uuid).await;
        match deletion_result {
            Ok(_) => {
                photo_dal::delete_photo(&state, photo_uuid)?;
                photo_dal::shift_order_photos(&state, jwt_claims.user_uuid, order_shift)?;
            }
            Err(err) => {
                println!("error delete photo: {:?}", err);
            }
        }
    } else {
        return Err(ServiceError::ForbiddenQuery);
    }

    response_ok(None::<()>)
}

pub async fn switch_photos(
    jwt_claims: JwtClaims,
    State(state): State<Arc<AppState>>,
    Json(request_switch_photo): Json<requests::SwitchPhotosRequest>,
) -> Result<(StatusCode, Json<ApiResponse<()>>), ServiceError> {
    let user_photos = photo_dal::get_user_photos(&state, jwt_claims.user_uuid.clone())?;
    let mut photos_found = 0;
    let mut order1 = 0;
    let mut order2 = 0;
    for photo in user_photos {
        if photo.photo_uuid == request_switch_photo.photo_uuid1 {
            order1 = photo.display_order;
            photos_found += 1;
        }
        if photo.photo_uuid == request_switch_photo.photo_uuid2.clone() {
            order2 = photo.display_order;
            photos_found += 1;
        }
    }
    if photos_found == 2 {
        photo_dal::switch_order_photos(
            &state,
            order1,
            order2,
            request_switch_photo.photo_uuid1,
            request_switch_photo.photo_uuid2,
        )?;
    } else {
        return Err(ServiceError::ForbiddenQuery);
    }

    response_ok(None::<()>)
}
