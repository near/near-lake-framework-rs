use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3::{Client, Region, Endpoint};

use futures::stream::StreamExt;
use http::Uri;
use tokio::sync::mpsc;

pub use near_indexer_primitives;

pub use types::LakeConfig;

mod s3_fetchers;
pub(crate) mod types;

pub(crate) const LAKE_FRAMEWORK: &str = "near_lake_framework";

pub fn streamer(config: LakeConfig) -> mpsc::Receiver<near_indexer_primitives::StreamerMessage> {
    let (sender, receiver) = mpsc::channel(100);
    tokio::spawn(start(
        sender,
        config.s3_endpoint,
        config.s3_bucket_name,
        config.s3_region_name,
        config.start_block_height,
    ));
    receiver
}

///
async fn start(
    streamer_message_sink: mpsc::Sender<near_indexer_primitives::StreamerMessage>,
    s3_endpoint: Option<String>,
    s3_bucket_name: String,
    s3_region_name: String,
    index_from_block_height: types::BlockHeight,
) {
    // instantiate AWS S3 Client
    let region_provider = RegionProviderChain::first_try(Some(s3_region_name).map(Region::new))
        .or_default_provider()
        .or_else(Region::new("eu-central-1"));
    let shared_config = aws_config::from_env().region(region_provider).load().await;
    let mut s3_conf = aws_sdk_s3::config::Builder::from(&shared_config);
    // Owerride S3 endpoint in case you want to use custom solution
    // like Minio or Localstack as a S3 compatible storage
    if let Some(endpoint) = s3_endpoint {
        s3_conf = s3_conf.endpoint_resolver(Endpoint::immutable(endpoint.parse::<Uri>().unwrap()));
        tracing::info!(
            target: LAKE_FRAMEWORK,
            "Custom S3 endpoint used: {}",
            endpoint
        );
    }
    let s3_client = Client::from_conf(s3_conf.build());

    let mut start_from_block_height = index_from_block_height;
    let mut last_processed_block_hash: Option<near_indexer_primitives::CryptoHash> = None;

    // Continuously get the list of block data from S3 and send them to the `streamer_message_sink`
    loop {
        if let Ok(block_heights_prefixes) =
            s3_fetchers::list_blocks(&s3_client, &s3_bucket_name, start_from_block_height).await
        {
            if block_heights_prefixes.is_empty() {
                tracing::debug!(
                    target: LAKE_FRAMEWORK,
                    "No new blocks on S3, retry in 2s..."
                );
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                continue;
            }
            tracing::debug!(
                target: LAKE_FRAMEWORK,
                "Received {} blocks from S3",
                block_heights_prefixes.len()
            );
            let mut streamer_messages_futures: futures::stream::FuturesOrdered<_> =
                block_heights_prefixes
                    .iter()
                    .map(|block_height| {
                        s3_fetchers::fetch_streamer_message(
                            &s3_client,
                            &s3_bucket_name,
                            *block_height,
                        )
                    })
                    .collect();

            while let Some(streamer_message_result) = streamer_messages_futures.next().await {
                let streamer_message =
                    streamer_message_result.expect("Failed to unwrap StreamerMessage from Result");
                // check if we have `last_processed_block_hash` (might be None only on start)
                if let Some(prev_block_hash) = last_processed_block_hash {
                    // compare last_processed_block_hash` with `block.header.prev_hash` of the current
                    // block (ensure we don't miss anything from S3)
                    // retrieve the data from S3 if prev_hashes don't match and repeat the main loop step
                    if prev_block_hash != streamer_message.block.header.prev_hash {
                        tracing::warn!(
                            target: LAKE_FRAMEWORK,
                            "`prev_hash` does not match, refetching the data from S3 in 200ms",
                        );
                        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                        break;
                    }
                }
                // store current block hash as `last_processed_block_hash` for next iteration
                last_processed_block_hash = Some(streamer_message.block.header.hash);
                // update start_after key
                start_from_block_height = streamer_message.block.header.height + 1;
                streamer_message_sink.send(streamer_message).await.unwrap();
            }
        } else {
            tracing::error!(
                target: LAKE_FRAMEWORK,
                "Failed to list objects from bucket {}. Retrying...",
                &s3_bucket_name
            );
        }
    }
}
