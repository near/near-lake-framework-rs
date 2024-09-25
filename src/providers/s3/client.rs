use std::error::Error;
use std::sync::Arc;

use async_trait::async_trait;

pub type S3ClientError = Arc<dyn Error + Send + Sync + 'static>;

#[derive(Debug, thiserror::Error, Clone)]
pub struct GetObjectBytesError(pub S3ClientError);

impl std::ops::Deref for GetObjectBytesError {
    type Target = S3ClientError;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::fmt::Display for GetObjectBytesError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "GetObjectBytesError: {}", self.0)
    }
}

impl From<aws_sdk_s3::error::SdkError<aws_sdk_s3::operation::get_object::GetObjectError>>
    for GetObjectBytesError
{
    fn from(
        error: aws_sdk_s3::error::SdkError<aws_sdk_s3::operation::get_object::GetObjectError>,
    ) -> Self {
        Self(Arc::new(error))
    }
}

impl From<aws_smithy_types::byte_stream::error::Error> for GetObjectBytesError {
    fn from(error: aws_smithy_types::byte_stream::error::Error) -> Self {
        Self(Arc::new(error))
    }
}

#[derive(Debug, thiserror::Error, Clone)]
pub struct ListCommonPrefixesError(pub S3ClientError);

impl std::fmt::Display for ListCommonPrefixesError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ListCommonPrefixesError: {}", self.0)
    }
}

impl std::ops::Deref for ListCommonPrefixesError {
    type Target = S3ClientError;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<aws_sdk_s3::error::SdkError<aws_sdk_s3::operation::list_objects_v2::ListObjectsV2Error>>
    for ListCommonPrefixesError
{
    fn from(
        error: aws_sdk_s3::error::SdkError<
            aws_sdk_s3::operation::list_objects_v2::ListObjectsV2Error,
        >,
    ) -> Self {
        Self(Arc::new(error))
    }
}

#[async_trait]
pub trait S3Client: Send + Sync {
    async fn get_object_bytes(
        &self,
        bucket: &str,
        prefix: &str,
    ) -> Result<Vec<u8>, GetObjectBytesError>;

    async fn list_common_prefixes(
        &self,
        bucket: &str,
        start_after_prefix: &str,
    ) -> Result<Vec<String>, ListCommonPrefixesError>;
}
