#![doc = include_str!("../README.md")]
#[macro_use]
extern crate derive_builder;

use futures::{Future, StreamExt};

pub use near_lake_primitives::{self, near_indexer_primitives, LakeContext};

pub use aws_credential_types::Credentials;
pub use types::{Lake, LakeBuilder};

mod s3_fetchers;
mod streamer;
pub(crate) mod types;

pub(crate) const LAKE_FRAMEWORK: &str = "near_lake_framework";

/// Creates `mpsc::channel` and returns the `receiver` to read the stream of `StreamerMessage`
///```no_run
///# fn main() -> anyhow::Result<()> {
///    struct MyContext {
///        pub my_field: String,
///    }
///
///    let context = MyContext {
///       my_field: "my_value".to_string(),
///    };
///
///    near_lake_framework::LakeBuilder::default()
///        .testnet()
///        .start_block_height(112205773)
///        .build()?
///        .run_with_context(handle_block, &context)
///# }
///
/// # async fn handle_block(_block: near_lake_primitives::block::Block, context: &MyContext) -> anyhow::Result<()> { Ok(()) }
///```
impl types::Lake {
    pub fn run_with_context<'context, C, Fut>(
        self,
        f: impl Fn(near_lake_primitives::block::Block, &'context C) -> Fut,
        context: &'context C,
    ) -> anyhow::Result<()>
    where
        Fut: Future<Output = anyhow::Result<()>>,
    {
        let runtime = tokio::runtime::Runtime::new()?;

        runtime.block_on(async move {
            // instantiate the NEAR Lake Framework Stream
            let (sender, stream) = streamer::streamer(self);

            // read the stream events and pass them to a handler function with
            // concurrency 1
            let mut handlers = tokio_stream::wrappers::ReceiverStream::new(stream)
                .map(|streamer_message| async {
                    let block: near_lake_primitives::block::Block = streamer_message.into();
                    f(block, &context).await
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

    pub fn run<Fut>(
        self,
        f: impl Fn(near_lake_primitives::block::Block) -> Fut,
    ) -> anyhow::Result<()>
    where
        Fut: Future<Output = anyhow::Result<()>>,
    {
        self.run_with_context(|block, _context| f(block), &())
    }
}
