use aws_sdk_s3::Client;
use futures::stream::StreamExt;

/// Queries the list of the objects in the bucket, grouped by "/" delimiter.
/// Returns the continuation token along with the so called list of folder names
/// that represent a block heights
pub(crate) async fn list_blocks(
    s3_client: &Client,
    s3_bucket_name: &str,
    start_from_block_height: String,
) -> anyhow::Result<Vec<String>> {
    let response = s3_client
        .list_objects_v2()
        .max_keys(1000)
        .delimiter("/".to_string())
        .start_after(start_from_block_height)
        .bucket(s3_bucket_name)
        .send()
        .await?;

    Ok(match response.common_prefixes() {
        None => vec![],
        Some(common_prefixes) => common_prefixes
            .iter()
            .filter_map(|common_prefix| common_prefix.prefix.clone())
            .collect(),
    })
}

/// By the given block height (`block_height_prefix`) gets the objects:
/// - block.json
/// - shard_N.json
/// Reads the content of the objects and parses it to JSON.
/// Returns the result in a single JSON
pub(crate) async fn fetch_streamer_message(
    s3_client: &Client,
    s3_bucket_name: &str,
    block_height_prefix: &str,
) -> anyhow::Result<near_indexer_primitives::StreamerMessage> {
    let block_json: serde_json::Value = {
        let response = loop {
            if let Ok(response) = s3_client
                .get_object()
                .bucket(s3_bucket_name)
                .key(format!("{}block.json", block_height_prefix))
                .send()
                .await
            {
                break response;
            }
        };

        let body_bytes = response.body.collect().await.unwrap().into_bytes();

        serde_json::from_slice(body_bytes.as_ref()).unwrap()
    };

    let shards_num: u64 = block_json["header"]["chunks_included"].as_u64().unwrap();

    let mut main_json = serde_json::json!({
        "block": block_json,
    });

    let mut shards: Vec<serde_json::Value> = vec![];

    let mut shards_futures: futures::stream::FuturesOrdered<_> = (0..shards_num)
        .collect::<Vec<u64>>()
        .into_iter()
        .map(|shard_id| {
            fetch_shard_or_retry(s3_client, s3_bucket_name, block_height_prefix, shard_id)
        })
        .collect();

    while let Some(shard) = shards_futures.next().await {
        shards.push(shard.unwrap());
    }

    main_json["shards"] = serde_json::Value::Array(shards);

    Ok(serde_json::from_value::<near_indexer_primitives::StreamerMessage>(main_json).unwrap())
}

async fn fetch_shard_or_retry(
    s3_client: &Client,
    s3_bucket_name: &str,
    block_height_prefix: &str,
    shard_id: u64,
) -> anyhow::Result<serde_json::Value> {
    loop {
        if let Ok(response) = s3_client
            .get_object()
            .bucket(s3_bucket_name)
            .key(format!("{}shard_{}.json", block_height_prefix, shard_id))
            .send()
            .await
        {
            let body_bytes = response.body.collect().await.unwrap().into_bytes();

            break Ok(serde_json::from_slice(body_bytes.as_ref()).unwrap());
        };
    }
}
