use super::client::FastNearClient;
use super::types;

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

/// Fetches streamer_message by finality from the fastenar
/// Returns `near_indexer_primitives::StreamerMessage`
pub async fn fetch_streamer_message_by_finality(
    client: &FastNearClient,
    finality: near_indexer_primitives::types::Finality,
) -> near_indexer_primitives::StreamerMessage {
    let finality_str = match finality {
        near_indexer_primitives::types::Finality::Final
        | near_indexer_primitives::types::Finality::DoomSlug => "final",
        near_indexer_primitives::types::Finality::None => "optimistic",
    };
    client
        .fetch_until_success::<near_indexer_primitives::StreamerMessage>(&format!(
            "/v0/last_block/{}",
            finality_str
        ))
        .await
        .expect("Failed to fetch streamer_message by finality")
}

/// Fetches the optimistic block from the fastenar
/// This function is used to fetch the optimistic block by height
/// This function will be using endpoint `/v0/block_opt/:block_height`
/// This would be waiting some time until the optimistic block is available
/// Returns `near_indexer_primitives::StreamerMessage` if the block is available
/// Returns `None` if the block height is skipped
pub async fn fetch_optimistic_streamer_message_by_height(
    client: &FastNearClient,
    block_height: types::BlockHeight,
) -> Option<near_indexer_primitives::StreamerMessage> {
    client
        .fetch_until_success(&format!("/v0/block_opt/{}", block_height))
        .await
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

/// Fetches block by finality from the fastenar
/// Returns `near_indexer_primitives::views::BlockView`
pub async fn fetch_block_by_finality(
    client: &FastNearClient,
    finality: near_indexer_primitives::types::Finality,
) -> near_indexer_primitives::views::BlockView {
    let finality_str = match finality {
        near_indexer_primitives::types::Finality::Final
        | near_indexer_primitives::types::Finality::DoomSlug => "final",
        near_indexer_primitives::types::Finality::None => "optimistic",
    };
    client
        .fetch_until_success::<near_indexer_primitives::views::BlockView>(&format!(
            "/v0/last_block/{}/headers",
            finality_str
        ))
        .await
        .expect("Failed to fetch block by finality")
}

/// Fetches block by finality from the fastenar
/// Returns `near_indexer_primitives::views::BlockView`
pub async fn fetch_optimistic_block_by_height(
    client: &FastNearClient,
    block_height: types::BlockHeight,
) -> Option<near_indexer_primitives::views::BlockView> {
    client
        .fetch_until_success::<near_indexer_primitives::views::BlockView>(&format!(
            "/v0/block_opt/{}/headers",
            block_height
        ))
        .await
}

/// Fetches the block from the fastenar by block height
/// Returns the result in `near_indexer_primitives::views::BlockView`
/// If the block does not exist, returns an error
pub async fn fetch_block(
    client: &FastNearClient,
    block_height: types::BlockHeight,
) -> Result<near_indexer_primitives::views::BlockView, types::FastNearError> {
    client
        .fetch::<near_indexer_primitives::views::BlockView>(&format!(
            "/v0/block/{}/headers",
            block_height
        ))
        .await?
        .ok_or_else(|| {
            types::FastNearError::BlockDoesNotExist(format!(
                "Block {} does not exist",
                block_height
            ))
        })
}

/// Fetches the block from the fastenar by block height
/// Returns the result in `near_indexer_primitives::views::BlockView`
/// If the block does not exist, retries fetching the block
pub async fn fetch_block_or_retry(
    client: &FastNearClient,
    block_height: types::BlockHeight,
) -> Result<near_indexer_primitives::views::BlockView, types::FastNearError> {
    client
        .fetch_until_success::<near_indexer_primitives::views::BlockView>(&format!(
            "/v0/block/{}/headers",
            block_height
        ))
        .await
        .ok_or_else(|| {
            types::FastNearError::BlockDoesNotExist(format!(
                "Block {} does not exist",
                block_height
            ))
        })
}

/// Fetches the shard from the fastenar by block height and shard id
/// Returns the result in `near_indexer_primitives::IndexerShard`
/// If the block does not exist, returns an error
pub async fn fetch_shard(
    client: &FastNearClient,
    block_height: types::BlockHeight,
    shard_id: u64,
) -> Result<near_indexer_primitives::IndexerShard, types::FastNearError> {
    client
        .fetch::<near_indexer_primitives::IndexerShard>(&format!(
            "/v0/block/{}/shard/{}",
            block_height, shard_id
        ))
        .await?
        .ok_or_else(|| {
            types::FastNearError::BlockDoesNotExist(format!(
                "Block {} and shard {} does not exist",
                block_height, shard_id
            ))
        })
}

/// Fetches the shard from the fastenar by block height and shard id
/// Returns the result in `near_indexer_primitives::IndexerShard`
/// If the block does not exist, retries fetching the block
pub async fn fetch_shard_or_retry(
    client: &FastNearClient,
    block_height: types::BlockHeight,
    shard_id: u64,
) -> Result<near_indexer_primitives::IndexerShard, types::FastNearError> {
    client
        .fetch_until_success::<near_indexer_primitives::IndexerShard>(&format!(
            "/v0/block/{}/shard/{}",
            block_height, shard_id
        ))
        .await
        .ok_or_else(|| {
            types::FastNearError::BlockDoesNotExist(format!(
                "Block {} and shard {} does not exist",
                block_height, shard_id
            ))
        })
}

/// Fetches the chunk from the fastenar by block height and shard id
/// Returns the result in `near_indexer_primitives::IndexerChunkView`
/// If the block does not exist, returns an error
pub async fn fetch_chunk(
    client: &FastNearClient,
    block_height: types::BlockHeight,
    shard_id: u64,
) -> Result<near_indexer_primitives::IndexerChunkView, types::FastNearError> {
    client
        .fetch::<near_indexer_primitives::IndexerChunkView>(&format!(
            "/v0/block/{}/chunk/{}",
            block_height, shard_id
        ))
        .await?
        .ok_or_else(|| {
            types::FastNearError::BlockDoesNotExist(format!(
                "Block {} and shard {} does not exist",
                block_height, shard_id
            ))
        })
}

/// Fetches the shard from the fastenar by block height and shard id
/// Returns the result in `near_indexer_primitives::IndexerShard`
/// If the block does not exist, retries fetching the block
pub async fn fetch_chunk_or_retry(
    client: &FastNearClient,
    block_height: types::BlockHeight,
    shard_id: u64,
) -> Result<near_indexer_primitives::IndexerShard, types::FastNearError> {
    client
        .fetch_until_success::<near_indexer_primitives::IndexerShard>(&format!(
            "/v0/block/{}/chunk/{}",
            block_height, shard_id
        ))
        .await
        .ok_or_else(|| {
            types::FastNearError::BlockDoesNotExist(format!(
                "Block {} and chunk {} does not exist",
                block_height, shard_id
            ))
        })
}
