use aws_sdk_s3::Client;
use futures::stream::StreamExt;

/// Queries the list of the objects in the bucket, grouped by "/" delimiter.
/// Returns the list of blocks that can be fetched
pub(crate) async fn list_blocks(
    s3_client: &Client,
    s3_bucket_name: &str,
    start_from_block_height: crate::types::BlockHeight,
) -> anyhow::Result<Vec<crate::types::BlockHeight>> {
    let response = s3_client
        .list_objects_v2()
        .max_keys(1000)
        .delimiter("/".to_string())
        .start_after(start_from_block_height.to_string())
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
            .map(|prefix_string| {
                prefix_string
                    .split('/')
                    .next()
                    .map(|s| s.parse::<u64>().expect("Failed to parse u64 from prefix"))
                    .expect("Failed to unwrap Option<u64>")
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
) -> anyhow::Result<near_indexer_primitives::StreamerMessage> {
    let block_view = {
        let response = loop {
            if let Ok(response) = s3_client
                .get_object()
                .bucket(s3_bucket_name)
                .key(format!("{}/block.json", block_height))
                .send()
                .await
            {
                break response;
            }
        };

        let body_bytes = response.body.collect().await.unwrap().into_bytes();

        serde_json::from_slice::<near_indexer_primitives::views::BlockView>(body_bytes.as_ref())
            .unwrap()
    };

    let shards: Vec<near_indexer_primitives::IndexerShard> = (0..block_view.header.chunks_included)
        .collect::<Vec<u64>>()
        .into_iter()
        .map(|shard_id| fetch_shard_or_retry(s3_client, s3_bucket_name, block_height, shard_id))
        .collect::<futures::stream::FuturesOrdered<_>>()
        .collect()
        .await;

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
) -> near_indexer_primitives::IndexerShard {
    loop {
        if let Ok(response) = s3_client
            .get_object()
            .bucket(s3_bucket_name)
            .key(format!("{}/shard_{}.json", block_height, shard_id))
            .send()
            .await
        {
            let body_bytes = response.body.collect().await.unwrap().into_bytes();

            break serde_json::from_slice::<near_indexer_primitives::IndexerShard>(
                body_bytes.as_ref(),
            )
            .unwrap();
        };
    }
}
