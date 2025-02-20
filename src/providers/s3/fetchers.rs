use std::str::FromStr;

use super::{client::S3Client, types};

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
///   Reads the content of the objects and parses as a JSON.
///   Returns the result in `near_indexer_primitives::StreamerMessage`
pub async fn fetch_streamer_message(
    lake_s3_client: &dyn S3Client,
    s3_bucket_name: &str,
    block_height: types::BlockHeight,
) -> Result<near_indexer_primitives::StreamerMessage, types::LakeError> {
    let block_view = fetch_block_or_retry(lake_s3_client, s3_bucket_name, block_height).await?;

    let fetch_shards_futures = block_view.chunks.iter().map(|chunk| {
        fetch_shard_or_retry(
            lake_s3_client,
            s3_bucket_name,
            block_height,
            chunk.shard_id.into(),
        )
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
