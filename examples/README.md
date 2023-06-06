# NEAR Lake Framework Examples

This directory contains several example scripts showcasing the usage of the NEAR Lake Framework. Each example demonstrates different aspects and features of the framework. Below is a brief description of each example:

## simple.rs

A simple example of how to use the Lake Framework. This indexer will listen to the NEAR blockchain and print the block height of each block.

```bash
$ cd lake-framework
$ cargo run --example simple
```

## actions.rs

This example shows how to filter actions in a block. It is a more real-life example than the simple example. It is going to follow the NEAR Social contract and print all function calls to it.

```bash
$ cd lake-framework
$ cargo run --example actions
```

## nft_indexer.rs

This is a more complex real-life example of how to use the NEAR Lake Framework.

It is going to follow the network and watch for the Events according to the [Events Format][1]. It will monitor for nft_mint events from the known marketplaces, such as Mintbase and Paras, and index them to print in the terminal.

[1]: https://nomicon.io/Standards/EventsFormat

```bash
$ cd lake-framework
$ cargo run --example nft_indexer
```

## with_context.rs

This example show how to use a context with Lake Framework. It is going to follow the NEAR Social contract and the block height along with a number of calls to the contract.

```bash
$ cd lake-framework
$ cargo run --example with_context
```

## with_context_parent_tx_cache.rs

This example show how to use a context `ParentTransactionCache` with the Lake Framework. It is going to follow the NEAR Social contract and cache the parent Transaction for the Receipts. Thus we would be able to capture the Transaction where the change to the contract state has started.

```bash
$ cd lake-parent-transaction-cache
$ cargo run --example with_context_parent_tx_cache
```