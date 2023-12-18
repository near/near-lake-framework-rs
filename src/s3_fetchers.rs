use async_trait::async_trait;
use std::str::FromStr;

#[async_trait]
pub trait S3Client {
    async fn get_object(
        &self,
        bucket: &str,
        prefix: &str,
    ) -> Result<
        aws_sdk_s3::operation::get_object::GetObjectOutput,
        aws_sdk_s3::error::SdkError<aws_sdk_s3::operation::get_object::GetObjectError>,
    >;

    async fn list_objects(
        &self,
        bucket: &str,
        start_after: &str,
    ) -> Result<
        aws_sdk_s3::operation::list_objects_v2::ListObjectsV2Output,
        aws_sdk_s3::error::SdkError<aws_sdk_s3::operation::list_objects_v2::ListObjectsV2Error>,
    >;
}

#[derive(Clone, Debug)]
pub struct LakeS3Client {
    s3: aws_sdk_s3::Client,
}

impl LakeS3Client {
    pub fn new(s3: aws_sdk_s3::Client) -> Self {
        Self { s3 }
    }
}

#[async_trait]
impl S3Client for LakeS3Client {
    async fn get_object(
        &self,
        bucket: &str,
        prefix: &str,
    ) -> Result<
        aws_sdk_s3::operation::get_object::GetObjectOutput,
        aws_sdk_s3::error::SdkError<aws_sdk_s3::operation::get_object::GetObjectError>,
    > {
        Ok(self
            .s3
            .get_object()
            .bucket(bucket)
            .key(prefix)
            .request_payer(aws_sdk_s3::types::RequestPayer::Requester)
            .send()
            .await?)
    }

    async fn list_objects(
        &self,
        bucket: &str,
        start_after: &str,
    ) -> Result<
        aws_sdk_s3::operation::list_objects_v2::ListObjectsV2Output,
        aws_sdk_s3::error::SdkError<aws_sdk_s3::operation::list_objects_v2::ListObjectsV2Error>,
    > {
        Ok(self
            .s3
            .list_objects_v2()
            .max_keys(1000) // 1000 is the default and max value for this parameter
            .delimiter("/".to_string())
            .start_after(start_after)
            .request_payer(aws_sdk_s3::types::RequestPayer::Requester)
            .bucket(bucket)
            .send()
            .await?)
    }
}

/// Queries the list of the objects in the bucket, grouped by "/" delimiter.
/// Returns the list of block heights that can be fetched
pub(crate) async fn list_block_heights(
    lake_s3_client: &impl S3Client,
    s3_bucket_name: &str,
    start_from_block_height: crate::types::BlockHeight,
) -> Result<
    Vec<crate::types::BlockHeight>,
    crate::types::LakeError<aws_sdk_s3::operation::list_objects_v2::ListObjectsV2Error>,
> {
    tracing::debug!(
        target: crate::LAKE_FRAMEWORK,
        "Fetching block heights from S3, after #{}...",
        start_from_block_height
    );
    let response = lake_s3_client
        .list_objects(s3_bucket_name, &format!("{:0>12}", start_from_block_height))
        .await?;

    Ok(match response.common_prefixes {
        None => vec![],
        Some(common_prefixes) => common_prefixes
            .into_iter()
            .filter_map(|common_prefix| common_prefix.prefix)
            .collect::<Vec<String>>()
            .into_iter()
            .filter_map(|prefix_string| {
                prefix_string
                    .split('/')
                    .next()
                    .map(u64::from_str)
                    .and_then(|num| num.ok())
            })
            .collect(),
    })
}

/// By the given block height gets the objects:
/// - block.json
/// - shard_N.json
/// Reads the content of the objects and parses as a JSON.
/// Returns the result in `near_indexer_primitives::StreamerMessage`
pub(crate) async fn fetch_streamer_message(
    lake_s3_client: &impl S3Client,
    s3_bucket_name: &str,
    block_height: crate::types::BlockHeight,
) -> Result<
    near_indexer_primitives::StreamerMessage,
    crate::types::LakeError<aws_sdk_s3::operation::get_object::GetObjectError>,
> {
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
    lake_s3_client: &impl S3Client,
    s3_bucket_name: &str,
    block_height: crate::types::BlockHeight,
) -> Result<
    near_indexer_primitives::views::BlockView,
    crate::types::LakeError<aws_sdk_s3::operation::get_object::GetObjectError>,
> {
    let body_bytes = lake_s3_client
        .get_object(s3_bucket_name, &format!("{:0>12}/block.json", block_height))
        .await?
        .body
        .collect()
        .await?
        .into_bytes();

    Ok(serde_json::from_slice::<
        near_indexer_primitives::views::BlockView,
    >(body_bytes.as_ref())?)
}

/// Fetches the block data JSON from AWS S3 and returns the `BlockView` retrying until it succeeds (indefinitely)
pub async fn fetch_block_or_retry(
    lake_s3_client: &impl S3Client,
    s3_bucket_name: &str,
    block_height: crate::types::BlockHeight,
) -> Result<
    near_indexer_primitives::views::BlockView,
    crate::types::LakeError<aws_sdk_s3::operation::get_object::GetObjectError>,
> {
    loop {
        match fetch_block(lake_s3_client, s3_bucket_name, block_height).await {
            Ok(block_view) => break Ok(block_view),
            Err(err) => match err {
                crate::types::LakeError::AwsError { .. } => {
                    tracing::debug!(
                        target: crate::LAKE_FRAMEWORK,
                        "Block #{:0>12} not found. Retrying in immediately...\n{:#?}",
                        block_height,
                        err,
                    );
                }
                crate::types::LakeError::AwsSmithyError { .. } => {
                    tracing::debug!(
                        target: crate::LAKE_FRAMEWORK,
                        "Failed to read bytes from the block #{:0>12} response. Retrying immediately.\n{:#?}",
                        block_height,
                        err,
                    );
                }
                _ => {
                    tracing::debug!(
                        target: crate::LAKE_FRAMEWORK,
                        "Failed to fetch block #{}, retrying immediately\n{:#?}",
                        block_height,
                        err
                    );
                }
            },
        }
    }
}

/// Fetches the shard data JSON from AWS S3 and returns the `IndexerShard`
pub async fn fetch_shard(
    lake_s3_client: &impl S3Client,
    s3_bucket_name: &str,
    block_height: crate::types::BlockHeight,
    shard_id: u64,
) -> Result<
    near_indexer_primitives::IndexerShard,
    crate::types::LakeError<aws_sdk_s3::operation::get_object::GetObjectError>,
> {
    let body_bytes = lake_s3_client
        .get_object(
            s3_bucket_name,
            &format!("{:0>12}/shard_{}.json", block_height, shard_id),
        )
        .await?
        .body
        .collect()
        .await?
        .into_bytes();

    Ok(serde_json::from_slice::<
        near_indexer_primitives::IndexerShard,
    >(body_bytes.as_ref())?)
}

/// Fetches the shard data JSON from AWS S3 and returns the `IndexerShard`
pub async fn fetch_shard_or_retry(
    lake_s3_client: &impl S3Client,
    s3_bucket_name: &str,
    block_height: crate::types::BlockHeight,
    shard_id: u64,
) -> Result<
    near_indexer_primitives::IndexerShard,
    crate::types::LakeError<aws_sdk_s3::operation::get_object::GetObjectError>,
> {
    loop {
        match fetch_shard(lake_s3_client, s3_bucket_name, block_height, shard_id).await {
            Ok(shard) => break Ok(shard),
            Err(err) => match err {
                crate::types::LakeError::AwsError { .. } => {
                    tracing::debug!(
                        target: crate::LAKE_FRAMEWORK,
                        "Shard {} of block #{:0>12} not found. Retrying in immediately...\n{:#?}",
                        shard_id,
                        block_height,
                        err,
                    );
                }
                crate::types::LakeError::AwsSmithyError { .. } => {
                    tracing::debug!(
                        target: crate::LAKE_FRAMEWORK,
                        "Failed to read bytes from the shard {} of block #{:0>12} response. Retrying immediately.\n{:#?}",
                        shard_id,
                        block_height,
                        err,
                    );
                }
                _ => {
                    tracing::debug!(
                        target: crate::LAKE_FRAMEWORK,
                        "Failed to fetch shard {} of block #{}, retrying immediately\n{:#?}",
                        shard_id,
                        block_height,
                        err
                    );
                }
            },
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use async_trait::async_trait;

    use aws_sdk_s3::operation::get_object::GetObjectOutput;
    use aws_sdk_s3::operation::list_objects_v2::ListObjectsV2Output;
    use aws_sdk_s3::primitives::ByteStream;

    use aws_smithy_types::body::SdkBody;

    #[derive(Clone, Debug)]
    pub struct LakeS3Client {}

    #[async_trait]
    impl S3Client for LakeS3Client {
        async fn get_object(
            &self,
            _bucket: &str,
            prefix: &str,
        ) -> Result<
            aws_sdk_s3::operation::get_object::GetObjectOutput,
            aws_sdk_s3::error::SdkError<aws_sdk_s3::operation::get_object::GetObjectError>,
        > {
            let path = format!("{}/blocks/{}", env!("CARGO_MANIFEST_DIR"), prefix);
            let file_bytes = tokio::fs::read(path).await.unwrap();
            let stream = ByteStream::new(SdkBody::from(file_bytes));
            Ok(GetObjectOutput::builder().body(stream).build())
        }

        async fn list_objects(
            &self,
            _bucket: &str,
            _start_after: &str,
        ) -> Result<
            aws_sdk_s3::operation::list_objects_v2::ListObjectsV2Output,
            aws_sdk_s3::error::SdkError<aws_sdk_s3::operation::list_objects_v2::ListObjectsV2Error>,
        > {
            Ok(ListObjectsV2Output::builder().build())
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
