use std::str::FromStr;

use aws_sdk_s3::Client;
use futures::stream::StreamExt;

const ESTIMATED_SHARDS_COUNT: usize = 4;

/// Queries the list of the objects in the bucket, grouped by "/" delimiter.
/// Returns the list of blocks that can be fetched
pub(crate) async fn list_blocks(
    s3_client: &Client,
    s3_bucket_name: &str,
    start_from_block_height: crate::types::BlockHeight,
    number_of_blocks_requested: usize,
) -> anyhow::Result<Vec<crate::types::BlockHeight>> {
    tracing::debug!(
        target: crate::LAKE_FRAMEWORK,
        "Fetching blocks from S3, after {}...",
        start_from_block_height
    );
    let response = s3_client
        .list_objects_v2()
        .max_keys((number_of_blocks_requested * (1 + ESTIMATED_SHARDS_COUNT)).try_into()?)
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
) -> anyhow::Result<crate::near_indexer_primitives::StreamerMessage> {
    let block_view = {
        let response = loop {
            match s3_client
                .get_object()
                .bucket(s3_bucket_name)
                .key(format!("{:0>12}/block.json", block_height))
                .request_payer(aws_sdk_s3::model::RequestPayer::Requester)
                .send()
                .await
            {
                Ok(response) => break response,
                Err(err) => {
                    tracing::debug!(
                        target: crate::LAKE_FRAMEWORK,
                        "Failed to get {:0>12}/block.json. Retrying immediately\n{:#?}",
                        block_height,
                        err
                    );
                }
            }
        };

        let body_bytes = response.body.collect().await?.into_bytes();

        serde_json::from_slice::<crate::near_indexer_primitives::views::BlockView>(
            body_bytes.as_ref(),
        )?
    };

    let shards: Vec<crate::near_indexer_primitives::IndexerShard> = (0..block_view.chunks.len()
        as u64)
        .collect::<Vec<u64>>()
        .into_iter()
        .map(|shard_id| fetch_shard_or_retry(s3_client, s3_bucket_name, block_height, shard_id))
        .collect::<futures::stream::FuturesOrdered<_>>()
        .collect()
        .await;

    Ok(crate::near_indexer_primitives::StreamerMessage {
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
) -> crate::near_indexer_primitives::IndexerShard {
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

                let indexer_shard = match serde_json::from_slice::<
                    crate::near_indexer_primitives::IndexerShard,
                >(body_bytes.as_ref())
                {
                    Ok(indexer_shard) => indexer_shard,
                    Err(err) => {
                        tracing::debug!(
                            target: crate::LAKE_FRAMEWORK,
                            "Failed to parse the {:0>12}/shard_{}.json. Retrying in 1s...\n {:#?}",
                            block_height,
                            shard_id,
                            err,
                        );
                        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                        continue;
                    }
                };

                break indexer_shard;
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
