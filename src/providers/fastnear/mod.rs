pub mod client;
pub mod fetchers;
pub mod types;

/// Starts the FastNear provider
/// Fetches the blocks from the FastNear and sends them to the blocks_sink
/// The fetching is done in parallel with multiple threads
/// The number of threads is defined in the FastNearConfig
/// The fetching starts from the start_block_height and continues until the last block
#[allow(unused_labels)] // we use loop labels for code-readability
pub async fn start(
    blocks_sink: tokio::sync::mpsc::Sender<near_indexer_primitives::StreamerMessage>,
    config: types::FastNearConfig,
) -> anyhow::Result<()> {
    let client = client::FastNearClient::new(&config);
    let max_num_threads = config.num_threads;
    let next_sink_block =
        std::sync::Arc::new(std::sync::atomic::AtomicU64::new(config.start_block_height));
    'main: loop {
        // In the beginning of the 'main' loop, we fetch the next block height to start fetching from
        let start_block_height = next_sink_block.load(std::sync::atomic::Ordering::SeqCst);
        let next_fetch_block =
            std::sync::Arc::new(std::sync::atomic::AtomicU64::new(start_block_height));
        let last_block_height = fetchers::fetch_last_block(&client)
            .await
            .block
            .header
            .height;
        let is_backfill = last_block_height > start_block_height + max_num_threads;
        let num_threads = if is_backfill { max_num_threads } else { 1 };
        tracing::info!(
            target: crate::LAKE_FRAMEWORK,
            "Start fetching from block {} to block {} with {} threads. Backfill: {:?}",
            start_block_height,
            last_block_height,
            num_threads,
            is_backfill
        );
        // starting backfill with multiple threads
        let handles = (0..num_threads)
            .map(|thread_index| {
                let client = client.clone();
                let blocks_sink = blocks_sink.clone();
                let next_fetch_block = next_fetch_block.clone();
                let next_sink_block = next_sink_block.clone();
                tokio::spawn(async move {
                    'stream: loop {
                        let block_height = next_fetch_block.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                        if is_backfill && block_height > last_block_height {
                            break 'stream;
                        }
                        tracing::debug!(target: crate::LAKE_FRAMEWORK, "#{}: Fetching block: {}", thread_index, block_height);
                        let block =
                            fetchers::fetch_streamer_message(&client, block_height).await;
                        'sender: loop {
                            let expected_block_height = next_sink_block.load(std::sync::atomic::Ordering::SeqCst);
                            if expected_block_height < block_height {
                                tokio::time::sleep(std::time::Duration::from_millis(
                                    block_height - expected_block_height,
                                )).await;
                            } else {
                                tracing::debug!(target: crate::LAKE_FRAMEWORK, "#{}: Sending block: {}", thread_index, block_height);
                                break 'sender;
                            }
                        }
                        if let Some(block) = block {
                            blocks_sink.send(block).await.expect("Failed to send block");
                        } else {
                            tracing::debug!(target: crate::LAKE_FRAMEWORK, "#{}: Skipped block: {}", thread_index, block_height);
                        }
                        next_sink_block.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    }
                })
            })
            .collect::<Vec<_>>();
        for handle in handles {
            handle.await?;
        }
    }
}
