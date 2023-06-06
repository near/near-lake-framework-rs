use crate::near_indexer_primitives::{
    types::AccountId, views, CryptoHash, IndexerExecutionOutcomeWithReceipt,
};

/// Simplified representation of the `Receipt`.
///
/// This is a simplification from the [near_primitives::views::ReceiptView] and [near_primitives::views::ReceiptEnumView] into a more flat structure.
/// The [ReceiptKind] is used to distinguish between the different types of receipts: Action and Data.
///
/// #### Important notes on the Receipt
///
/// The original low-level Receipt is represented by the enum that differentiates between the Action and Data receipts. In turn this enum is a field
/// `receipt` in the parent `ReceiptView` struct.
/// Parent structure has a set of fields that are common for both Action and Data receipts.
/// During the simplification we have put the common fields into the [Receipt] struct itself and extracted the `actions` from Action Receipt into a separate struct.
/// Since the high-level NEAR Lake Framework update we encourage developers to create more actions-and-events oriented indexers instead.
#[derive(Debug, Clone)]
pub struct Receipt {
    receipt_kind: ReceiptKind,
    receipt_id: CryptoHash,
    receiver_id: AccountId,
    predecessor_id: AccountId,
    status: ExecutionStatus,
    execution_outcome_id: Option<CryptoHash>,
    logs: Vec<String>,
}

impl Receipt {
    /// Returns the [ReceiptKind](ReceiptKind) of the receipt.
    ///
    /// This is a simplification from the [near_primitives::views::ReceiptEnumView::Action] into a more flat structure
    /// that has a type.
    pub fn receipt_kind(&self) -> ReceiptKind {
        self.receipt_kind.clone()
    }

    /// Returns the [CryptoHash] id of the receipt.
    pub fn receipt_id(&self) -> CryptoHash {
        self.receipt_id
    }

    /// Returns the [AccountId] of the receiver of the receipt.
    pub fn receiver_id(&self) -> AccountId {
        self.receiver_id.clone()
    }

    /// Returns the [AccountId] of the predecessor of the receipt.
    pub fn predecessor_id(&self) -> AccountId {
        self.predecessor_id.clone()
    }

    /// Returns the [ExecutionStatus] of the corresponding ExecutionOutcome.
    ///
    /// Note that the status will be `Postponed` for the receipts that are included in the block but not executed yet.
    pub fn status(&self) -> ExecutionStatus {
        self.status.clone()
    }

    /// Returns the [CryptoHash] id of the corresponding ExecutionOutcome if it exists.
    ///
    /// Note that this is an optional field because the ExecutionOutcome might not be available
    /// if the [Receipt] is "postponed" (included in the block but not executed yet)
    pub fn execution_outcome_id(&self) -> Option<CryptoHash> {
        self.execution_outcome_id
    }

    /// Returns the logs of the corresponding ExecutionOutcome.
    /// Might be an empty Vec if the ExecutionOutcome is not available.
    pub fn logs(&self) -> Vec<String> {
        self.logs.clone()
    }
}

impl From<&IndexerExecutionOutcomeWithReceipt> for Receipt {
    fn from(outcome_with_receipt: &IndexerExecutionOutcomeWithReceipt) -> Self {
        Self {
            receipt_kind: (&outcome_with_receipt.receipt.receipt).into(),
            receipt_id: outcome_with_receipt.receipt.receipt_id,
            receiver_id: outcome_with_receipt.receipt.receiver_id.clone(),
            predecessor_id: outcome_with_receipt.receipt.predecessor_id.clone(),
            execution_outcome_id: Some(outcome_with_receipt.execution_outcome.id),
            logs: outcome_with_receipt
                .execution_outcome
                .outcome
                .logs
                .iter()
                .map(Clone::clone)
                .collect(),
            status: (&outcome_with_receipt.execution_outcome.outcome.status).into(),
        }
    }
}

impl From<&views::ReceiptView> for Receipt {
    fn from(receipt: &views::ReceiptView) -> Self {
        Self {
            receipt_kind: (&receipt.receipt).into(),
            receipt_id: receipt.receipt_id,
            receiver_id: receipt.receiver_id.clone(),
            predecessor_id: receipt.predecessor_id.clone(),
            status: ExecutionStatus::Postponed,
            execution_outcome_id: None,
            logs: vec![],
        }
    }
}

/// Represents the Receipt kind: Action or Data.
#[derive(Debug, Clone)]
pub enum ReceiptKind {
    /// For the Action Receipt
    Action,
    /// For the Data Receipt
    Data,
}

impl From<&views::ReceiptEnumView> for ReceiptKind {
    fn from(receipt_enum: &views::ReceiptEnumView) -> Self {
        match receipt_enum {
            views::ReceiptEnumView::Action { .. } => Self::Action,
            views::ReceiptEnumView::Data { .. } => Self::Data,
        }
    }
}

/// Representation of the execution status for the [Receipt].
#[derive(Debug, Clone)]
pub enum ExecutionStatus {
    /// Execution succeeded with a value, value is represented by [`Vec<u8>`] and literally can be anything.
    SuccessValue(Vec<u8>),
    /// Execution succeeded and a result of the execution is a new [Receipt] with the id represented by [CryptoHash]
    SuccessReceiptId(CryptoHash),
    // TODO: handle the Failure and all the nested errors it has
    /// Execution failed with an error represented by a [String]
    /// **WARNINNG!** Here must be our representation of the `TxExecutionError from `near-primitives` instead of the [String].
    /// It requires some additional work on our version of the error, meanwhile we’ve left the [String] here, **this is subject to change
    /// in the nearest updates**.
    Failure(String),
    /// Execution hasn’t started yet, it is postponed (delayed) and will be later.
    /// The Receipt with such status is considered as postponed too (included, yet not executed)
    Postponed,
}

impl From<&views::ExecutionStatusView> for ExecutionStatus {
    fn from(execution_status_view: &views::ExecutionStatusView) -> Self {
        match execution_status_view {
            views::ExecutionStatusView::Unknown => Self::Postponed,
            views::ExecutionStatusView::SuccessValue(value) => Self::SuccessValue(value.clone()),
            views::ExecutionStatusView::SuccessReceiptId(receipt_id) => {
                Self::SuccessReceiptId(*receipt_id)
            }
            views::ExecutionStatusView::Failure(tx_execution_error) => {
                // TODO: handle the Failure and all the nested errors it has instead of stringifying
                Self::Failure(tx_execution_error.to_string())
            }
        }
    }
}
