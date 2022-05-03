# near-lake-framework

NEAR Lake Framework is a small library companion to [NEAR Lake](https://github.com/near/near-lake). It allows you to build
your own indexer that subscribes to the stream of blocks from the NEAR Lake data source and create your own logic to process
the NEAR Protocol data.

[![crates.io](https://img.shields.io/crates/v/near-lake-framework?label=latest)](https://crates.io/crates/near-lake-framework)
[![Documentation](https://docs.rs/near-lake-framework/badge.svg)](https://docs.rs/near-lake-framework)
![MIT or Apache 2.0 licensed](https://img.shields.io/crates/l/near-lake-framework.svg)

---

[Official NEAR Lake Framework launch announcement](https://gov.near.org/t/announcement-near-lake-framework-brand-new-word-in-indexer-building-approach/17668) has been published on the NEAR Gov Forum
Greetings from the Data Platform Team! We are happy and proud to announce an MVP release of a brand new word in indexer building approach - NEAR Lake Framework.

---

## Example

```rust
use futures::StreamExt;
use near_lake_framework::LakeConfigBuilder;

#[tokio::main]
async fn main() -> Result<(), tokio::io::Error> {
    // create a NEAR Lake Framework config
    let config = LakeConfigBuilder::default()
        .s3_bucket_name("near-lake-data-testnet")
        .s3_region_name("eu-west-1")
        .start_block_height(82422587)
        .build()
        .expect("Failed to build Lakeconfig");

    // instantiate the NEAR Lake Framework Stream
    let stream = near_lake_framework::streamer(config);

    // read the stream events and pass them to a handler function with
    // concurrency 1
    let mut handlers = tokio_stream::wrappers::ReceiverStream::new(stream)
        .map(|streamer_message| handle_streamer_message(streamer_message))
        .buffer_unordered(1usize);

    while let Some(_handle_message) = handlers.next().await {}

    Ok(())
}

// The handler function to take the entire `StreamerMessage`
// and print the block height and number of shards
async fn handle_streamer_message(
    streamer_message: near_lake_framework::near_indexer_primitives::StreamerMessage,
) {
    eprintln!(
        "{} / shards {}",
        streamer_message.block.header.height,
        streamer_message.shards.len()
    );
}
```

Video tutorial:

https://youtu.be/GsF7I93K-EQ

### More examples

- https://github.com/near-examples/near-lake-raw-printer simple example of a data printer built on top of NEAR Lake Framework
- https://github.com/near-examples/near-lake-accounts-watcher another simple example of the indexer built on top of NEAR Lake Framework for a tutorial purpose

## How to use

## Custom S3 storage

In case you want to run your own [near-lake](https://github.com/near/near-lake) instance and store data in some S3 compatible storage ([Minio](https://min.io/) or [Localstack](https://localstack.cloud/) as example)
You can owerride default S3 API endpoint by using `s3_endpoint` option

- run minio

```bash
$ mkdir -p /data/near-lake-custom && minio server /data
```

- add `s3_endpoint` parameter to LakeConfig instance

```bash
let config = LakeConfigBuilder::default()
        .s3_bucket_name("near-lake-custom")
        .start_block_height(1)
        .custom_endpoint("http://0.0.0.0:9000")
        .build()
        .expect("Failed to build Lakeconfig");
```

### AWS S3 Credentials

In order to be able to get objects from the AWS S3 bucket you need to provide the AWS credentials.

#### Passing credentials to the config builder

```rust
let config = LakeConfigBuilder::default()
 .s3_bucket_name("near-lake-testnet")
 .s3_region_name("eu-central-1")
 .start_block_height(82422587)
 .custom_credentials(
      "AKIAIOSFODNN7EXAMPLE",
      "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY",
  )
 .build()
 .expect("Failed to build LakeConfig");
```
**You should never hardcode your credentials, it is insecure. Use the described method to pass the credentials you read from CLI arguments**

#### File-based AWS credentials

AWS default profile configuration with aws configure looks similar to the following:

`~/.aws/credentials`
```
[default]
aws_access_key_id=AKIAIOSFODNN7EXAMPLE
aws_secret_access_key=wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY
```

[AWS docs: Configuration and credential file settings](https://docs.aws.amazon.com/cli/latest/userguide/cli-configure-files.html)

### Dependencies

Add the following dependencies to your `Cargo.toml`

```toml
...
[dependencies]
futures = "0.3.5"
itertools = "0.10.3"
tokio = { version = "1.1", features = ["sync", "time", "macros", "rt-multi-thread"] }
tokio-stream = { version = "0.1" }

# NEAR Lake Framework
near-lake-framework = "0.3.0"
```

## Configuration

Everything should be configured before the start of your indexer application via `LakeConfigBuilder` struct.

Available parameters:

- `s3_bucket_name(value: impl Into<String>)` - provide the AWS S3 bucket name (`near-lake-testnet`, `near-lake-mainnet` or yours if you run your own NEAR Lake)
- `start_block_height(value: u64)` - block height to start the stream from
- *optional* `with_custom_endpoint(value: impl Into<String>)` - provide the AWS S3 custom API ednpoint
- *optional* `s3_region_name(value: impl Into<String>)` - provide the region for AWS S3 bucket
- *optional* `with_custom_credentials(access_key_id: impl Into<String>, secret_access_key: impl Into<String>)` - provide custom credentials to AWS

## Cost estimates

**TL;DR** approximately $18.15 per month (for AWS S3 access, paid directly to AWS) for the reading of fresh blocks

Explanation:

Assuming NEAR Protocol produces accurately 1 block per second (which is really not, the average block production time is 1.3s). A full day consists of 86400 seconds, that's the max number of blocks that can be produced.

According the [Amazon S3 prices](https://aws.amazon.com/s3/pricing/?nc1=h_ls) `list` requests are charged for $0.005 per 1000 requests and `get` is charged for $0.0004 per 1000 requests.

Calculations (assuming we are following the tip of the network all the time):

```
86400 blocks per day * 5 requests for each block / 1000 requests * $0.0004 per 1k requests = $0.173 * 30 days = $5.19
```
**Note:** 5 requests for each block means we have 4 shards (1 file for common block data and 4 separate files for each shard)

And a number of `list` requests we need to perform for 30 days:

```
86400 blocks per day / 1000 requests * $0.005 per 1k list requests = $0.432 * 30 days = $12.96

$5.19 + $12.96 = $18.15
```

The price depends on the number of shards

## Future plans

We use Milestones with clearly defined acceptance criteria:

* [x] [MVP](https://github.com/near/near-lake-framework/milestone/1)
* [ ] [1.0](https://github.com/near/near-lake-framework/milestone/2)
