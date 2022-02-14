use aws_sdk_s3::{Client, Error};

/// Queries the list of the objects in the bucket, grouped by "/" delimiter.
/// Returns the continuation token along with the so called list of folder names
/// that represent a block heights
pub(crate) async fn list_blocks(
    client: &Client,
    bucket: &str,
    start_after_block_height: Option<String>,
    continuation_token: Option<String>,
) -> Result<crate::types::ListObjectResponse, Error> {
    let response = client
        .list_objects_v2()
        .max_keys(1000)
        .delimiter("/".to_string())
        .set_start_after(start_after_block_height)
        .set_continuation_token(continuation_token)
        .bucket(bucket)
        .send()
        .await?;

    let continuation_token = response.next_continuation_token().map(ToOwned::to_owned);
    let folder_names = match response.common_prefixes() {
        None => vec![],
        Some(common_prefixes) => common_prefixes
            .into_iter()
            .filter_map(|common_prefix| common_prefix.prefix.clone())
            .collect(),
    };

    Ok(crate::types::ListObjectResponse {
        continuation_token,
        folder_names,
    })
}

/// By the given block height (`key`) gets the objects:
/// - block.json
/// - shard_N.json
/// Reads the content of the objects and parses it to JSON.
/// Returns the result in a single JSON
pub(crate) async fn get_object(
    client: &Client,
    bucket: &str,
    key: &str,
) -> Result<serde_json::Value, Error> {
    let mut main_json = serde_json::json!({});
    let block_json: serde_json::Value = {
        let response = client
            .get_object()
            .bucket(bucket)
            .key(format!("{}block.json", key))
            .send()
            .await?;

        let body_bytes = response.body.collect().await.unwrap().into_bytes();

        serde_json::from_slice(body_bytes.as_ref()).unwrap()
    };

    let shards_num: u64 = block_json["header"]["chunks_included"].as_u64().unwrap();

    main_json["block"] = block_json;

    let mut shards: Vec<serde_json::Value> = vec![];

    for shard_id in 0..shards_num {
        let response = client
            .get_object()
            .bucket(bucket)
            .key(format!("{}shard_{}.json", key, shard_id))
            .send()
            .await?;

        let body_bytes = response.body.collect().await.unwrap().into_bytes();

        shards.push(serde_json::from_slice(body_bytes.as_ref()).unwrap());
    }

    main_json["shards"] = serde_json::Value::Array(shards);

    Ok(main_json)
}
