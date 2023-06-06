pub mod actions;
pub mod block;
pub mod delegate_actions;
pub mod events;
mod impl_actions;
pub mod receipts;
pub mod state_changes;
pub mod transactions;

/// Since both [transactions::Transaction] hash and [receipts::Receipt] id are the [crate::CryptoHash] type,
/// we use this type alias to make the code more readable.
pub type ReceiptId = near_indexer_primitives::CryptoHash;
