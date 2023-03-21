use crate::near_indexer_primitives::{
    types::{AccountId, Balance, Gas},
    views, CryptoHash, IndexerExecutionOutcomeWithReceipt,
};
use near_crypto::{PublicKey, Signature};

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
    pub fn receipt_kind(&self) -> ReceiptKind {
        self.receipt_kind.clone()
    }

    pub fn receipt_id(&self) -> CryptoHash {
        self.receipt_id
    }

    pub fn receiver_id(&self) -> AccountId {
        self.receiver_id.clone()
    }

    pub fn predecessor_id(&self) -> AccountId {
        self.predecessor_id.clone()
    }

    pub fn status(&self) -> ExecutionStatus {
        self.status.clone()
    }

    pub fn execution_outcome_id(&self) -> Option<CryptoHash> {
        self.execution_outcome_id
    }

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

#[derive(Debug, Clone)]
pub enum ReceiptKind {
    Action,
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

#[derive(Debug, Clone)]
pub enum ExecutionStatus {
    SuccessValue(Vec<u8>),
    SuccessReceiptId(CryptoHash),
    Failure(String),
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
                Self::Failure(tx_execution_error.to_string())
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Action {
    receipt_id: CryptoHash,
    predecessor_id: AccountId,
    receiver_id: AccountId,
    signer_id: AccountId,
    signer_public_key: PublicKey,
    operations: Vec<Operation>,
}

impl Action {
    pub fn receipt_id(&self) -> CryptoHash {
        self.receipt_id
    }

    pub fn predecessor_id(&self) -> AccountId {
        self.predecessor_id.clone()
    }

    pub fn receiver_id(&self) -> AccountId {
        self.receiver_id.clone()
    }

    pub fn signer_id(&self) -> AccountId {
        self.signer_id.clone()
    }

    pub fn signer_public_key(&self) -> PublicKey {
        self.signer_public_key.clone()
    }

    pub fn operations(&self) -> Vec<Operation> {
        self.operations.clone()
    }
}

impl TryFrom<&views::ReceiptView> for Action {
    type Error = &'static str;

    fn try_from(receipt: &views::ReceiptView) -> Result<Self, Self::Error> {
        if let views::ReceiptEnumView::Action {
            signer_id,
            signer_public_key,
            actions,
            ..
        } = &receipt.receipt
        {
            Ok(Self {
                receipt_id: receipt.receipt_id,
                predecessor_id: receipt.predecessor_id.clone(),
                receiver_id: receipt.receiver_id.clone(),
                signer_id: signer_id.clone(),
                signer_public_key: signer_public_key.clone(),
                operations: actions.iter().map(Into::into).collect(),
            })
        } else {
            Err("Only `ReceiptEnumView::Action` can be converted into Action")
        }
    }
}

#[derive(Debug, Clone)]
pub enum Operation {
    CreateAccount,
    DeployContract {
        code: Vec<u8>,
    },
    FunctionCall {
        method_name: String,
        args: Vec<u8>,
        gas: Gas,
        deposit: Balance,
    },
    Transfer {
        deposit: Balance,
    },
    Stake {
        stake: Balance,
        public_key: PublicKey,
    },
    AddKey {
        public_key: PublicKey,
        access_key: views::AccessKeyView,
    },
    DeleteKey {
        public_key: PublicKey,
    },
    DeleteAccount {
        beneficiary_id: AccountId,
    },
    Delegate {
        delegate_action: DelegateAction,
        signature: Signature,
    },
}

impl From<&views::ActionView> for Operation {
    fn from(action: &views::ActionView) -> Self {
        match action {
            &views::ActionView::CreateAccount => Self::CreateAccount,
            views::ActionView::DeployContract { code } => {
                Self::DeployContract { code: code.clone() }
            }
            views::ActionView::FunctionCall {
                method_name,
                args,
                gas,
                deposit,
            } => Self::FunctionCall {
                method_name: method_name.to_string(),
                args: args.clone(),
                gas: *gas,
                deposit: *deposit,
            },
            views::ActionView::Transfer { deposit } => Self::Transfer { deposit: *deposit },
            views::ActionView::Stake { stake, public_key } => Self::Stake {
                stake: *stake,
                public_key: public_key.clone(),
            },
            views::ActionView::AddKey {
                public_key,
                access_key,
            } => Self::AddKey {
                public_key: public_key.clone(),
                access_key: access_key.clone(),
            },
            views::ActionView::DeleteKey { public_key } => Self::DeleteKey {
                public_key: public_key.clone(),
            },
            views::ActionView::DeleteAccount { beneficiary_id } => Self::DeleteAccount {
                beneficiary_id: beneficiary_id.clone(),
            },
            views::ActionView::Delegate {
                delegate_action,
                signature,
            } => Self::Delegate {
                delegate_action: delegate_action.into(),
                signature: signature.clone(),
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct DelegateAction {
    pub sender_id: AccountId,
    pub receiver_id: AccountId,
    /// List of actions to be executed.
    ///
    /// With the meta transactions MVP defined in NEP-366, nested
    /// DelegateActions are not allowed. A separate type is used to enforce it.
    pub operations: Vec<NonDelegateOperation>,
    pub nonce: u64,
    /// The maximal height of the block in the blockchain below which the given DelegateAction is valid.
    pub max_block_height: u64,
    pub public_key: PublicKey,
}

impl From<&near_primitives::delegate_action::DelegateAction> for DelegateAction {
    fn from(delegate_action: &near_primitives::delegate_action::DelegateAction) -> Self {
        let operations: Vec<NonDelegateOperation> = delegate_action
            .actions
            .iter()
            .cloned()
            .map(Into::into)
            .collect();

        Self {
            sender_id: delegate_action.sender_id.clone(),
            receiver_id: delegate_action.receiver_id.clone(),
            operations,
            nonce: delegate_action.nonce,
            max_block_height: delegate_action.max_block_height,
            public_key: delegate_action.public_key.clone(),
        }
    }
}

impl From<near_primitives::delegate_action::NonDelegateAction> for NonDelegateOperation {
    fn from(non_delegate_action: near_primitives::delegate_action::NonDelegateAction) -> Self {
        non_delegate_action.into()
    }
}

#[derive(Debug, Clone)]
pub struct NonDelegateOperation(Operation);

impl From<NonDelegateOperation> for Operation {
    fn from(operation: NonDelegateOperation) -> Self {
        operation.0
    }
}

#[derive(Debug, thiserror::Error)]
#[error("attempted to construct NonDelegateOperation from Operation::Delegate")]
pub struct IsDelegateAction;

impl TryFrom<Operation> for NonDelegateOperation {
    type Error = IsDelegateAction;

    fn try_from(operation: Operation) -> Result<Self, IsDelegateAction> {
        if matches!(operation, Operation::Delegate { .. }) {
            Err(IsDelegateAction)
        } else {
            Ok(Self(operation))
        }
    }
}
