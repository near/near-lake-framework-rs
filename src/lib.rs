//! # NEAR Lake Framework
//!
//! NEAR Lake Framework is a small library companion to [NEAR Lake](https://github.com/near/near-lake). It allows you to build
//! your own indexer that subscribes to the stream of blocks from the NEAR Lake data source and create your own logic to process
//! the NEAR Protocol data.

//! ## Example

//! ```rust
//! use futures::StreamExt;
//! use near_lake_framework::LakeConfigBuilder;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), tokio::io::Error> {
//!    // create a NEAR Lake Framework config
//!    let config = LakeConfigBuilder::default()
//!        .testnet()
//!        .start_block_height(82422587)
//!        .build()
//!        .expect("Failed to build LakeConfig");
//!
//!    // instantiate the NEAR Lake Framework Stream
//!    let stream = near_lake_framework::streamer(config);
//!
//!    // read the stream events and pass them to a handler function with
//!    // concurrency 1
//!    let mut handlers = tokio_stream::wrappers::ReceiverStream::new(stream)
//!        .map(|streamer_message| handle_streamer_message(streamer_message))
//!        .buffer_unordered(1usize);
//!
//!    while let Some(_handle_message) = handlers.next().await {}
//!
//!    Ok(())
//!}
//!
//! // The handler function to take the entire `StreamerMessage`
//! // and print the block height and number of shards
//! async fn handle_streamer_message(
//!    streamer_message: near_lake_framework::near_indexer_primitives::StreamerMessage,
//! ) {
//!    eprintln!(
//!        "{} / shards {}",
//!        streamer_message.block.header.height,
//!        streamer_message.shards.len()
//!    );
//!}
//!```
//!
//! ## Video tutorial:
//!
//! <https://youtu.be/GsF7I93K-EQ>
//!
//! ### More examples
//!
//! - <https://github.com/near-examples/near-lake-raw-printer> simple example of a data printer built on top of NEAR Lake Framework
//! - <https://github.com/near-examples/near-lake-accounts-watcher> another simple example of the indexer built on top of NEAR Lake Framework for a tutorial purpose
//!
//! - <https://github.com/near-examples/indexer-tx-watcher-example-lake> an example of the indexer built on top of NEAR Lake Framework that watches for transactions related to specified account(s)
//! - <https://github.com/octopus-network/octopus-near-indexer-s3> a community-made project that uses NEAR Lake Framework
//!
//! ## How to use
//!
//! ### AWS S3 Credentials
//!
//! In order to be able to get objects from the AWS S3 bucket you need to provide the AWS credentials.
//! #### Passing credentials to the config builder
//!
//! ```rust
//! use near_lake_framework::LakeConfigBuilder;
//!
//! # async fn main() {
//! let credentials = aws_types::Credentials::new(
//!     "AKIAIOSFODNN7EXAMPLE",
//!     "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY",
//!     None,
//!     None,
//!     "custom_credentials",
//! );
//! let s3_config = aws_sdk_s3::Config::builder()
//!     .credentials_provider(credentials)
//!     .build();
//!
//! let config = LakeConfigBuilder::default()
//!      .s3_config(s3_config)
//!      .s3_bucket_name("near-lake-data-custom")
//!      .start_block_height(1)
//!      .build()
//!      .expect("Failed to build LakeConfig");
//! # }
//! ```
//!
//! **You should never hardcode your credentials, it is insecure. Use the described method to pass the credentials you read from CLI arguments**
//!
//! #### File-based AWS credentials
//!AWS default profile configuration with aws configure looks similar to the following:
//!
//!`~/.aws/credentials`
//!```
//![default]
//!aws_access_key_id=AKIAIOSFODNN7EXAMPLE
//!aws_secret_access_key=wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY
//!```
//!
//![AWS docs: Configuration and credential file settings](https://docs.aws.amazon.com/cli/latest/userguide/cli-configure-files.html)
//!
//!### Dependencies
//!
//!Add the following dependencies to your `Cargo.toml`
//!
//!```toml
//!...
//![dependencies]
//!futures = "0.3.5"
//!itertools = "0.10.3"
//!tokio = { version = "1.1", features = ["sync", "time", "macros", "rt-multi-thread"] }
//!tokio-stream = { version = "0.1" }
//!
//!# NEAR Lake Framework
//!near-lake-framework = "0.3.0"
//!```
//!
//! ### Custom S3 storage
//!
//! In case you want to run your own [near-lake](https://github.com/near/near-lake) instance and store data in some S3 compatible storage ([Minio](https://min.io/) or [Localstack](https://localstack.cloud/) as example)
//! You can owerride default S3 API endpoint by using `s3_endpoint` option
//!
//! - run minio
//!
//! ```bash
//! $ mkdir -p /data/near-lake-custom && minio server /data
//! ```
//!
//! - pass custom `aws_sdk_s3::config::Config` to the [LakeConfigBuilder]
//!
//! ```rust
//! use aws_sdk_s3::Endpoint;
//! use http::Uri;
//! use near_lake_framework::LakeConfigBuilder;
//!
//! # async fn main() {
//! let aws_config = aws_config::from_env().load().await;
//! let mut s3_conf = aws_sdk_s3::config::Builder::from(&aws_config);
//! s3_conf = s3_conf
//!     .endpoint_resolver(
//!             Endpoint::immutable("http://0.0.0.0:9000".parse::<Uri>().unwrap()))
//!     .build();
//!
//! let config = LakeConfigBuilder::default()
//!     .s3_config(s3_conf)
//!     .s3_bucket_name("near-lake-data-custom")
//!     .start_block_height(1)
//!     .build()
//!     .expect("Failed to build LakeConfig");
//! # }
//! ```
//!
//! ## Configuration
//!
//! Everything should be configured before the start of your indexer application via `LakeConfigBuilder` struct.
//!
//! Available parameters:
//!
//! * [`start_block_height(value: u64)`](LakeConfigBuilder::start_block_height) - block height to start the stream from
//! * *optional* [`s3_bucket_name(value: impl Into<String>)`](LakeConfigBuilder::s3_bucket_name) - provide the AWS S3 bucket name (you need to provide it if you use custom S3-compatible service, otherwise you can use [LakeConfigBuilder::mainnet] and [LakeConfigBuilder::testnet])
//! * *optional* [`LakeConfigBuilder::s3_region_name(value: impl Into<String>)`](LakeConfigBuilder::s3_region_name) - provide the AWS S3 region name (if you need to set a custom one)
//! * *optional* [`LakeConfigBuilder::s3_config(value: aws_sdk_s3::config::Config`](LakeConfigBuilder::s3_config) - provide custom AWS SDK S3 Config
//!
//! ## Cost estimates
//!
//! **TL;DR** approximately $18.15 per month (for AWS S3 access, paid directly to AWS) for the reading of fresh blocks
//!
//! Explanation:
//!
//! Assuming NEAR Protocol produces accurately 1 block per second (which is really not, the average block production time is 1.3s). A full day consists of 86400 seconds, that's the max number of blocks that can be produced.
//!
//! According the [Amazon S3 prices](https://aws.amazon.com/s3/pricing/?nc1=h_ls) `list` requests are charged for $0.005 per 1000 requests and `get` is charged for $0.0004 per 1000 requests.
//!
//! Calculations (assuming we are following the tip of the network all the time):
//!
//! ```text
//! 86400 blocks per day * 5 requests for each block / 1000 requests * $0.0004 per 1k requests = $0.173 * 30 days = $5.19
//! ```
//! **Note:** 5 requests for each block means we have 4 shards (1 file for common block data and 4 separate files for each shard)
//!
//! And a number of `list` requests we need to perform for 30 days:
//!
//! ```text
//! 86400 blocks per day / 1000 requests * $0.005 per 1k list requests = $0.432 * 30 days = $12.96
//!
//! $5.19 + $12.96 = $18.15
//!```
//!
//! The price depends on the number of shards
//!
//! ## Future plans
//!
//! We use Milestones with clearly defined acceptance criteria:
//!
//! * [x] [MVP](https://github.com/near/near-lake-framework/milestone/1)
//! * [ ] [1.0](https://github.com/near/near-lake-framework/milestone/2)
use aws_sdk_s3::Client;

#[macro_use]
extern crate derive_builder;

use futures::stream::{StreamExt as _};
use tokio_stream::{Stream as _};
use tokio::sync::mpsc;
use tokio::sync::mpsc::error::SendError;

pub use near_indexer_primitives;

pub use aws_types::Credentials;
pub use types::{LakeConfig, LakeConfigBuilder};

mod s3_fetchers;
pub(crate) mod types;

pub(crate) const LAKE_FRAMEWORK: &str = "near_lake_framework";

/// Creates `mpsc::channel` and returns the `receiver` to read the stream of `StreamerMessage`
/// ```
/// use near_lake_framework::LakeConfigBuilder;
/// use tokio::sync::mpsc;
///
/// # async fn main() {
///    let config = LakeConfigBuilder::default()
///        .testnet()
///        .start_block_height(82422587)
///        .build()
///        .expect("Failed to build LakeConfig");
///
///     let (_, stream) = near_lake_framework::streamer(config);
///
///     while let Some(streamer_message) = stream.recv().await {
///         eprintln!("{:#?}", streamer_message);
///     }
/// # }
/// ```
pub fn streamer(
    config: LakeConfig,
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
            tracing::warn!("awaiting list of blocks");
            match s3_fetchers::list_blocks(
                &s3_client,
                &s3_bucket_name,
                start_from_block_height,
                number_of_blocks_requested
            )
            .await {
                Ok(block_heights) => {
                    if block_heights.is_empty() {
                        tracing::debug!(
                            target: LAKE_FRAMEWORK,
                            "There are no newer block heights than {} in bucket {}. Fetching again in 2s...",
                            start_from_block_height,
                            s3_bucket_name,
                        );
                        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                        continue;
                    }
                    tracing::debug!(
                        target: LAKE_FRAMEWORK,
                        "Received {} newer block heights",
                        block_heights.len()
                    );

                    start_from_block_height = *block_heights.last().unwrap() + 1;
                    for block_height in block_heights {
                        tracing::warn!("yield {} block height", block_height);
                        yield block_height;
                    }
                }
                Err(err) => {
                    tracing::warn!(
                        target: LAKE_FRAMEWORK,
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

async fn start(
    streamer_message_sink: mpsc::Sender<near_indexer_primitives::StreamerMessage>,
    config: LakeConfig,
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
        for _ in 0..config.blocks_preload_pool_size {
            tracing::warn!("awaiting next block height (1)");
            match futures::poll!(pending_block_heights.next()) {
                std::task::Poll::Ready(Some(block_height)) => {
                    streamer_messages_futures.push_back(s3_fetchers::fetch_streamer_message(
                        &s3_client,
                        &config.s3_bucket_name,
                        block_height,
                    ));
                }
                std::task::Poll::Ready(None) => {
                    return Err(anyhow::anyhow!("This state should be unreachable as the block heights stream should be infinite."));
                }
                std::task::Poll::Pending => {
                    if streamer_messages_futures.is_empty() {
                        match tokio_stream::StreamExt::next(&mut pending_block_heights).await {
                            Some(block_height) => {
                                streamer_messages_futures.push_back(
                                    s3_fetchers::fetch_streamer_message(
                                        &s3_client,
                                        &config.s3_bucket_name,
                                        block_height,
                                    ),
                                );
                            }
                            None => {
                                return Err(anyhow::anyhow!("This state should be unreachable as the block heights stream should be infinite."));
                            }
                        }
                        continue;
                    }
                    // There are no more block heights at the moment, so we should not block here
                    // and start processing the blocks.
                    break;
                }
            }
        }

        tracing::warn!("awaiting block (2)");
        while let Some(streamer_message_result) = futures::StreamExt::next(&mut streamer_messages_futures).await {
            tracing::warn!("received block (3)");
            let streamer_message = streamer_message_result?;
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
            // store current block info as `last_processed_block_*` for next iteration
            last_processed_block_hash = Some(streamer_message.block.header.hash);
            start_from_block_height = streamer_message.block.header.height + 1;
            tracing::warn!("awaiting next block height (4)");
            let blocks_preload_pool_current_len = streamer_messages_futures.len();
            for _ in blocks_preload_pool_current_len..config.blocks_preload_pool_size {
                tracing::warn!("awaiting next block height (41)");
                match futures::poll!(pending_block_heights.next()) {
                    std::task::Poll::Ready(Some(block_height)) => {
                        streamer_messages_futures.push_back(s3_fetchers::fetch_streamer_message(
                            &s3_client,
                            &config.s3_bucket_name,
                            block_height,
                        ));
                    }
                    std::task::Poll::Ready(None) => {
                        return Err(anyhow::anyhow!("This state should be unreachable as the block heights stream should be infinite."));
                    }
                    std::task::Poll::Pending => {
                        if streamer_messages_futures.is_empty() {
                            match tokio_stream::StreamExt::next(&mut pending_block_heights).await {
                                Some(block_height) => {
                                    streamer_messages_futures.push_back(
                                        s3_fetchers::fetch_streamer_message(
                                            &s3_client,
                                            &config.s3_bucket_name,
                                            block_height,
                                        ),
                                    );
                                }
                                None => {
                                    return Err(anyhow::anyhow!("This state should be unreachable as the block heights stream should be infinite."));
                                }
                            }
                            continue;
                        }
                        // There are no more block heights at the moment, so we should not block here
                        // and keep processing the blocks.
                        break;
                    }
                }
            }

            tracing::warn!("streaming block (5)");
            if let Err(SendError(_)) = streamer_message_sink.send(streamer_message).await {
                tracing::debug!(target: LAKE_FRAMEWORK, "Channel closed, exiting");
                return Ok(());
            }
        }
    }
}
