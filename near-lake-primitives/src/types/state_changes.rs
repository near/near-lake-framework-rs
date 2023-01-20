use near_crypto::PublicKey;

use crate::near_indexer_primitives::{
    types::AccountId,
    views::{
        AccessKeyView, AccountView, StateChangeCauseView, StateChangeValueView,
        StateChangeWithCauseView,
    },
    CryptoHash,
};

#[derive(Debug, Clone)]
pub struct StateChange {
    affected_account_id: AccountId,
    cause: StateChangeCause,
    value: StateChangeValue,
}

impl StateChange {
    pub fn affected_account_id(&self) -> AccountId {
        self.affected_account_id.clone()
    }

    pub fn cause(&self) -> StateChangeCause {
        self.cause.clone()
    }

    pub fn value(&self) -> StateChangeValue {
        self.value.clone()
    }
}

impl From<&StateChangeWithCauseView> for StateChange {
    fn from(state_change_with_cause_view: &StateChangeWithCauseView) -> Self {
        let cause: StateChangeCause = (&state_change_with_cause_view.cause).into();
        let value: StateChangeValue = (&state_change_with_cause_view.value).into();
        Self {
            affected_account_id: value.affected_account_id(),
            cause,
            value,
        }
    }
}

#[derive(Debug, Clone)]
pub enum StateChangeCause {
    NotWritableToDisk,
    InitialState,
    TransactionProcessing { tx_hash: CryptoHash },
    ActionReceiptProcessingStarted { receipt_hash: CryptoHash },
    ActionReceiptGasReward { receipt_hash: CryptoHash },
    ReceiptProcessing { receipt_hash: CryptoHash },
    PostponedReceipt { receipt_hash: CryptoHash },
    UpdatedDelayedReceipts,
    ValidatorAccountsUpdate,
    Migration,
    Resharding,
}

impl From<&StateChangeCauseView> for StateChangeCause {
    fn from(state_change_cause: &StateChangeCauseView) -> Self {
        match state_change_cause {
            StateChangeCauseView::NotWritableToDisk => Self::NotWritableToDisk,
            StateChangeCauseView::InitialState => Self::InitialState,
            StateChangeCauseView::TransactionProcessing { tx_hash } => {
                Self::TransactionProcessing { tx_hash: *tx_hash }
            }
            StateChangeCauseView::ActionReceiptProcessingStarted { receipt_hash } => {
                Self::ActionReceiptProcessingStarted {
                    receipt_hash: *receipt_hash,
                }
            }
            StateChangeCauseView::ActionReceiptGasReward { receipt_hash } => {
                Self::ActionReceiptGasReward {
                    receipt_hash: *receipt_hash,
                }
            }
            StateChangeCauseView::ReceiptProcessing { receipt_hash } => Self::ReceiptProcessing {
                receipt_hash: *receipt_hash,
            },
            StateChangeCauseView::PostponedReceipt { receipt_hash } => Self::PostponedReceipt {
                receipt_hash: *receipt_hash,
            },
            StateChangeCauseView::UpdatedDelayedReceipts => Self::UpdatedDelayedReceipts,
            StateChangeCauseView::ValidatorAccountsUpdate => Self::ValidatorAccountsUpdate,
            StateChangeCauseView::Migration => Self::Migration,
            StateChangeCauseView::Resharding => Self::Resharding,
        }
    }
}

#[derive(Debug, Clone)]
pub enum StateChangeValue {
    AccountUpdate {
        account_id: AccountId,
        account: AccountView,
    },
    AccountDeletion {
        account_id: AccountId,
    },
    AccessKeyUpdate {
        account_id: AccountId,
        public_key: PublicKey,
        access_key: AccessKeyView,
    },
    AccessKeyDeletion {
        account_id: AccountId,
        public_key: PublicKey,
    },
    DataUpdate {
        account_id: AccountId,
        key: Vec<u8>,
        value: Vec<u8>,
    },
    DataDeletion {
        account_id: AccountId,
        key: Vec<u8>,
    },
    ContractCodeUpdate {
        account_id: AccountId,
        code: Vec<u8>,
    },
    ContractCodeDeletion {
        account_id: AccountId,
    },
}

impl StateChangeValue {
    pub fn affected_account_id(&self) -> AccountId {
        match self {
            Self::AccountUpdate { account_id, .. } => account_id.clone(),
            Self::AccountDeletion { account_id } => account_id.clone(),
            Self::AccessKeyUpdate { account_id, .. } => account_id.clone(),
            Self::AccessKeyDeletion { account_id, .. } => account_id.clone(),
            Self::DataUpdate { account_id, .. } => account_id.clone(),
            Self::DataDeletion { account_id, .. } => account_id.clone(),
            Self::ContractCodeUpdate { account_id, .. } => account_id.clone(),
            Self::ContractCodeDeletion { account_id } => account_id.clone(),
        }
    }
}

impl From<&StateChangeValueView> for StateChangeValue {
    fn from(state_change_value: &StateChangeValueView) -> Self {
        match state_change_value {
            StateChangeValueView::AccountUpdate {
                account_id,
                account,
            } => Self::AccountUpdate {
                account_id: account_id.clone(),
                account: account.clone(),
            },
            StateChangeValueView::AccountDeletion { account_id } => Self::AccountDeletion {
                account_id: account_id.clone(),
            },
            StateChangeValueView::AccessKeyUpdate {
                account_id,
                public_key,
                access_key,
            } => Self::AccessKeyUpdate {
                account_id: account_id.clone(),
                public_key: public_key.clone(),
                access_key: access_key.clone(),
            },
            StateChangeValueView::AccessKeyDeletion {
                account_id,
                public_key,
            } => Self::AccessKeyDeletion {
                account_id: account_id.clone(),
                public_key: public_key.clone(),
            },
            StateChangeValueView::DataUpdate {
                account_id,
                key,
                value,
            } => {
                let key: &[u8] = key.as_ref();
                let value: &[u8] = value.as_ref();
                Self::DataUpdate {
                    account_id: account_id.clone(),
                    key: key.to_vec(),
                    value: value.to_vec(),
                }
            }
            StateChangeValueView::DataDeletion { account_id, key } => {
                let key: &[u8] = key.as_ref();
                Self::DataDeletion {
                    account_id: account_id.clone(),
                    key: key.to_vec(),
                }
            }
            StateChangeValueView::ContractCodeUpdate { account_id, code } => {
                Self::ContractCodeUpdate {
                    account_id: account_id.clone(),
                    code: code.clone(),
                }
            }
            StateChangeValueView::ContractCodeDeletion { account_id } => {
                Self::ContractCodeDeletion {
                    account_id: account_id.clone(),
                }
            }
        }
    }
}
