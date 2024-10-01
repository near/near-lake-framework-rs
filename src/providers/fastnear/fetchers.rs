use super::client::FastNearClient;
use super::types;

/// Fetches the last block from the fastenar
/// Returns `near_indexer_primitives::StreamerMessage`
pub async fn fetch_last_block(client: &FastNearClient) -> near_indexer_primitives::StreamerMessage {
    client
        .fetch_until_success("/v0/last_block/final")
        .await
        .expect("Failed to fetch last block")
}

/// Fetches the optimistic block from the fastenar
/// Returns `near_indexer_primitives::StreamerMessage`
pub async fn fetch_optimistic_block(
    client: &FastNearClient,
) -> near_indexer_primitives::StreamerMessage {
    client
        .fetch_until_success("/v0/last_block/optimistic")
        .await
        .expect("Failed to fetch optimistic block")
}

/// Fetches the genesis block from the fastenar
/// Returns `near_indexer_primitives::StreamerMessage`
pub async fn fetch_first_block(
    client: &FastNearClient,
) -> near_indexer_primitives::StreamerMessage {
    client
        .fetch_until_success("/v0/first_block")
        .await
        .expect("Failed to fetch first block")
}

/// Fetches the block data from the fastenar by block height
/// Returns the result in `Option<near_indexer_primitives::StreamerMessage>`
/// If the block does not exist, returns `None`
pub async fn fetch_streamer_message(
    client: &FastNearClient,
    block_height: types::BlockHeight,
) -> Option<near_indexer_primitives::StreamerMessage> {
    client
        .fetch_until_success(&format!("/v0/block/{}", block_height))
        .await
}

/// Fetches the block from the fastenar by block height
/// Returns the result in `near_indexer_primitives::views::BlockView`
/// If the block does not exist, returns an error
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

/// Fetches the block from the fastenar by block height
/// Returns the result in `near_indexer_primitives::views::BlockView`
/// If the block does not exist, retries fetching the block
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

/// Fetches the shard from the fastenar by block height and shard id
/// Returns the result in `near_indexer_primitives::IndexerShard`
/// If the block does not exist, returns an error
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

/// Fetches the shard from the fastenar by block height and shard id
/// Returns the result in `near_indexer_primitives::IndexerShard`
/// If the block does not exist, retries fetching the block
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
