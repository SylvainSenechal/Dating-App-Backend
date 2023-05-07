use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3::error::SdkError;
use aws_sdk_s3::operation::{
    delete_object::{DeleteObjectError, DeleteObjectOutput},
    put_object::{PutObjectError, PutObjectOutput},
};
use aws_sdk_s3::Client;
use aws_smithy_http;

pub struct AwsClient {
    client: Client,
    bucket_name: String,
}

impl AwsClient {
    pub async fn new(bucket_name: String) -> AwsClient {
        let region_provider = RegionProviderChain::default_provider().or_else("ap-southeast-1");
        let shared_config = aws_config::from_env().region(region_provider).load().await;
        return AwsClient {
            client: Client::new(&shared_config),
            bucket_name: bucket_name,
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
