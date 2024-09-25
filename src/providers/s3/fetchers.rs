use std::str::FromStr;

use async_trait::async_trait;

use super::{types, client::{GetObjectBytesError, ListCommonPrefixesError, S3Client}};

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

/// Queries the list of the objects in the bucket, grouped by "/" delimiter.
/// Returns the list of block heights that can be fetched
pub async fn list_block_heights(
    lake_s3_client: &dyn S3Client,
    s3_bucket_name: &str,
    start_from_block_height: types::BlockHeight,
) -> Result<Vec<types::BlockHeight>, types::LakeError> {
    tracing::debug!(
        target: crate::LAKE_FRAMEWORK,
        "Fetching block heights from S3, after #{}...",
        start_from_block_height
    );

    let prefixes = lake_s3_client
        .list_common_prefixes(s3_bucket_name, &format!("{:0>12}", start_from_block_height))
        .await?;

    Ok(prefixes
        .iter()
        .map(|folder| u64::from_str(folder.as_str()))
        .filter_map(|num| num.ok())
        .collect())
}

/// By the given block height gets the objects:
/// - block.json
/// - shard_N.json
/// Reads the content of the objects and parses as a JSON.
/// Returns the result in `near_indexer_primitives::StreamerMessage`
pub(crate) async fn fetch_streamer_message(
    lake_s3_client: &dyn S3Client,
    s3_bucket_name: &str,
    block_height: types::BlockHeight,
) -> Result<near_indexer_primitives::StreamerMessage, types::LakeError> {
    let block_view = fetch_block_or_retry(lake_s3_client, s3_bucket_name, block_height).await?;

    let fetch_shards_futures = (0..block_view.chunks.len() as u64)
        .collect::<Vec<u64>>()
        .into_iter()
        .map(|shard_id| {
            fetch_shard_or_retry(lake_s3_client, s3_bucket_name, block_height, shard_id)
        });

    let shards = futures::future::try_join_all(fetch_shards_futures).await?;

    Ok(near_indexer_primitives::StreamerMessage {
        block: block_view,
        shards,
    })
}

/// Fetches the block data JSON from AWS S3 and returns the `BlockView`
pub async fn fetch_block(
    lake_s3_client: &dyn S3Client,
    s3_bucket_name: &str,
    block_height: types::BlockHeight,
) -> Result<near_indexer_primitives::views::BlockView, types::LakeError> {
    let bytes = lake_s3_client
        .get_object_bytes(s3_bucket_name, &format!("{:0>12}/block.json", block_height))
        .await?;

    Ok(serde_json::from_slice::<
        near_indexer_primitives::views::BlockView,
    >(&bytes)?)
}

/// Fetches the block data JSON from AWS S3 and returns the `BlockView` retrying until it succeeds (indefinitely)
pub async fn fetch_block_or_retry(
    lake_s3_client: &dyn S3Client,
    s3_bucket_name: &str,
    block_height: types::BlockHeight,
) -> Result<near_indexer_primitives::views::BlockView, types::LakeError> {
    loop {
        match fetch_block(lake_s3_client, s3_bucket_name, block_height).await {
            Ok(block_view) => break Ok(block_view),
            Err(err) => {
                if let types::LakeError::S3GetError { ref error } = err {
                    if let Some(get_object_error) =
                        error.downcast_ref::<aws_sdk_s3::operation::get_object::GetObjectError>()
                    {
                        tracing::debug!(
                            target: crate::LAKE_FRAMEWORK,
                            "Block #{:0>12} not found. Retrying immediately...\n{:#?}",
                            block_height,
                            get_object_error,
                        );
                    }

                    if let Some(bytes_error) =
                        error.downcast_ref::<aws_smithy_types::byte_stream::error::Error>()
                    {
                        tracing::debug!(
                            target: crate::LAKE_FRAMEWORK,
                            "Failed to read bytes from the block #{:0>12} response. Retrying immediately.\n{:#?}",
                            block_height,
                            bytes_error,
                        );
                    }

                    tracing::debug!(
                        target: crate::LAKE_FRAMEWORK,
                        "Failed to fetch block #{}, retrying immediately\n{:#?}",
                        block_height,
                        err
                    );
                }
            }
        }
    }
}

/// Fetches the shard data JSON from AWS S3 and returns the `IndexerShard`
pub async fn fetch_shard(
    lake_s3_client: &dyn S3Client,
    s3_bucket_name: &str,
    block_height: types::BlockHeight,
    shard_id: u64,
) -> Result<near_indexer_primitives::IndexerShard, types::LakeError> {
    let bytes = lake_s3_client
        .get_object_bytes(
            s3_bucket_name,
            &format!("{:0>12}/shard_{}.json", block_height, shard_id),
        )
        .await?;

    Ok(serde_json::from_slice::<
        near_indexer_primitives::IndexerShard,
    >(&bytes)?)
}

/// Fetches the shard data JSON from AWS S3 and returns the `IndexerShard`
pub async fn fetch_shard_or_retry(
    lake_s3_client: &dyn S3Client,
    s3_bucket_name: &str,
    block_height: types::BlockHeight,
    shard_id: u64,
) -> Result<near_indexer_primitives::IndexerShard, types::LakeError> {
    loop {
        match fetch_shard(lake_s3_client, s3_bucket_name, block_height, shard_id).await {
            Ok(shard) => break Ok(shard),
            Err(err) => {
                if let types::LakeError::S3ListError { ref error } = err {
                    if let Some(list_objects_error) =
                        error.downcast_ref::<aws_sdk_s3::operation::list_objects_v2::ListObjectsV2Error>()
                    {
                        tracing::debug!(
                            target: crate::LAKE_FRAMEWORK,
                            "Shard {} of block #{:0>12} not found. Retrying immediately...\n{:#?}",
                            shard_id,
                            block_height,
                            list_objects_error,
                        );
                    }

                    if let Some(bytes_error) =
                        error.downcast_ref::<aws_smithy_types::byte_stream::error::Error>()
                    {
                        tracing::debug!(
                            target: crate::LAKE_FRAMEWORK,
                            "Failed to read bytes from the shard {} of block #{:0>12} response. Retrying immediately.\n{:#?}",
                            shard_id,
                            block_height,
                            bytes_error,
                        );
                    }

                    tracing::debug!(
                        target: crate::LAKE_FRAMEWORK,
                        "Failed to fetch shard {} of block #{}, retrying immediately\n{:#?}",
                        shard_id,
                        block_height,
                        err
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use std::sync::Arc;

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
