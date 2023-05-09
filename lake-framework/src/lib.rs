#![doc = include_str!("../README.md")]
#[macro_use]
extern crate derive_builder;

pub use async_trait;

use futures::{Future, StreamExt};

pub use near_lake_primitives::{self, near_indexer_primitives, LakeContext};

pub use aws_credential_types::Credentials;
pub use types::{Lake, LakeBuilder, LakeError, LakeMiddleware};

mod s3_fetchers;
mod streamer;
pub(crate) mod types;

pub(crate) const LAKE_FRAMEWORK: &str = "near_lake_framework";

impl types::Lake {
    /// Creates `mpsc::channel` and returns the `receiver` to read the stream of `StreamerMessage`
    ///```no_run
    ///  struct MyContext {
    ///      my_field: String,
    ///  }
    ///# fn main() -> anyhow::Result<()> {
    ///
    ///    let context = MyContext {
    ///       my_field: "my_value".to_string(),
    ///    };
    ///
    ///    near_lake_framework::LakeBuilder::default()
    ///        .testnet()
    ///        .start_block_height(112205773)
    ///        .build()?
    ///        .run_with_context(handle_block, &context)?;
    ///    Ok(())
    ///# }
    ///
    /// # async fn handle_block(_block: near_lake_primitives::block::Block, context: &MyContext) -> anyhow::Result<()> { Ok(()) }
    ///```
    pub fn run_with_context<'context, C, E, Fut>(
        mut self,
        indexer_function: impl Fn(near_lake_primitives::block::Block, &'context C) -> Fut,
        context: &'context C,
    ) -> Result<(), LakeError>
    where
        Fut: Future<Output = Result<(), E>>,
        E: Into<Box<dyn std::error::Error>>,
    {
        let runtime = tokio::runtime::Runtime::new()
            .map_err(|err| LakeError::RuntimeStartError { error: err })?;

        runtime.block_on(async move {
            // capture the concurrency since we need to pass it to the handler
            let concurrency = self.concurrency;

            // capture the middlewares if any
            let middlewares = self.middlewares.take();

            // instantiate the NEAR Lake Framework Stream
            let (sender, stream) = streamer::streamer(self);

            // read the stream events and pass them to a handler function with custom concurrency (default 1)
            let mut handlers = tokio_stream::wrappers::ReceiverStream::new(stream)
                .map(|streamer_message| async {
                    let block: near_lake_primitives::block::Block = streamer_message.into();
                    let mut block = block;

                    // Applying middlewares if any in the order they were added
                    if let Some(middlewares) = middlewares.as_ref() {
                        if !middlewares.is_empty() {
                            for middleware in middlewares {
                                block = middleware.process(block).await;
                            }
                        }
                    }

                    indexer_function(block, context).await
                })
                .buffer_unordered(concurrency);

            while let Some(_handle_message) = handlers.next().await {}
            drop(handlers); // close the channel so the sender will stop

            // propagate errors from the sender
            match sender.await {
                Ok(Ok(())) => Ok(()),
                Ok(Err(err)) => Err(err),
                Err(err) => Err(err.into()), // JoinError
            }
        })
    }

    /// Creates `mpsc::channel` and returns the `receiver` to read the stream of `StreamerMessage`
    ///```no_run
    ///# fn main() -> anyhow::Result<()> {
    ///    near_lake_framework::LakeBuilder::default()
    ///        .testnet()
    ///        .start_block_height(112205773)
    ///        .build()?
    ///        .run(handle_block)?;
    ///    Ok(())
    ///# }
    ///
    /// # async fn handle_block(_block: near_lake_primitives::block::Block) -> anyhow::Result<()> { Ok(()) }
    ///```
    pub fn run<Fut, E>(
        self,
        indexer_function: impl Fn(near_lake_primitives::block::Block) -> Fut,
    ) -> Result<(), LakeError>
    where
        Fut: Future<Output = Result<(), E>>,
        E: Into<Box<dyn std::error::Error>>,
    {
        self.run_with_context(|block, _context| indexer_function(block), &())
    }
}
