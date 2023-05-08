use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3::error::SdkError;
use aws_sdk_s3::operation::{
    delete_object::{DeleteObjectError, DeleteObjectOutput},
    put_object::{PutObjectError, PutObjectOutput},
};
use aws_sdk_s3::Client;
use aws_smithy_http;
use aws_types::region::Region;

pub struct AwsClient {
    client: Client,
    bucket_name: String,
    pub r2_image_domain: String,
}

impl AwsClient {
    pub async fn new(
        r2_account_id: String,
        r2_image_domain: String,
        bucket_name: String,
    ) -> AwsClient {
        let endpoint_url = format!(
            "{}{}{}",
            "https://", r2_account_id, ".r2.cloudflarestorage.com"
        );
        let shared_config = aws_config::from_env()
            .endpoint_url(endpoint_url)
            .region(Region::new("auto"))
            .load()
            .await;
        return AwsClient {
            client: Client::new(&shared_config),
            bucket_name: bucket_name,
            r2_image_domain: r2_image_domain,
        };
    }

    pub async fn upload_object(
        &self,
        key: &str,
        content_type: &str,
        data: aws_smithy_http::byte_stream::ByteStream,
    ) -> Result<PutObjectOutput, SdkError<PutObjectError>> {
        self.client
            .put_object()
            .bucket(&self.bucket_name)
            .key(key)
            .body(data)
            .content_type(content_type)
            .send()
            .await
    }

    pub async fn delete_object(
        &self,
        key: &str,
    ) -> Result<DeleteObjectOutput, SdkError<DeleteObjectError>> {
        self.client
            .delete_object()
            .bucket(&self.bucket_name)
            .key(key)
            .send()
            .await
    }
}
