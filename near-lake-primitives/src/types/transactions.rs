use near_crypto::{PublicKey, Signature};

use super::receipts::{ExecutionStatus, Operation};
use crate::near_indexer_primitives::{types::AccountId, CryptoHash, IndexerTransactionWithOutcome};

#[derive(Debug, Clone)]
pub struct Transaction {
    pub transaction_hash: CryptoHash,
    pub signer_id: AccountId,
    pub signer_public_key: PublicKey,
    pub signature: Signature,
    pub receiver_id: AccountId,
    pub status: ExecutionStatus,
    pub execution_outcome_id: CryptoHash,
    pub operations: Vec<Operation>,
}

impl From<&IndexerTransactionWithOutcome> for Transaction {
    fn from(tx_with_outcome: &IndexerTransactionWithOutcome) -> Self {
        Self {
            transaction_hash: tx_with_outcome.transaction.hash,
            signer_id: tx_with_outcome.transaction.signer_id.clone(),
            signer_public_key: tx_with_outcome.transaction.public_key.clone(),
            signature: tx_with_outcome.transaction.signature.clone(),
            receiver_id: tx_with_outcome.transaction.receiver_id.clone(),
            execution_outcome_id: tx_with_outcome.outcome.execution_outcome.id,
            status: (&tx_with_outcome.outcome.execution_outcome.outcome.status).into(),
            operations: tx_with_outcome
                .transaction
                .actions
                .iter()
                .map(Into::into)
                .collect(),
        }
    }
}
