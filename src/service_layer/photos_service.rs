use crate::my_errors::service_errors::ServiceError;
use crate::service_layer::auth_service::JwtClaims;
use crate::utilities::responses::{response_ok, ApiResponse};
use axum::{http::StatusCode, Json};

// https://github.com/actix/examples/tree/master/forms/multipart
pub async fn save_file(
    jwt_claims: JwtClaims,
    // mut payload: Multipart,
) -> Result<(StatusCode, Json<ApiResponse<()>>), ServiceError> {
    // ) -> Result<HttpResponse, Error> {
    println!("ID user authorized to save file : {}", jwt_claims.user_uuid);
    // // iterate over multipart stream
    // while let Some(mut field) = payload.try_next().await? {
    //     // A multipart/form-data stream has to contain `content_disposition`
    //     let content_disposition = field
    //         .content_disposition()
    //         .ok_or_else(|| HttpResponse::BadRequest().finish())?;

    //     let filename = content_disposition.get_filename().map_or_else(
    //         || Uuid::new_v4().to_string(),
    //         |f| sanitize_filename::sanitize(f),
    //     );
    //     let filepath = filename; //format!("./tmp/{}", filename);

    //     // File::create is blocking operation, use threadpool
    //     let mut f = web::block(|| std::fs::File::create(filepath)).await?;

    //     // Field in turn is stream of *Bytes* object
    //     while let Some(chunk) = field.try_next().await? {
    //         // filesystem operations are blocking, we have to use threadpool
    //         f = web::block(move || f.write_all(&chunk).map(|_| f)).await?;
    //     }
    // }

    // Ok(HttpResponse::Ok().into())
    response_ok(None::<()>)
}
