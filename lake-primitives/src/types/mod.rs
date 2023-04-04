pub mod actions;
pub mod block;
pub mod delegate_actions;
pub mod events;
mod impl_actions;
pub mod receipts;
pub mod state_changes;
pub mod transactions;

pub type ReceiptId = near_indexer_primitives::CryptoHash;
