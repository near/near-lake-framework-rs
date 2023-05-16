# Lake Parent Transaction Cache (Context)

Lake Parent Transaction Cache is a ready-to-use context for the Lake Framework in Rust. It provides a cache for keeping the relation between transactions and receipts in cache.

## Example Usage

```no_run
use lake_parent_transaction_cache::{ParentTransactionCache, ParentTransactionCacheBuilder};
# use near_lake_framework::LakeBuilder;
# use near_lake_framework::near_lake_primitives::{block::Block, actions::ActionMetaDataExt};

# fn main() {
let parent_transaction_cache_ctx = ParentTransactionCacheBuilder::default()
    .build()
    .expect("Failed to build the ParentTransactionCache context");

LakeBuilder::default()
    .mainnet()
    .start_block_height(80504433)
    .build()
    .expect("Failed to build the Lake Framework")
    .run_with_context(handle_block, &parent_transaction_cache_ctx)
    .expect("Failed to run the Lake Framework");
# }

async fn handle_block(
    mut block: Block,
    ctx: &ParentTransactionCache,
) -> anyhow::Result<()> {
    for action in block.actions() {
        println!(
            "Action receipt ID: {:?} | Parent TX hash: {:?}",
            action.receipt_id(),
            ctx.get_parent_transaction_hash(&action.receipt_id())
        );
    }
    Ok(())
}
```

## Getting Started

To use the Lake Parent Transaction Cache context in your Rust project, follow these steps:

1. Add the following dependencies to your `Cargo.toml` file:

```toml
[dependencies]
lake_parent_transaction_cache = "<version>"
```

2. Import the necessary modules in your code:

```ignore
use lake_parent_transaction_cache::ParentTransactionCache;
use near_lake_primitives::actions::ActionMetaDataExt;
```

3. Create an instance of the `ParentTransactionCache` context:

```no_run
# use lake_parent_transaction_cache::ParentTransactionCacheBuilder;
let parent_transaction_cache_ctx = ParentTransactionCacheBuilder::default();
```

4. Configure the Lake Framework and run it with the created context:

```ignore
near_lake_framework::LakeBuilder::default()
    .mainnet()
    .start_block_height(<desired_block_height>)
    .build()?
    .run_with_context(<you_indexing_function>, &parent_transaction_cache_ctx)?;
```

Replace `<desired_block_height>` with the starting block height you want to use. Replace `<you_indexing_function>` with the function you want to use to index the blocks.