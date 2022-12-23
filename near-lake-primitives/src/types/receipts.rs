use crate::near_indexer_primitives::{
    types::{AccountId, Balance, Gas},
    views, CryptoHash, IndexerExecutionOutcomeWithReceipt,
};
use near_crypto::PublicKey;
use near_primitives_core::serialize::{base64_format, dec_format};

#[derive(Debug, Clone)]
pub struct Receipt {
    pub receipt_kind: ReceiptKind,
    pub receipt_id: CryptoHash,
    pub receiver_id: AccountId,
    pub predecessor_id: AccountId,
    pub status: ExecutionStatus,
    pub execution_outcome_id: Option<CryptoHash>,
    pub logs: Vec<String>,
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Action {
    pub receipt_id: CryptoHash,
    pub predecessor_id: AccountId,
    pub receiver_id: AccountId,
    pub signer_id: AccountId,
    pub signer_public_key: PublicKey,
    pub operations: Vec<Operation>,
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum Operation {
    CreateAccount,
    DeployContract {
        #[serde(with = "base64_format")]
        code: Vec<u8>,
    },
    FunctionCall {
        method_name: String,
        #[serde(with = "base64_format")]
        args: Vec<u8>,
        gas: Gas,
        #[serde(with = "dec_format")]
        deposit: Balance,
    },
    Transfer {
        #[serde(with = "dec_format")]
        deposit: Balance,
    },
    Stake {
        #[serde(with = "dec_format")]
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
        }
    }
}
