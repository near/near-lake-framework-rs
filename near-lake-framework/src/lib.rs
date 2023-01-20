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
#[macro_use]
extern crate derive_builder;

use futures::{Future, StreamExt};

pub use near_lake_primitives::{self, near_indexer_primitives, LakeContext};

pub use aws_types::Credentials;
pub use types::{Lake, LakeBuilder};

mod s3_fetchers;
mod streamer;
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
impl types::Lake {
    pub fn run<Fut>(
        self,
        f: impl Fn(near_lake_primitives::block::Block, near_lake_primitives::LakeContext) -> Fut,
    ) -> anyhow::Result<()>
    where
        Fut: Future<Output = anyhow::Result<()>>,
    {
        let runtime = tokio::runtime::Runtime::new()?;

        runtime.block_on(async {
            // instantiate the NEAR Lake Framework Stream
            let (sender, stream) = streamer::streamer(self);

            // read the stream events and pass them to a handler function with
            // concurrency 1
            let mut handlers = tokio_stream::wrappers::ReceiverStream::new(stream)
                .map(|streamer_message| async {
                    let context = LakeContext {};
                    let block: near_lake_primitives::block::Block = streamer_message.into();
                    f(block, context).await
                })
                .buffer_unordered(1usize);

            while let Some(_handle_message) = handlers.next().await {}
            drop(handlers); // close the channel so the sender will stop

            // propagate errors from the sender
            match sender.await {
                Ok(Ok(())) => Ok(()),
                Ok(Err(e)) => Err(e),
                Err(e) => Err(anyhow::Error::from(e)), // JoinError
            }
        })
    }
}
