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
//! ### FastNear provider
//!
//! FastNear provider is a new way to get the data from NEAR Protocol. It is a separate service that provides the data in a more efficient way.
//!
//! How to use it:
//!
//! ## Example
//!
//! ```rust
//! use futures::StreamExt;
//! use near_lake_framework::FastNearConfigBuilder;
//!
//! #[tokio::main]
//! # async fn main() {
//!    let config = FastNearConfigBuilder::default()
//!        .testnet()
//!        .start_block_height(82422587)
//!        .build()
//!        .expect("Failed to build LakeConfig");
//!
//!     let (_, stream) = near_lake_framework::streamer(config);
//!
//!     while let Some(streamer_message) = stream.recv().await {
//!         eprintln!("{:#?}", streamer_message);
//!     }
//! # }
//! ```
//!
//! How to migrate from Lake to FastNear:
//!
//! - Replace `LakeConfigBuilder` with `FastNearConfigBuilder`
//! - Replace `LakeConfig` with `FastNearConfig`
//!
//! ## Future plans
//!
//! We use Milestones with clearly defined acceptance criteria:
//!
//! * [x] [MVP](https://github.com/near/near-lake-framework/milestone/1)
//! * [ ] [0.8 High-level update](https://github.com/near/near-lake-framework-rs/milestone/3)
//! * [ ] [1.0](https://github.com/near/near-lake-framework/milestone/2)

#[macro_use]
extern crate derive_builder;

pub use near_indexer_primitives;

pub use aws_credential_types::Credentials;

mod providers;

pub use providers::fastnear;
pub use providers::s3;

pub use providers::fastnear::client::FastNearClient;
pub use providers::fastnear::types::{FastNearConfig, FastNearConfigBuilder};

pub use providers::s3::client::LakeS3Client;
pub use providers::s3::types::{LakeConfig, LakeConfigBuilder};

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
pub fn streamer<T: Into<providers::NearLakeFrameworkConfig>>(
    config: T,
) -> (
    tokio::task::JoinHandle<Result<(), anyhow::Error>>,
    tokio::sync::mpsc::Receiver<near_indexer_primitives::StreamerMessage>,
) {
    let config: providers::NearLakeFrameworkConfig = config.into();
    let (sender, receiver) = tokio::sync::mpsc::channel(config.blocks_preload_pool_size());
    match config {
        providers::NearLakeFrameworkConfig::Lake(config) => {
            (tokio::spawn(s3::start(sender, *config)), receiver)
        }
        providers::NearLakeFrameworkConfig::FastNear(config) => {
            (tokio::spawn(fastnear::start(sender, config)), receiver)
        }
    }
}
