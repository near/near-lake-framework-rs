//! A simple example of how to use the Lake Framework with middlewares
//! This indexer will listen to the NEAR blockchain and print the block height of each block
//! Though the print will be done by the middleware, not the handler

// Using the async_trait re-exported from the framework
use near_lake_framework::async_trait::async_trait;
use near_lake_framework::near_lake_primitives;

#[derive(Debug, Default)]
struct PrinterMiddleware;

#[async_trait]
impl near_lake_framework::LakeMiddleware for PrinterMiddleware {
    async fn process(
        &self,
        block: near_lake_primitives::block::Block,
    ) -> near_lake_primitives::block::Block {
        println!("Block {:?}", block.block_height());
        block
    }
}

fn main() -> anyhow::Result<()> {
    eprintln!("Starting...");
    // Lake Framework start boilerplate
    near_lake_framework::LakeBuilder::default()
        .testnet()
        .start_block_height(112205773)
        .middleware(PrinterMiddleware::default())
        .build()?
        .run(handle_block)?; // developer-defined async function that handles each block
    Ok(())
}

async fn handle_block(_block: near_lake_primitives::block::Block) -> anyhow::Result<()> {
    Ok(())
}
