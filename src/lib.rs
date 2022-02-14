use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3::{Client, Region};

use futures::stream::StreamExt;
use tokio::sync::mpsc;

pub use near_indexer_primitives;

pub use types::LakeConfig;

mod s3_fetchers;
pub(crate) mod types;

const LAKE_FRAMEWORK: &str = "near_lake_framework";

pub fn streamer(config: LakeConfig) -> mpsc::Receiver<near_indexer_primitives::StreamerMessage> {
    let (sender, receiver) = mpsc::channel(16);
    tokio::spawn(start(
        sender,
        config.bucket,
        config.region,
        config.start_block_height,
    ));
    receiver
}

///
async fn start(
    file_sink: mpsc::Sender<near_indexer_primitives::StreamerMessage>,
    bucket: String,
    region: String,
    start_from_block_height: u64,
) {
    // instantiate AWS S3 Client
    let region_provider = RegionProviderChain::first_try(Some(region).map(Region::new))
        .or_default_provider()
        .or_else(Region::new("eu-central-1"));
    let shared_config = aws_config::from_env().region(region_provider).load().await;
    let client = Client::new(&shared_config);

    // `list_block` returns `continuation_token` which needs to be provided
    // on the next call to get the next portion of objects (pagination)
    let mut continuation_token: Option<String> = None;

    // Continuously get the list of block data from S3 and send them to the `file_sink`
    loop {
        // TODO: decide what to do if we got an error
        if let Ok(list_object_response) = s3_fetchers::list_blocks(
            &client,
            &bucket,
            Some(start_from_block_height.to_string()),
            continuation_token.clone(),
        )
        .await
        {
            // update the token for the next iter (pagination)
            continuation_token = list_object_response.continuation_token;

            let mut blocks_futures: futures::stream::FuturesOrdered<_> = list_object_response
                .folder_names
                .iter()
                .map(|folder| {
                    build_and_send_streamer_message(&file_sink, &client, &bucket, &folder)
                })
                .collect();

            while let Some(_response) = blocks_futures.next().await {}
        } else {
            tracing::error!(
                target: LAKE_FRAMEWORK,
                "Failed to list objects from bucket {}. Retrying...",
                &bucket
            );
        }
    }
}

/// Fetches the necessary data to build up the `StreamerMessage`
/// and send it to the stream (`file_sink`)
async fn build_and_send_streamer_message(
    file_sink: &mpsc::Sender<near_indexer_primitives::StreamerMessage>,
    client: &Client,
    bucket: &str,
    folder: &str,
) {
    let streamer_message_json = s3_fetchers::get_object(&client, &bucket, &folder)
        .await
        .unwrap();
    if let Ok(streamer_message) =
        serde_json::from_value::<near_indexer_primitives::StreamerMessage>(streamer_message_json)
    {
        file_sink.send(streamer_message).await.unwrap();
    }
}
