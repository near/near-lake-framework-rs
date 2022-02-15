use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3::{Client, Region};

use futures::stream::StreamExt;
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
        config.s3_bucket_name,
        config.s3_region_name,
        config.start_block_height,
    ));
    receiver
}

///
async fn start(
    streamer_message_sink: mpsc::Sender<near_indexer_primitives::StreamerMessage>,
    s3_bucket_name: String,
    s3_region_name: String,
    index_from_block_height: u64,
) {
    // instantiate AWS S3 Client
    let region_provider = RegionProviderChain::first_try(Some(s3_region_name).map(Region::new))
        .or_default_provider()
        .or_else(Region::new("eu-central-1"));
    let shared_config = aws_config::from_env().region(region_provider).load().await;
    let s3_client = Client::new(&shared_config);

    let mut start_from_block_height: String = index_from_block_height.to_string();

    // Continuously get the list of block data from S3 and send them to the `streamer_message_sink`
    loop {
        // TODO: decide what to do if we got an error
        if let Ok(block_heights_prefixes) =
            s3_fetchers::list_blocks(&s3_client, &s3_bucket_name, start_from_block_height.clone())
                .await
        {
            // update start_after key
            start_from_block_height = {
                if block_heights_prefixes.is_empty() {
                    start_from_block_height
                } else {
                    block_heights_prefixes[block_heights_prefixes.len() - 1]
                        .split('/')
                        .collect::<Vec<&str>>()
                        .get(0)
                        .map(|s| (s.parse::<u64>().unwrap() + 1).to_string())
                        .unwrap_or_else(|| start_from_block_height)
                }
            };

            let mut streamer_messages_futures: futures::stream::FuturesOrdered<_> =
                block_heights_prefixes
                    .iter()
                    .map(|block_height_prefix| {
                        s3_fetchers::fetch_streamer_message(
                            &s3_client,
                            &s3_bucket_name,
                            block_height_prefix,
                        )
                    })
                    .collect();

            while let Some(streamer_message_result) = streamer_messages_futures.next().await {
                streamer_message_sink
                    .send(streamer_message_result.unwrap())
                    .await
                    .unwrap();
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
