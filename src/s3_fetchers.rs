use std::str::FromStr;

use aws_sdk_s3::Client;

const ESTIMATED_SHARDS_COUNT: usize = 4;

/// Queries the list of the objects in the bucket, grouped by "/" delimiter.
/// Returns the list of block heights that can be fetched
pub(crate) async fn list_block_heights(
    s3_client: &Client,
    s3_bucket_name: &str,
    start_from_block_height: crate::types::BlockHeight,
    number_of_blocks_requested: usize,
) -> Result<
    Vec<crate::types::BlockHeight>,
    crate::types::LakeError<aws_sdk_s3::error::ListObjectsV2Error>,
> {
    tracing::debug!(
        target: crate::LAKE_FRAMEWORK,
        "Fetching block heights from S3, after #{}...",
        start_from_block_height
    );
    let response = s3_client
        .list_objects_v2()
        .max_keys(std::cmp::min(
            (number_of_blocks_requested * (1 + ESTIMATED_SHARDS_COUNT)).try_into()?,
            1000i32,
        ))
        .delimiter("/".to_string())
        .start_after(format!("{:0>12}", start_from_block_height))
        .request_payer(aws_sdk_s3::model::RequestPayer::Requester)
        .bucket(s3_bucket_name)
        .send()
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
    s3_client: &Client,
    s3_bucket_name: &str,
    block_height: crate::types::BlockHeight,
) -> Result<
    near_indexer_primitives::StreamerMessage,
    crate::types::LakeError<aws_sdk_s3::error::GetObjectError>,
> {
    let block_view = {
        let body_bytes = loop {
            match s3_client
                .get_object()
                .bucket(s3_bucket_name)
                .key(format!("{:0>12}/block.json", block_height))
                .request_payer(aws_sdk_s3::model::RequestPayer::Requester)
                .send()
                .await
            {
                Ok(response) => {
                    match response.body.collect().await {
                        Ok(bytes_stream) => break bytes_stream.into_bytes(),
                        Err(err) => {
                            tracing::debug!(
                                target: crate::LAKE_FRAMEWORK,
                                "Failed to read bytes from the block #{:0>12} response. Retrying immediately.\n{:#?}",
                                block_height,
                                err,
                            );
                        }
                    };
                }
                Err(err) => {
                    tracing::debug!(
                        target: crate::LAKE_FRAMEWORK,
                        "Failed to get {:0>12}/block.json. Retrying immediately\n{:#?}",
                        block_height,
                        err
                    );
                }
            };
        };

        serde_json::from_slice::<near_indexer_primitives::views::BlockView>(body_bytes.as_ref())?
    };

    let fetch_shards_futures = (0..block_view.chunks.len() as u64)
        .collect::<Vec<u64>>()
        .into_iter()
        .map(|shard_id| fetch_shard_or_retry(s3_client, s3_bucket_name, block_height, shard_id));

    let shards = futures::future::try_join_all(fetch_shards_futures).await?;

    Ok(near_indexer_primitives::StreamerMessage {
        block: block_view,
        shards,
    })
}

/// Fetches the shard data JSON from AWS S3 and returns the `IndexerShard`
async fn fetch_shard_or_retry(
    s3_client: &Client,
    s3_bucket_name: &str,
    block_height: crate::types::BlockHeight,
    shard_id: u64,
) -> Result<
    near_indexer_primitives::IndexerShard,
    crate::types::LakeError<aws_sdk_s3::error::GetObjectError>,
> {
    loop {
        match s3_client
            .get_object()
            .bucket(s3_bucket_name)
            .key(format!("{:0>12}/shard_{}.json", block_height, shard_id))
            .request_payer(aws_sdk_s3::model::RequestPayer::Requester)
            .send()
            .await
        {
            Ok(response) => {
                let body_bytes = match response.body.collect().await {
                    Ok(body) => body.into_bytes(),
                    Err(err) => {
                        tracing::debug!(
                            target: crate::LAKE_FRAMEWORK,
                            "Failed to read the {:0>12}/shard_{}.json. Retrying in 1s...\n {:#?}",
                            block_height,
                            shard_id,
                            err,
                        );
                        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                        continue;
                    }
                };

                break Ok(serde_json::from_slice::<
                    near_indexer_primitives::IndexerShard,
                >(body_bytes.as_ref())?);
            }
            Err(err) => {
                tracing::debug!(
                    target: crate::LAKE_FRAMEWORK,
                    "Failed to fetch shard #{}, retrying immediately\n{:#?}",
                    shard_id,
                    err
                );
            }
        }
    }
}
