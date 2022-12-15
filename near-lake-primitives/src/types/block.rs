use super::receipts;
use crate::near_indexer_primitives::{types::AccountId, CryptoHash, StreamerMessage};

#[derive(Debug, Clone)]
pub struct Block {
    header: BlockHeader,
    executed_receipts: Vec<receipts::ExecutedReceipt>,
}

impl Block {
    pub fn header(&self) -> &BlockHeader {
        &self.header
    }

    pub fn receipts(&self) -> &[receipts::ExecutedReceipt] {
        &self.executed_receipts
    }
}

impl From<&StreamerMessage> for Block {
    fn from(streamer_message: &StreamerMessage) -> Self {
        Self {
            header: streamer_message.into(),
            executed_receipts: streamer_message
                .shards
                .iter()
                .flat_map(|shard| shard.receipt_execution_outcomes.iter())
                .map(Into::into)
                .collect(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BlockHeader {
    pub height: u64,
    pub hash: CryptoHash,
    pub prev_hash: CryptoHash,
    pub author: AccountId,
}

impl From<&StreamerMessage> for BlockHeader {
    fn from(streamer_message: &StreamerMessage) -> Self {
        Self {
            height: streamer_message.block.header.height,
            hash: streamer_message.block.header.hash,
            prev_hash: streamer_message.block.header.prev_hash,
            author: streamer_message.block.author.clone(),
        }
    }
}
