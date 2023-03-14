# near-lake-framework-rs

Available in programming languages: **Rust** | [Javascript](https://github.com/near/near-lake-framework-js)

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
       .testnet()
       .start_block_height(82422587)
       .build()
       .expect("Failed to build LakeConfig");

   // instantiate the NEAR Lake Framework Stream
   let (sender, stream) = near_lake_framework::streamer(config);

   // read the stream events and pass them to a handler function with
   // concurrency 1
   let mut handlers = tokio_stream::wrappers::ReceiverStream::new(stream)
       .map(|streamer_message| handle_streamer_message(streamer_message))
       .buffer_unordered(1usize);

   while let Some(_handle_message) = handlers.next().await {}
   drop(handlers); // close the channel so the sender will stop

   // propagate errors from the sender
   match sender.await {
       Ok(Ok(())) => Ok(()),
       Ok(Err(e)) => Err(e),
       Err(e) => Err(anyhow::Error::from(e)), // JoinError
   }
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

For more information [refer to the docs](https://docs.rs/near-lake-framework)

### Tutorials

- Video tutorial about [`near-examples/near-lake-accounts-watcher`](https://github.com/near-examples/near-lake-accounts-watcher) https://youtu.be/GsF7I93K-EQ
- [Migrating to NEAR Lake Framework](https://near-indexers.io/tutorials/lake/migrating-to-near-lake-framework) from [NEAR Indexer Framework](https://near-indexers.io/docs/projects/near-indexer-framework)

### More examples

- [`near-examples/near-lake-raw-printer`](https://github.com/near-examples/near-lake-raw-printer) simple example of a data printer built on top of NEAR Lake Framework
- [`near-examples/near-lake-accounts-watcher`](https://github.com/near-examples/near-lake-accounts-watcher) another simple example of the indexer built on top of NEAR Lake Framework for a tutorial purpose
- [`near-examples/indexer-tx-watcher-example-lake`](https://github.com/near-examples/indexer-tx-watcher-example-lake) an example of the indexer built on top of NEAR Lake Framework that watches for transactions related to specified account(s)
- [`octopus-network/octopus-near-indexer-s3`](https://github.com/octopus-network/octopus-near-indexer-s3) a community-made project that uses NEAR Lake Framework

## How to use

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
near-lake-framework = "0.6.1"
```

## Cost estimates (Updated Mar 10, 2023 with more precise calculations)

**TL;DR** approximately $20 per month (for AWS S3 access, paid directly to AWS) for the reading of fresh blocks

### Historical indexing

| Blocks | GET | LIST | Subtotal GET | Subtotal LIST | Total $ |
|---|---|---|---|---|---|
| 1000 | 5000 | 4 | 0.00215 | 0.0000216 | $0.00 |
| 86,400 | 432000 | 345.6 | 0.18576 | 0.00186624 | $0.19 |
| 2,592,000 | 12960000 | 10368 | 5.5728 | 0.0559872 | $5.63 |
| 77,021,059 | 385105295 | 308084.236 | 165.5952769 | 1.663654874 | $167.26 |

**Note:** ~77m of blocks is the number of blocks on the moment I was calculating.

**84,400 blocks is approximate number of blocks per day** (1 block per second * 60 seconds * 60 minutes * 24 hours)

**2,592,000 blocks is approximate number of blocks per months** (86,400 blocks per day * 30 days)

### Tip of the network indexing

| Blocks | GET | LIST | Subtotal GET | Subtotal LIST | Total $ |
|---|---|---|---|---|---|
| 1000 | 5000 | 1000 | 0.00215 | 0.0054 | $0.01 |
| 86,400 | 432000 | 86,400 | 0.18576 | 0.46656 | $0.65 |
| 2,592,000 | 12960000 | 2,592,000 | 5.5728 | 13.9968 | $19.57 |
| 77,021,059 | 385105295 | 77,021,059 | 165.5952769 | 415.9137186 | $581.51 |

Explanation:

Assuming NEAR Protocol produces accurately 1 block per second (which is really not, the average block production time is 1.3s). A full day consists of 86400 seconds, that's the max number of blocks that can be produced.

According the [Amazon S3 prices](https://aws.amazon.com/s3/pricing/?nc1=h_ls) `list` requests are charged for $0.0054 per 1000 requests and `get` is charged for $0.00043 per 1000 requests.

Calculations (assuming we are following the tip of the network all the time):

```
86400 blocks per day * 5 requests for each block / 1000 requests * $0.0004 per 1k requests = $0.19 * 30 days = $5.7
```
**Note:** 5 requests for each block means we have 4 shards (1 file for common block data and 4 separate files for each shard)

And a number of `list` requests we need to perform for 30 days:

```
86400 blocks per day / 1000 requests * $0.005 per 1k list requests = $0.47 * 30 days = $14.1

$5.7 + $14.1 = $19.8
```

The price depends on the number of shards

## Future plans

We use Milestones with clearly defined acceptance criteria:

* [x] [MVP](https://github.com/near/near-lake-framework/milestone/1)
* [ ] [0.8 High-level update](https://github.com/near/near-lake-framework-rs/milestone/3)
* [ ] [1.0](https://github.com/near/near-lake-framework/milestone/2)
