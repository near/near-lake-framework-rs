//! A simple example of how to use the Lake Framework
//! This indexer will listen to the NEAR blockchain and print the block height of each block

use near_lake_framework::near_lake_primitives;

fn main() -> anyhow::Result<()> {
    eprintln!("Starting...");
    // Lake Framework start boilerplate
    near_lake_framework::LakeBuilder::default()
        .testnet()
        .start_block_height(112205773)
        .build()?
        .run(handle_block) // developer-defined async function that handles each block
}

async fn handle_block(
    block: near_lake_primitives::block::Block,
    _ctx: near_lake_framework::LakeContext,
) -> anyhow::Result<()> {
    println!("Block {:?}", block.block_height());

    Ok(())
}
