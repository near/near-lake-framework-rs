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

#[derive(Clone, Debug)]
pub struct LakeS3Client {
    s3: aws_sdk_s3::Client,
}

impl LakeS3Client {
    pub fn new(s3: aws_sdk_s3::Client) -> Self {
        Self { s3 }
    }

    pub fn from_conf(config: aws_sdk_s3::config::Config) -> Self {
        let s3_client = aws_sdk_s3::Client::from_conf(config);

        Self { s3: s3_client }
    }
}

#[async_trait]
impl S3Client for LakeS3Client {
    async fn get_object_bytes(
        &self,
        bucket: &str,
        prefix: &str,
    ) -> Result<Vec<u8>, GetObjectBytesError> {
        let object = self
            .s3
            .get_object()
            .bucket(bucket)
            .key(prefix)
            .request_payer(aws_sdk_s3::types::RequestPayer::Requester)
            .send()
            .await?;

        let bytes = object.body.collect().await?.into_bytes().to_vec();

        Ok(bytes)
    }

    async fn list_common_prefixes(
        &self,
        bucket: &str,
        start_after_prefix: &str,
    ) -> Result<Vec<String>, ListCommonPrefixesError> {
        let response = self
            .s3
            .list_objects_v2()
            .max_keys(1000) // 1000 is the default and max value for this parameter
            .delimiter("/".to_string())
            .start_after(start_after_prefix)
            .request_payer(aws_sdk_s3::types::RequestPayer::Requester)
            .bucket(bucket)
            .send()
            .await?;

        let prefixes = match response.common_prefixes {
            None => vec![],
            Some(common_prefixes) => common_prefixes
                .into_iter()
                .filter_map(|common_prefix| common_prefix.prefix)
                .collect::<Vec<String>>()
                .into_iter()
                .filter_map(|prefix_string| prefix_string.split('/').next().map(String::from))
                .collect(),
        };

        Ok(prefixes)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use std::sync::Arc;

    use crate::providers::s3::fetchers::fetch_streamer_message;
    use async_trait::async_trait;

    #[derive(Clone, Debug)]
    pub struct LakeS3Client {}

    #[async_trait]
    impl S3Client for LakeS3Client {
        async fn get_object_bytes(
            &self,
            _bucket: &str,
            prefix: &str,
        ) -> Result<Vec<u8>, GetObjectBytesError> {
            let path = format!("{}/blocks/{}", env!("CARGO_MANIFEST_DIR"), prefix);
            tokio::fs::read(path)
                .await
                .map_err(|e| GetObjectBytesError(Arc::new(e)))
        }

        async fn list_common_prefixes(
            &self,
            _bucket: &str,
            _start_after: &str,
        ) -> Result<Vec<String>, ListCommonPrefixesError> {
            Ok(Vec::new())
        }
    }

    #[tokio::test]
    async fn deserializes_meta_transactions() {
        let lake_client = LakeS3Client {};
        let streamer_message =
            fetch_streamer_message(&lake_client, "near-lake-data-mainnet", 879765)
                .await
                .unwrap();

        let delegate_action = &streamer_message.shards[0]
            .chunk
            .as_ref()
            .unwrap()
            .transactions[0]
            .transaction
            .actions[0];

        assert_eq!(
            serde_json::to_value(delegate_action).unwrap(),
            serde_json::json!({
                "Delegate": {
                    "delegate_action": {
                        "sender_id": "test.near",
                        "receiver_id": "test.near",
                        "actions": [
                          {
                            "AddKey": {
                              "public_key": "ed25519:CnQMksXTTtn81WdDujsEMQgKUMkFvDJaAjDeDLTxVrsg",
                              "access_key": {
                                "nonce": 0,
                                "permission": "FullAccess"
                              }
                            }
                          }
                        ],
                        "nonce": 879546,
                        "max_block_height": 100,
                        "public_key": "ed25519:8Rn4FJeeRYcrLbcrAQNFVgvbZ2FCEQjgydbXwqBwF1ib"
                    },
                    "signature": "ed25519:25uGrsJNU3fVgUpPad3rGJRy2XQum8gJxLRjKFCbd7gymXwUxQ9r3tuyBCD6To7SX5oSJ2ScJZejwqK1ju8WdZfS"
                }
            })
        );
    }
}
