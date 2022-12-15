use std::collections::HashMap;

// use futures::{Future, StreamExt};
// pub use near_lake_framework::{
pub use near_indexer_primitives;

use types::events::EventsTrait;

pub mod types;

// #[derive(Debug)]
// pub struct L8kit {
//     // chain_id: ChainId,
//     lake_config: LakeConfigBuilder, // does not implement Debug
// }

// impl L8kit {
//     pub fn testnet() -> Self {
//         Self {
//             // chain_id: ChainId::Testnet,
//             lake_config: LakeConfigBuilder::default().testnet(),
//         }
//     }

//     pub fn mainnet() -> Self {
//         Self {
//             // chain_id: ChainId::Mainnet,
//             lake_config: LakeConfigBuilder::default().mainnet(),
//         }
//     }

//     pub fn from_block(mut self, block_height: u64) -> Self {
//         self.lake_config = self.lake_config.start_block_height(block_height);
//         self
//     }

//     pub fn run<Fut>(self, f: impl Fn(L8kitContext) -> Fut) -> anyhow::Result<()>
//     where
//         Fut: Future<Output = anyhow::Result<()>>,
//     {
//         let runtime = tokio::runtime::Runtime::new()?;

//         let lake_config = self
//             .lake_config
//             .build()
//             .expect("Failed to build LakeConfig");

//         runtime.block_on(async {
//             // instantiate the NEAR Lake Framework Stream
//             let (sender, stream) = near_lake_framework::streamer(lake_config);
//             // read the stream events and pass them to a handler function with
//             // concurrency 1
//             let mut handlers = tokio_stream::wrappers::ReceiverStream::new(stream)
//                 .map(|streamer_message| async {
//                     let context = L8kitContext {
//                         block: (&streamer_message).into(),
//                         streamer_message,
//                         actions: HashMap::new(),
//                         events: HashMap::new(),
//                     };
//                     f(context).await
//                 })
//                 .buffer_unordered(1usize);

//             while let Some(_handle_message) = handlers.next().await {}
//             drop(handlers); // close the channel so the sender will stop

//             // propagate errors from the sender
//             match sender.await {
//                 Ok(Ok(())) => Ok(()),
//                 Ok(Err(e)) => Err(e),
//                 Err(e) => Err(anyhow::Error::from(e)), // JoinError
//             }
//         })
//     }
// }

#[derive(Debug)]
pub struct LakeContext {
    pub block: types::block::Block,
    streamer_message: near_indexer_primitives::StreamerMessage,
    actions: HashMap<types::ReceiptId, types::receipts::Action>,
    events: HashMap<types::ReceiptId, Vec<types::events::Event>>,
}

impl LakeContext {
    pub fn from_streamer_message(
        streamer_message: near_indexer_primitives::StreamerMessage,
    ) -> Self {
        Self {
            block: (&streamer_message).into(),
            streamer_message,
            actions: HashMap::new(),
            events: HashMap::new(),
        }
    }
    pub fn streamer_message(&self) -> &near_indexer_primitives::StreamerMessage {
        &self.streamer_message
    }
    pub fn actions(&self) -> Vec<types::receipts::Action> {
        self.streamer_message()
            .shards
            .iter()
            .flat_map(|shard| shard.receipt_execution_outcomes.iter())
            .filter_map(|receipt_execution_outcome| {
                types::receipts::Action::try_from(&receipt_execution_outcome.receipt).ok()
            })
            .collect()
    }
    pub fn events(&self) -> Vec<(types::ReceiptId, types::events::Event)> {
        self.block
            .receipts()
            .iter()
            .flat_map(|executed_receipt| {
                executed_receipt.logs.iter().filter_map(|log| {
                    if let Ok(event) = types::events::Event::from_log(log) {
                        Some((executed_receipt.receipt_id, event))
                    } else {
                        None
                    }
                })
            })
            .collect()
    }
    pub fn action_by_receipt_id(
        &mut self,
        receipt_id: &types::ReceiptId,
    ) -> Option<&types::receipts::Action> {
        if self.actions.is_empty() {
            self.build_actions_hashmap();
        }
        self.actions.get(receipt_id)
    }

    pub fn events_by_receipt_id(
        &mut self,
        receipt_id: &types::ReceiptId,
    ) -> Vec<types::events::Event> {
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

impl LakeContext {
    fn build_actions_hashmap(&mut self) {
        self.actions = self
            .actions()
            .iter()
            .map(|action| (action.receipt_id, action.clone()))
            .collect();
    }

    fn build_events_hashmap(&mut self) {
        self.events = self
            .block
            .receipts()
            .iter()
            .map(|receipt| (receipt.receipt_id, receipt.events()))
            .collect();
    }
}
