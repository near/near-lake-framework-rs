use near_crypto::{PublicKey, Signature};

use super::receipts::ExecutionStatus;
use crate::near_indexer_primitives::{types::AccountId, CryptoHash, IndexerTransactionWithOutcome};

/// High-level representation of the `Transaction`.
///
/// The structure basically combines the `Transaction` itself and the corresponding `ExecutionOutcome`.
/// **Reminder**: the result of the transaction execution is always a [Receipt](super::receipts::Receipt)
/// that looks pretty much like the `Transaction` itself.
///
/// #### Important notes on the Transaction
///
/// Transaction's `actions` are represented by the [Action](super::actions::Action) enum. Actions are
/// included for the informational purpose to help developers to know what exactly should happen after the
/// `Transaction` is executed.
#[derive(Debug, Clone)]
pub struct Transaction {
    transaction_hash: CryptoHash,
    signer_id: AccountId,
    signer_public_key: PublicKey,
    signature: Signature,
    receiver_id: AccountId,
    status: ExecutionStatus,
    execution_outcome_id: CryptoHash,
    actions: Vec<super::actions::Action>,
}

impl Transaction {
    /// Returns the [CryptoHash] hash of the transaction.
    pub fn transaction_hash(&self) -> CryptoHash {
        self.transaction_hash
    }

    /// Returns the [AccountId] of the signer of the transaction.
    pub fn signer_id(&self) -> &AccountId {
        &self.signer_id
    }

    /// Returns the [PublicKey] of the signer of the transaction.
    pub fn signer_public_key(&self) -> &PublicKey {
        &self.signer_public_key
    }

    /// Returns the [Signature] of the transaction.
    pub fn signature(&self) -> &Signature {
        &self.signature
    }

    /// Returns the [AccountId] of the receiver of the transaction.
    pub fn receiver_id(&self) -> &AccountId {
        &self.receiver_id
    }

    /// Returns the [ExecutionStatus] of the corresponding ExecutionOutcome.
    pub fn status(&self) -> &ExecutionStatus {
        &self.status
    }

    /// Returns the [CryptoHash] id of the corresponding ExecutionOutcome.
    pub fn execution_outcome_id(&self) -> CryptoHash {
        self.execution_outcome_id
    }

    /// Returns the [Action](super::actions::Action) of the transaction.
    pub fn actions_included(&self) -> impl Iterator<Item = &super::actions::Action> {
        self.actions.iter()
    }
}

impl TryFrom<&IndexerTransactionWithOutcome> for Transaction {
    type Error = &'static str;

    fn try_from(tx_with_outcome: &IndexerTransactionWithOutcome) -> Result<Self, Self::Error> {
        Ok(Self {
            transaction_hash: tx_with_outcome.transaction.hash,
            signer_id: tx_with_outcome.transaction.signer_id.clone(),
            signer_public_key: tx_with_outcome.transaction.public_key.clone(),
            signature: tx_with_outcome.transaction.signature.clone(),
            receiver_id: tx_with_outcome.transaction.receiver_id.clone(),
            execution_outcome_id: tx_with_outcome.outcome.execution_outcome.id,
            status: (&tx_with_outcome.outcome.execution_outcome.outcome.status).into(),
            actions: super::actions::Action::try_vec_from_transaction_outcome(tx_with_outcome)?,
        })
    }
}
