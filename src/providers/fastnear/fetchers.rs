use super::client::FastNearClient;
use super::types;

pub async fn fetch_last_block(client: &FastNearClient) -> near_indexer_primitives::StreamerMessage {
    client
        .fetch_until_success("/v0/last_block/final")
        .await
        .expect("Failed to fetch last block")
}

pub async fn fetch_first_block(
    client: &FastNearClient,
) -> near_indexer_primitives::StreamerMessage {
    client
        .fetch_until_success("/v0/first_block")
        .await
        .expect("Failed to fetch first block")
}

pub async fn fetch_streamer_message(
    client: &FastNearClient,
    block_height: types::BlockHeight,
) -> Option<near_indexer_primitives::StreamerMessage> {
    client
        .fetch_until_success(&format!("/v0/block/{}", block_height))
        .await
}

pub async fn fetch_block(
    client: &FastNearClient,
    block_height: types::BlockHeight,
) -> Result<near_indexer_primitives::views::BlockView, types::FastNearError> {
    let streamer_message = client.fetch(&format!("/v0/block/{}", block_height)).await?;
    if let Some(msg) = streamer_message {
        Ok(msg.block)
    } else {
        Err(types::FastNearError::BlockDoesNotExist(format!(
            "Block {} does not exist",
            block_height
        )))
    }
}

pub async fn fetch_block_or_retry(
    client: &FastNearClient,
    block_height: types::BlockHeight,
) -> Result<near_indexer_primitives::views::BlockView, types::FastNearError> {
    let streamer_message = client
        .fetch_until_success(&format!("/v0/block/{}", block_height))
        .await;
    if let Some(msg) = streamer_message {
        Ok(msg.block)
    } else {
        Err(types::FastNearError::BlockDoesNotExist(format!(
            "Block {} does not exist",
            block_height
        )))
    }
}

pub async fn fetch_shard(
    client: &FastNearClient,
    block_height: types::BlockHeight,
    shard_id: u64,
) -> Result<near_indexer_primitives::IndexerShard, types::FastNearError> {
    let streamer_message = client.fetch(&format!("/v0/block/{}", block_height)).await?;
    if let Some(msg) = streamer_message {
        Ok(msg
            .shards
            .iter()
            .filter_map(|shard| {
                if shard.shard_id == shard_id {
                    Some(shard.clone())
                } else {
                    None
                }
            })
            .next()
            .ok_or_else(|| {
                types::FastNearError::BlockDoesNotExist(format!(
                    "Block {} and shard {} does not exist",
                    block_height, shard_id
                ))
            })?)
    } else {
        Err(types::FastNearError::BlockDoesNotExist(format!(
            "Block {} does not exist",
            block_height
        )))
    }
}

pub async fn fetch_shard_or_retry(
    client: &FastNearClient,
    block_height: types::BlockHeight,
    shard_id: u64,
) -> Result<near_indexer_primitives::IndexerShard, types::FastNearError> {
    let streamer_message = client
        .fetch_until_success(&format!("/v0/block/{}", block_height))
        .await;
    if let Some(msg) = streamer_message {
        Ok(msg
            .shards
            .iter()
            .filter_map(|shard| {
                if shard.shard_id == shard_id {
                    Some(shard.clone())
                } else {
                    None
                }
            })
            .next()
            .ok_or_else(|| {
                types::FastNearError::BlockDoesNotExist(format!(
                    "Block {} and shard {} does not exist",
                    block_height, shard_id
                ))
            })?)
    } else {
        Err(types::FastNearError::BlockDoesNotExist(format!(
            "Block {} does not exist",
            block_height
        )))
    }
}
