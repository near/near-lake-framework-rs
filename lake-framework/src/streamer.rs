use aws_sdk_s3::Client;

use futures::stream::StreamExt;
use tokio::sync::mpsc;
use tokio::sync::mpsc::error::SendError;

use near_lake_primitives::near_indexer_primitives;

use crate::{s3_fetchers, types};

/// Creates [mpsc::Receiver<near_indexer_primitives::StreamerMessage>] and
/// [mpsc::Sender<near_indexer_primitives::StreamerMessage>] spawns the streamer
/// process that writes [near_idnexer_primitives::StreamerMessage] to the given `mpsc::channel`
/// returns both `sender` and `receiver`
pub(crate) fn streamer(
    config: crate::Lake,
) -> (
    tokio::task::JoinHandle<Result<(), anyhow::Error>>,
    mpsc::Receiver<near_indexer_primitives::StreamerMessage>,
) {
    let (sender, receiver) = mpsc::channel(config.blocks_preload_pool_size);
    (tokio::spawn(start(sender, config)), receiver)
}

fn stream_block_heights<'a: 'b, 'b>(
    lake_s3_client: &'a s3_fetchers::LakeS3Client,
    s3_bucket_name: &'a str,
    mut start_from_block_height: crate::types::BlockHeight,
) -> impl futures::Stream<Item = u64> + 'b {
    async_stream::stream! {
        loop {
            tracing::debug!(target: crate::LAKE_FRAMEWORK, "Fetching a list of blocks from S3...");
            match s3_fetchers::list_block_heights(
                lake_s3_client,
                s3_bucket_name,
                start_from_block_height,
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

// The only consumer of the BlockHeights Streamer
async fn prefetch_block_heights_into_pool(
    pending_block_heights: &mut std::pin::Pin<
        &mut impl tokio_stream::Stream<Item = crate::types::BlockHeight>,
    >,
    limit: usize,
    await_for_at_least_one: bool,
) -> anyhow::Result<Vec<crate::types::BlockHeight>> {
    let mut block_heights = Vec::with_capacity(limit);
    for remaining_limit in (0..limit).rev() {
        tracing::debug!(target: crate::LAKE_FRAMEWORK, "Polling for the next block height without awaiting... (up to {} block heights are going to be fetched)", remaining_limit);
        match futures::poll!(pending_block_heights.next()) {
            std::task::Poll::Ready(Some(block_height)) => {
                block_heights.push(block_height);
            }
            std::task::Poll::Pending => {
                if await_for_at_least_one && block_heights.is_empty() {
                    tracing::debug!(target: crate::LAKE_FRAMEWORK, "There were no block heights available immediatelly, and the prefetching blocks queue is empty, so we need to await for at least a single block height to be available before proceeding...");
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
                tracing::debug!(target: crate::LAKE_FRAMEWORK, "There were no block heights available immediatelly, so we should not block here and keep processing the blocks.");
                break;
            }
            std::task::Poll::Ready(None) => {
                return Err(anyhow::anyhow!("This state should be unreachable as the block heights stream should be infinite."));
            }
        }
    }
    Ok(block_heights)
}

#[allow(unused_labels)] // we use loop labels for code-readability
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
    let lake_s3_client = s3_fetchers::LakeS3Client::new(s3_client.clone());

    let mut last_processed_block_hash: Option<near_indexer_primitives::CryptoHash> = None;

    'main: loop {
        // In the beginning of the 'main' loop we create a Block Heights stream
        // and prefetch the initial data in that pool.
        // Later the 'stream' loop might exit to this 'main' one to repeat the procedure.
        // This happens because we assume Lake Indexer that writes to the S3 Bucket might
        // in some cases, write N+1 block before it finishes writing the N block.
        // We require to stream blocks consistently, so we need to try to load the block again.

        let pending_block_heights = stream_block_heights(
            &lake_s3_client,
            &config.s3_bucket_name,
            start_from_block_height,
        );
        tokio::pin!(pending_block_heights);

        let mut streamer_messages_futures = futures::stream::FuturesOrdered::new();
        tracing::debug!(
            target: crate::LAKE_FRAMEWORK,
            "Prefetching up to {} blocks...",
            config.blocks_preload_pool_size
        );

        streamer_messages_futures.extend(
            prefetch_block_heights_into_pool(
                &mut pending_block_heights,
                config.blocks_preload_pool_size,
                true,
            )
            .await?
            .into_iter()
            .map(|block_height| {
                s3_fetchers::fetch_streamer_message(
                    &lake_s3_client,
                    &config.s3_bucket_name,
                    block_height,
                )
            }),
        );

        tracing::debug!(
            target: crate::LAKE_FRAMEWORK,
            "Awaiting for the first prefetched block..."
        );
        'stream: while let Some(streamer_message_result) = streamer_messages_futures.next().await {
            let streamer_message = streamer_message_result.map_err(|err| {
                tracing::error!(
                    target: crate::LAKE_FRAMEWORK,
                    "Failed to fetch StreamerMessage with error: \n{:#?}",
                    err,
                );
                err
            })?;

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
                    break 'stream;
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
            tracing::debug!(
                target: crate::LAKE_FRAMEWORK,
                "Streaming block #{} ({})",
                streamer_message.block.header.height,
                streamer_message.block.header.hash
            );
            let blocks_preload_pool_current_len = streamer_messages_futures.len();

            let prefetched_block_heights_future = prefetch_block_heights_into_pool(
                &mut pending_block_heights,
                config
                    .blocks_preload_pool_size
                    .saturating_sub(blocks_preload_pool_current_len),
                blocks_preload_pool_current_len == 0,
            );

            let streamer_message_sink_send_future = streamer_message_sink.send(streamer_message);

            let (prefetch_res, send_res): (
                Result<Vec<types::BlockHeight>, anyhow::Error>,
                Result<_, SendError<near_indexer_primitives::StreamerMessage>>,
            ) = futures::join!(
                prefetched_block_heights_future,
                streamer_message_sink_send_future,
            );

            if let Err(SendError(err)) = send_res {
                tracing::debug!(
                    target: crate::LAKE_FRAMEWORK,
                    "Failed to send StreamerMessage (#{:0>12}) to the channel. Channel is closed, exiting \n{:?}",
                    start_from_block_height - 1,
                    err,
                );
                return Ok(());
            }

            streamer_messages_futures.extend(
                prefetch_res
                    .map_err(|err| {
                        tracing::error!(
                            target: crate::LAKE_FRAMEWORK,
                            "Failed to prefetch block heights to the prefetching pool with error: \n{:#?}",
                            err
                        );
                        err
                    })?
                    .into_iter()
                    .map(|block_height| {
                        s3_fetchers::fetch_streamer_message(
                            &lake_s3_client,
                            &config.s3_bucket_name,
                            block_height,
                        )
                    }
            ));
        }

        tracing::warn!(
            target: crate::LAKE_FRAMEWORK,
            "Exited from the 'stream' loop. It may happen in two cases:\n
            1. Blocks has ended (impossible, might be an error on the Lake Buckets),\n
            2. Received a Block which prev_hash doesn't match the previously streamed block.\n
            Will attempt to restart the stream from block #{}",
            start_from_block_height,
        );
    }
}
