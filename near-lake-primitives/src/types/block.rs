use std::collections::HashMap;

use super::events::{self, EventsTrait};
use super::receipts;
use super::state_changes;
use super::transactions;
use crate::near_indexer_primitives::{types::AccountId, views, CryptoHash, StreamerMessage};

#[derive(Debug)]
pub struct Block {
    streamer_message: StreamerMessage,
    executed_receipts: Vec<receipts::Receipt>,
    postponed_receipts: Vec<receipts::Receipt>,
    transactions: Vec<transactions::Transaction>,
    actions: HashMap<super::ReceiptId, receipts::Action>,
    events: HashMap<super::ReceiptId, Vec<events::Event>>,
    state_changes: Vec<state_changes::StateChange>,
}

impl Block {
    pub fn streamer_message(&self) -> &StreamerMessage {
        &self.streamer_message
    }

    pub fn header(&self) -> BlockHeader {
        (&self.streamer_message).into()
    }

    pub fn receipts(&mut self) -> &[receipts::Receipt] {
        if self.executed_receipts.is_empty() {
            self.executed_receipts = self
                .streamer_message
                .shards
                .iter()
                .flat_map(|shard| shard.receipt_execution_outcomes.iter())
                .map(Into::into)
                .collect();
        }
        &self.executed_receipts
    }

    pub fn postponed_receipts(&mut self) -> &[receipts::Receipt] {
        if self.postponed_receipts.is_empty() {
            self.postponed_receipts = self
                .streamer_message
                .shards
                .iter()
                .filter_map(|shard| shard.chunk.as_ref().map(|chunk| chunk.receipts.iter()))
                .flatten()
                .map(Into::into)
                .collect();
        }
        &self.postponed_receipts
    }

    pub fn transactions(&mut self) -> &[transactions::Transaction] {
        if self.transactions.is_empty() {
            self.transactions = self
                .streamer_message
                .shards
                .iter()
                .filter_map(|shard| shard.chunk.as_ref().map(|chunk| chunk.transactions.iter()))
                .flatten()
                .map(Into::into)
                .collect();
        }
        &self.transactions
    }

    pub fn actions(&self) -> Vec<receipts::Action> {
        self.streamer_message()
            .shards
            .iter()
            .flat_map(|shard| shard.receipt_execution_outcomes.iter())
            .filter_map(|receipt_execution_outcome| {
                receipts::Action::try_from(&receipt_execution_outcome.receipt).ok()
            })
            .collect()
    }

    pub fn events(&mut self) -> Vec<(super::ReceiptId, events::Event)> {
        self.receipts()
            .iter()
            .flat_map(|executed_receipt| {
                executed_receipt.logs.iter().filter_map(|log| {
                    if let Ok(event) = events::Event::from_log(log) {
                        Some((executed_receipt.receipt_id, event))
                    } else {
                        None
                    }
                })
            })
            .collect()
    }

    pub fn state_changes(&mut self) -> &[state_changes::StateChange] {
        if self.state_changes.is_empty() {
            self.state_changes = self
                .streamer_message
                .shards
                .iter()
                .flat_map(|shard| shard.state_changes.iter())
                .map(Into::into)
                .collect();
        }
        &self.state_changes
    }

    pub fn action_by_receipt_id(
        &mut self,
        receipt_id: &super::ReceiptId,
    ) -> Option<&receipts::Action> {
        if self.actions.is_empty() {
            self.build_actions_hashmap();
        }
        self.actions.get(receipt_id)
    }

    pub fn events_by_receipt_id(&mut self, receipt_id: &super::ReceiptId) -> Vec<events::Event> {
        if self.events.is_empty() {
            self.build_events_hashmap();
        }
        if let Some(events) = self.events.get(receipt_id) {
            events.to_vec()
        } else {
            vec![]
        }
    }
}

impl Block {
    fn build_actions_hashmap(&mut self) {
        self.actions = self
            .actions()
            .iter()
            .map(|action| (action.receipt_id, action.clone()))
            .collect();
    }

    fn build_events_hashmap(&mut self) {
        self.events = self
            .receipts()
            .iter()
            .map(|receipt| (receipt.receipt_id, receipt.events()))
            .collect();
    }
}

impl From<StreamerMessage> for Block {
    fn from(streamer_message: StreamerMessage) -> Self {
        Self {
            streamer_message,
            executed_receipts: vec![],
            postponed_receipts: vec![],
            transactions: vec![],
            actions: HashMap::new(),
            events: HashMap::new(),
            state_changes: vec![],
        }
    }
}

#[derive(Debug, Clone)]
pub struct BlockHeader {
    pub height: u64,
    pub hash: CryptoHash,
    pub prev_hash: CryptoHash,
    pub author: AccountId,
    pub timestamp_nanosec: u64,
    pub epoch_id: CryptoHash,
    pub next_epoch_id: CryptoHash,
    pub gas_price: u128,
    pub total_supply: u128,
    pub latest_protocol_version: u32,
    pub random_value: CryptoHash,
    pub chunks_included: u64,
    pub validator_proposals: Vec<views::validator_stake_view::ValidatorStakeView>,
}

impl From<&StreamerMessage> for BlockHeader {
    fn from(streamer_message: &StreamerMessage) -> Self {
        Self {
            height: streamer_message.block.header.height,
            hash: streamer_message.block.header.hash,
            prev_hash: streamer_message.block.header.prev_hash,
            author: streamer_message.block.author.clone(),
            timestamp_nanosec: streamer_message.block.header.timestamp_nanosec,
            epoch_id: streamer_message.block.header.epoch_id,
            next_epoch_id: streamer_message.block.header.next_epoch_id,
            gas_price: streamer_message.block.header.gas_price,
            total_supply: streamer_message.block.header.total_supply,
            latest_protocol_version: streamer_message.block.header.latest_protocol_version,
            random_value: streamer_message.block.header.random_value,
            chunks_included: streamer_message.block.header.chunks_included,
            validator_proposals: streamer_message.block.header.validator_proposals.clone(),
        }
    }
}
