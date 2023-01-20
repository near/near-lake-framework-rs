use aws_sdk_s3::Client;

use futures::stream::StreamExt;
use tokio::sync::mpsc;
use tokio::sync::mpsc::error::SendError;

use near_lake_primitives::near_indexer_primitives;

/// Creates [mpsc::Receiver<near_indexer_primitives::StreamerMessage>] and
/// [mpsc::Sender<near_indexer_primitives::StreamerMessage>]spawns the streamer
/// process that writes [near_idnexer_primitives::StreamerMessage] to the given `mpsc::channel`
/// returns both `sender` and `receiver`
pub(crate) fn streamer(
    config: crate::types::Lake,
) -> (
    tokio::task::JoinHandle<Result<(), anyhow::Error>>,
    mpsc::Receiver<near_indexer_primitives::StreamerMessage>,
) {
    let (sender, receiver) = mpsc::channel(config.blocks_preload_pool_size);
    (tokio::spawn(start(sender, config)), receiver)
}

fn stream_block_heights<'a: 'b, 'b>(
    s3_client: &'a Client,
    s3_bucket_name: &'a str,
    mut start_from_block_height: crate::types::BlockHeight,
    number_of_blocks_requested: usize,
) -> impl futures::Stream<Item = u64> + 'b {
    async_stream::stream! {
        loop {
            tracing::debug!(target: crate::LAKE_FRAMEWORK, "Fetching a list of blocks from S3...");
            match crate::s3_fetchers::list_blocks(
                s3_client,
                s3_bucket_name,
                start_from_block_height,
                number_of_blocks_requested
            )
            .await {
                Ok(block_heights) => {
                    if block_heights.is_empty() {
                        tracing::debug!(
                            target: crate::LAKE_FRAMEWORK,
                            "There are no newer block heights than {} in bucket {}. Fetching again in 2s...",
                            start_from_block_height,
                            s3_bucket_name,
                        );
                        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                        continue;
                    }
                    tracing::debug!(
                        target: crate::LAKE_FRAMEWORK,
                        "Received {} newer block heights",
                        block_heights.len()
                    );

                    start_from_block_height = *block_heights.last().unwrap() + 1;
                    for block_height in block_heights {
                        tracing::debug!(target: crate::LAKE_FRAMEWORK, "Yielding {} block height...", block_height);
                        yield block_height;
                    }
                }
                Err(err) => {
                    tracing::warn!(
                        target: crate::LAKE_FRAMEWORK,
                        "Failed to get block heights from bucket {}: {}. Retrying in 1s...",
                        s3_bucket_name,
                        err,
                    );
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                }
            }
        }
    }
}

async fn fast_fetch_block_heights(
    pending_block_heights: &mut std::pin::Pin<&mut impl tokio_stream::Stream<Item = u64>>,
    limit: usize,
    await_for_at_least_one: bool,
) -> anyhow::Result<Vec<u64>> {
    let mut block_heights = Vec::with_capacity(limit);
    for remaining_limit in (0..limit).rev() {
        tracing::debug!(
            target: crate::LAKE_FRAMEWORK,
            "Polling for the next block height without awaiting... (up to {} block heights are going to be fetched)",
            remaining_limit
        );
        match futures::poll!(pending_block_heights.next()) {
            std::task::Poll::Ready(Some(block_height)) => {
                block_heights.push(block_height);
            }
            std::task::Poll::Pending => {
                if await_for_at_least_one && block_heights.is_empty() {
                    tracing::debug!(
                        target: crate::LAKE_FRAMEWORK,
                        "There were no block heights available immediately, and the prefetching blocks queue is empty, so we need to await for at least a single block height to be available before proceeding..."
                    );
                    match pending_block_heights.next().await {
                        Some(block_height) => {
                            block_heights.push(block_height);
                        }
                        None => {
                            return Err(anyhow::anyhow!("This state should be unreachable as the block heights stream should be infinite."));
                        }
                    }
                    continue;
                }
                tracing::debug!(
                    target: crate::LAKE_FRAMEWORK,
                    "There were no block heights available immediately, so we should not block here and keep processing the blocks."
                );
                break;
            }
            std::task::Poll::Ready(None) => {
                return Err(anyhow::anyhow!("This state should be unreachable as the block heights stream should be infinite."));
            }
        }
    }
    Ok(block_heights)
}

pub(crate) async fn start(
    streamer_message_sink: mpsc::Sender<near_indexer_primitives::StreamerMessage>,
    config: crate::Lake,
) -> anyhow::Result<()> {
    let mut start_from_block_height = config.start_block_height;

    let s3_client = if let Some(config) = config.s3_config {
        Client::from_conf(config)
    } else {
        let aws_config = aws_config::from_env().load().await;
        let s3_config = aws_sdk_s3::config::Builder::from(&aws_config)
            .region(aws_types::region::Region::new(config.s3_region_name))
            .build();
        Client::from_conf(s3_config)
    };

    let mut last_processed_block_hash: Option<near_indexer_primitives::CryptoHash> = None;

    loop {
        let pending_block_heights = stream_block_heights(
            &s3_client,
            &config.s3_bucket_name,
            start_from_block_height,
            config.blocks_preload_pool_size * 2,
        );
        tokio::pin!(pending_block_heights);

        let mut streamer_messages_futures = futures::stream::FuturesOrdered::new();
        tracing::debug!(
            target: crate::LAKE_FRAMEWORK,
            "Prefetching up to {} blocks...",
            config.blocks_preload_pool_size
        );

        let is_blocks_preload_pool_empty = streamer_messages_futures.is_empty();
        streamer_messages_futures.extend(
            fast_fetch_block_heights(
                &mut pending_block_heights,
                config.blocks_preload_pool_size,
                is_blocks_preload_pool_empty,
            )
            .await?
            .into_iter()
            .map(|block_height| {
                crate::s3_fetchers::fetch_streamer_message(
                    &s3_client,
                    &config.s3_bucket_name,
                    block_height,
                )
            }),
        );

        tracing::debug!(
            target: crate::LAKE_FRAMEWORK,
            "Awaiting for the first prefetched block..."
        );
        while let Some(streamer_message_result) = streamer_messages_futures.next().await {
            let streamer_message = streamer_message_result?;
            tracing::debug!(
                target: crate::LAKE_FRAMEWORK,
                "Received block #{} ({})",
                streamer_message.block.header.height,
                streamer_message.block.header.hash
            );
            // check if we have `last_processed_block_hash` (might be None only on start)
            if let Some(prev_block_hash) = last_processed_block_hash {
                // compare last_processed_block_hash` with `block.header.prev_hash` of the current
                // block (ensure we don't miss anything from S3)
                // retrieve the data from S3 if prev_hashes don't match and repeat the main loop step
                if prev_block_hash != streamer_message.block.header.prev_hash {
                    tracing::warn!(
                        target: crate::LAKE_FRAMEWORK,
                        "`prev_hash` does not match, refetching the data from S3 in 200ms",
                    );
                    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                    break;
                }
            }

            // store current block info as `last_processed_block_*` for next iteration
            last_processed_block_hash = Some(streamer_message.block.header.hash);
            start_from_block_height = streamer_message.block.header.height + 1;

            tracing::debug!(
                target: crate::LAKE_FRAMEWORK,
                "Prefetching up to {} blocks... (there are {} blocks in the prefetching pool)",
                config.blocks_preload_pool_size,
                streamer_messages_futures.len(),
            );

            let blocks_preload_pool_current_len = streamer_messages_futures.len();
            streamer_messages_futures.extend(
                fast_fetch_block_heights(
                    &mut pending_block_heights,
                    config
                        .blocks_preload_pool_size
                        .saturating_sub(blocks_preload_pool_current_len),
                    blocks_preload_pool_current_len == 0,
                )
                .await?
                .into_iter()
                .map(|block_height| {
                    crate::s3_fetchers::fetch_streamer_message(
                        &s3_client,
                        &config.s3_bucket_name,
                        block_height,
                    )
                }),
            );

            tracing::debug!(
                target: crate::LAKE_FRAMEWORK,
                "Streaming block #{} ({})",
                streamer_message.block.header.height,
                streamer_message.block.header.hash
            );
            if let Err(SendError(_)) = streamer_message_sink.send(streamer_message).await {
                tracing::debug!(target: crate::LAKE_FRAMEWORK, "Channel closed, exiting");
                return Ok(());
            }
        }
    }
}
