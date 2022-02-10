use tokio::sync::mpsc;
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3::{Client, Region};

pub use types::LakeConfig;

pub(crate) mod types;
mod s3_fetchers;

pub fn streamer(config: LakeConfig) -> mpsc::Receiver<serde_json::Value> {
    let (sender, receiver) = mpsc::channel(16);
    tokio::spawn(
        start(
            sender,
            config.bucket,
            config.region,
            config.start_block_height,
            config.tracked_shards,
        )
    );
    receiver
}

///
async fn start(
    file_sink: mpsc::Sender<serde_json::Value>,
    bucket: String,
    region: String,
    start_from_block_height: Option<u64>,
    tracked_shards: Vec<u8>,
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

    // Continuosly get the list of block data from S3 and stream send them to the `file_sink`
    loop {
        // TODO: decide what to do if we got an error
        if let Ok(list_object_response) = s3_fetchers::list_blocks(
            &client,
            &bucket,
            start_from_block_height.map(|block_height| block_height.to_string()),
            continuation_token.clone(),
        ).await {
            // update the token for the next iter (pagination)
            continuation_token = list_object_response.continuation_token;

            // read each of the block separately from S3
            for folder in list_object_response.folder_names {
                let block_json = s3_fetchers::get_object(
                    &client,
                    &bucket,
                    &folder,
                    &tracked_shards,
                ).await.unwrap(); // TODO: handle error avoid unwraps
                file_sink.send(block_json).await.unwrap(); // TODO: handle error avoid unwraps
            }
        }
    }
}
