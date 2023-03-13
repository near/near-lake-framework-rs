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
//!    let (sender, stream) = near_lake_framework::streamer(config);
//!
//!    // read the stream events and pass them to a handler function with
//!    // concurrency 1
//!    let mut handlers = tokio_stream::wrappers::ReceiverStream::new(stream)
//!        .map(|streamer_message| handle_streamer_message(streamer_message))
//!        .buffer_unordered(1usize);
//!
//!    while let Some(_handle_message) = handlers.next().await {}
//!    drop(handlers); // close the channel so the sender will stop
//!
//!    // propagate errors from the sender
//!    match sender.await {
//!        Ok(Ok(())) => Ok(()),
//!        Ok(Err(e)) => Err(e),
//!        Err(e) => Err(anyhow::Error::from(e)), // JoinError
//!    }
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
//! ## Tutorials:
//!
//! - <https://youtu.be/GsF7I93K-EQ>
//! - [Migrating to NEAR Lake Framework](https://near-indexers.io/tutorials/lake/migrating-to-near-lake-framework) from [NEAR Indexer Framework](https://near-indexers.io/docs/projects/near-indexer-framework)
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
//! let credentials = aws_credential_types::Credentials::new(
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
//! ### Environmental variables
//!
//! Alternatively, you can provide your AWS credentials via environment variables with constant names:
//!
//!```
//!$ export AWS_ACCESS_KEY_ID=AKIAIOSFODNN7EXAMPLE
//!$ AWS_SECRET_ACCESS_KEY=wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY
//!$ AWS_DEFAULT_REGION=eu-central-1
//!```
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
//!near-lake-framework = "0.6.1"
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
//! ## Cost estimates (Updated Mar 10, 2022 with more precise calculations)
//!
//! **TL;DR** approximately $20 per month (for AWS S3 access, paid directly to AWS) for the reading of fresh blocks
//!
//! ### Historical indexing
//!
//! | Blocks | GET | LIST | Subtotal GET | Subtotal LIST | Total $ |
//! |---|---|---|---|---|---|
//! | 1000 | 5000 | 4 | 0.00215 | 0.0000216 | $0.00 |
//! | 86,400 | 432000 | 345.6 | 0.18576 | 0.00186624 | $0.19 |
//! | 2,592,000 | 12960000 | 10368 | 5.5728 | 0.0559872 | $5.63 |
//! | 77,021,059 | 385105295 | 308084.236 | 165.5952769 | 1.663654874 | $167.26 |
//!
//! **Note:** ~77m of blocks is the number of blocks on the moment I was calculating.
//!
// !**84,400 blocks is approximate number of blocks per day** (1 block per second * 60 seconds * 60 minutes * 24 hours)
//!
//! **2,592,000 blocks is approximate number of blocks per months** (86,400 blocks per day * 30 days)
//!
//! ### Tip of the network indexing
//!
//! | Blocks | GET | LIST | Subtotal GET | Subtotal LIST | Total $ |
//! |---|---|---|---|---|---|
//! | 1000 | 5000 | 1000 | 0.00215 | 0.0054 | $0.01 |
//! | 86,400 | 432000 | 86,400 | 0.18576 | 0.46656 | $0.65 |
//! | 2,592,000 | 12960000 | 2,592,000 | 5.5728 | 13.9968 | $19.57 |
//! | 77,021,059 | 385105295 | 77,021,059 | 165.5952769 | 415.9137186 | $581.51 |
//!
//! Explanation:
//!
//! Assuming NEAR Protocol produces accurately 1 block per second (which is really not, the average block production time is 1.3s). A full day consists of 86400 seconds, that's the max number of blocks that can be produced.
//!
//! According the [Amazon S3 prices](https://aws.amazon.com/s3/pricing/?nc1=h_ls) `list` requests are charged for $0.0054 per 1000 requests and `get` is charged for $0.00043 per 1000 requests.
//!
//! Calculations (assuming we are following the tip of the network all the time):
//!
//! ```text
//! 86400 blocks per day * 5 requests for each block / 1000 requests * $0.0004 per 1k requests = $0.19 * 30 days = $5.7
//! ```
//! **Note:** 5 requests for each block means we have 4 shards (1 file for common block data and 4 separate files for each shard)
//!
//! And a number of `list` requests we need to perform for 30 days:
//!
//! ```text
//! 86400 blocks per day / 1000 requests * $0.005 per 1k list requests = $0.47 * 30 days = $14.1
//!
//! $5.7 + $14.1 = $19.8
//! ```
//!
//! The price depends on the number of shards
//!
//! ## Future plans
//!
//! We use Milestones with clearly defined acceptance criteria:
//!
//! * [x] [MVP](https://github.com/near/near-lake-framework/milestone/1)
//! * [ ] [0.8 High-level update](https://github.com/near/near-lake-framework-rs/milestone/3)
//! * [ ] [1.0](https://github.com/near/near-lake-framework/milestone/2)
use aws_sdk_s3::Client;

#[macro_use]
extern crate derive_builder;

use futures::stream::StreamExt;
use tokio::sync::mpsc;
use tokio::sync::mpsc::error::SendError;

pub use near_indexer_primitives;

pub use aws_credential_types::Credentials;
pub use types::{LakeConfig, LakeConfigBuilder};

use s3_fetchers::LakeS3Client;

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
    lake_s3_client: &'a LakeS3Client,
    s3_bucket_name: &'a str,
    mut start_from_block_height: crate::types::BlockHeight,
) -> impl futures::Stream<Item = u64> + 'b {
    async_stream::stream! {
        loop {
            tracing::debug!(target: LAKE_FRAMEWORK, "Fetching a list of blocks from S3...");
            match s3_fetchers::list_block_heights(
                lake_s3_client,
                s3_bucket_name,
                start_from_block_height,
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
                        tracing::debug!(target: LAKE_FRAMEWORK, "Yielding {} block height...", block_height);
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
        tracing::debug!(target: LAKE_FRAMEWORK, "Polling for the next block height without awaiting... (up to {} block heights are going to be fetched)", remaining_limit);
        match futures::poll!(pending_block_heights.next()) {
            std::task::Poll::Ready(Some(block_height)) => {
                block_heights.push(block_height);
            }
            std::task::Poll::Pending => {
                if await_for_at_least_one && block_heights.is_empty() {
                    tracing::debug!(target: LAKE_FRAMEWORK, "There were no block heights available immediatelly, and the prefetching blocks queue is empty, so we need to await for at least a single block height to be available before proceeding...");
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
                tracing::debug!(target: LAKE_FRAMEWORK, "There were no block heights available immediatelly, so we should not block here and keep processing the blocks.");
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
            target: LAKE_FRAMEWORK,
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
            target: LAKE_FRAMEWORK,
            "Awaiting for the first prefetched block..."
        );
        'stream: while let Some(streamer_message_result) = streamer_messages_futures.next().await {
            let streamer_message = streamer_message_result.map_err(|err| {
                tracing::error!(
                    target: LAKE_FRAMEWORK,
                    "Failed to fetch StreamerMessage with error: \n{:#?}",
                    err,
                );
                err
            })?;

            tracing::debug!(
                target: LAKE_FRAMEWORK,
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
                        target: LAKE_FRAMEWORK,
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
                target: LAKE_FRAMEWORK,
                "Prefetching up to {} blocks... (there are {} blocks in the prefetching pool)",
                config.blocks_preload_pool_size,
                streamer_messages_futures.len(),
            );
            tracing::debug!(
                target: LAKE_FRAMEWORK,
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
                    target: LAKE_FRAMEWORK,
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
                            target: LAKE_FRAMEWORK,
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
            target: LAKE_FRAMEWORK,
            "Exited from the 'stream' loop. It may happen in two cases:\n
            1. Blocks has ended (impossible, might be an error on the Lake Buckets),\n
            2. Received a Block which prev_hash doesn't match the previously streamed block.\n
            Will attempt to restart the stream from block #{}",
            start_from_block_height,
        );
    }
}
