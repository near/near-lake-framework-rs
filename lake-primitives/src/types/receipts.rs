use std::convert::TryFrom;

use crate::near_indexer_primitives::{
    near_primitives,
    types::{AccountId, Balance, Gas},
    views, CryptoHash, IndexerExecutionOutcomeWithReceipt, IndexerTransactionWithOutcome,
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
    // TODO: handle the Failure and all the nested errors it has
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
                // TODO: handle the Failure and all the nested errors it has instead of stringifying
                Self::Failure(tx_execution_error.to_string())
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct ActionMetadata {
    receipt_id: CryptoHash,
    predecessor_id: AccountId,
    receiver_id: AccountId,
    signer_id: AccountId,
    signer_public_key: PublicKey,
}

impl ActionMetadata {
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
}

pub trait ActionMetaDataExt {
    fn metadata(&self) -> &ActionMetadata;

    fn receipt_id(&self) -> CryptoHash {
        self.metadata().receipt_id()
    }
    fn predecessor_id(&self) -> AccountId {
        self.metadata().predecessor_id()
    }
    fn receiver_id(&self) -> AccountId {
        self.metadata().receiver_id()
    }
    fn signer_id(&self) -> AccountId {
        self.metadata().signer_id()
    }
    fn signer_public_key(&self) -> PublicKey {
        self.metadata().signer_public_key()
    }
}

#[derive(Debug, Clone)]
pub enum Action {
    CreateAccount(CreateAccount),
    DeployContract(DeployContract),
    FunctionCall(FunctionCall),
    Transfer(Transfer),
    Stake(Stake),
    AddKey(AddKey),
    DeleteKey(DeleteKey),
    DeleteAccount(DeleteAccount),
    Delegate(Delegate),
}

impl ActionMetaDataExt for Action {
    fn metadata(&self) -> &ActionMetadata {
        match self {
            Self::CreateAccount(action) => action.metadata(),
            Self::DeployContract(action) => action.metadata(),
            Self::FunctionCall(action) => action.metadata(),
            Self::Transfer(action) => action.metadata(),
            Self::Stake(action) => action.metadata(),
            Self::AddKey(action) => action.metadata(),
            Self::DeleteKey(action) => action.metadata(),
            Self::DeleteAccount(action) => action.metadata(),
            Self::Delegate(action) => action.metadata(),
        }
    }
}

macro_rules! impl_as_action_for {
    ($action_type:ident) => {
        paste::paste! {
            pub fn [< as_ $action_type:snake:lower >](&self) -> Option<&$action_type> {
                match self {
                    Self::$action_type(action) => Some(action),
                    _ => None,
                }
            }
        }
    };
}

impl Action {
    impl_as_action_for!(CreateAccount);
    impl_as_action_for!(DeployContract);
    impl_as_action_for!(FunctionCall);
    impl_as_action_for!(Transfer);
    impl_as_action_for!(Stake);
    impl_as_action_for!(AddKey);
    impl_as_action_for!(DeleteKey);
    impl_as_action_for!(DeleteAccount);
    impl_as_action_for!(Delegate);
}

// Macro to implement ActionMetaDataExt trait for each Action variant.
macro_rules! impl_action_metadata_ext {
    ($action:ident) => {
        impl ActionMetaDataExt for $action {
            fn metadata(&self) -> &ActionMetadata {
                &self.metadata
            }
        }
    };
}

impl_action_metadata_ext!(CreateAccount);
impl_action_metadata_ext!(DeployContract);
impl_action_metadata_ext!(FunctionCall);
impl_action_metadata_ext!(Transfer);
impl_action_metadata_ext!(Stake);
impl_action_metadata_ext!(AddKey);
impl_action_metadata_ext!(DeleteKey);
impl_action_metadata_ext!(DeleteAccount);
impl_action_metadata_ext!(Delegate);

#[derive(Debug, Clone)]
pub struct CreateAccount {
    metadata: ActionMetadata,
}

#[derive(Debug, Clone)]
pub struct DeployContract {
    metadata: ActionMetadata,
    code: Vec<u8>,
}

impl DeployContract {
    pub fn code(&self) -> &[u8] {
        &self.code
    }
}

#[derive(Debug, Clone)]
pub struct FunctionCall {
    metadata: ActionMetadata,
    method_name: String,
    args: Vec<u8>,
    gas: Gas,
    deposit: Balance,
}

impl FunctionCall {
    pub fn method_name(&self) -> &str {
        &self.method_name
    }

    pub fn args(&self) -> &[u8] {
        &self.args
    }

    pub fn gas(&self) -> Gas {
        self.gas
    }

    pub fn deposit(&self) -> Balance {
        self.deposit
    }
}

#[derive(Debug, Clone)]
pub struct Transfer {
    metadata: ActionMetadata,
    deposit: Balance,
}

impl Transfer {
    pub fn deposit(&self) -> Balance {
        self.deposit
    }
}

#[derive(Debug, Clone)]
pub struct Stake {
    metadata: ActionMetadata,
    stake: Balance,
    public_key: PublicKey,
}

impl Stake {
    pub fn stake(&self) -> Balance {
        self.stake
    }

    pub fn public_key(&self) -> &PublicKey {
        &self.public_key
    }
}

#[derive(Debug, Clone)]
pub struct AddKey {
    metadata: ActionMetadata,
    public_key: PublicKey,
    access_key: views::AccessKeyView,
}

impl AddKey {
    pub fn public_key(&self) -> &PublicKey {
        &self.public_key
    }

    pub fn access_key(&self) -> &views::AccessKeyView {
        &self.access_key
    }
}

#[derive(Debug, Clone)]
pub struct DeleteKey {
    metadata: ActionMetadata,
    public_key: PublicKey,
}

impl DeleteKey {
    pub fn public_key(&self) -> &PublicKey {
        &self.public_key
    }
}

#[derive(Debug, Clone)]
pub struct DeleteAccount {
    metadata: ActionMetadata,
    beneficiary_id: AccountId,
}

impl DeleteAccount {
    pub fn beneficiary_id(&self) -> &AccountId {
        &self.beneficiary_id
    }
}

#[derive(Debug, Clone)]
pub struct Delegate {
    metadata: ActionMetadata,
    delegate_action: Vec<DelegateAction>,
    signature: Signature,
}

impl Delegate {
    pub fn delegate_action(&self) -> &[DelegateAction] {
        &self.delegate_action
    }

    pub fn signature(&self) -> &Signature {
        &self.signature
    }
}

#[derive(Debug, Clone)]
pub enum DelegateAction {
    CreateAccount(CreateAccount),
    DeployContract(DeployContract),
    FunctionCall(FunctionCall),
    Transfer(Transfer),
    Stake(Stake),
    AddKey(AddKey),
    DeleteKey(DeleteKey),
    DeleteAccount(DeleteAccount),
}

impl TryFrom<Action> for DelegateAction {
    type Error = &'static str;

    fn try_from(action: Action) -> Result<Self, Self::Error> {
        match action {
            Action::CreateAccount(ca) => Ok(Self::CreateAccount(ca)),
            Action::DeployContract(dc) => Ok(Self::DeployContract(dc)),
            Action::FunctionCall(fc) => Ok(Self::FunctionCall(fc)),
            Action::Transfer(t) => Ok(Self::Transfer(t)),
            Action::Stake(s) => Ok(Self::Stake(s)),
            Action::AddKey(ak) => Ok(Self::AddKey(ak)),
            Action::DeleteKey(dk) => Ok(Self::DeleteKey(dk)),
            Action::DeleteAccount(da) => Ok(Self::DeleteAccount(da)),
            Action::Delegate(_) => Err("Cannot convert DelegateAction to DelegateAction"),
        }
    }
}

impl Action {
    // Tries to convert a ReceiptView into a vector of Action.
    pub fn try_vec_from_receipt_view(
        receipt_view: &views::ReceiptView,
    ) -> Result<Vec<Self>, &'static str> {
        if let views::ReceiptEnumView::Action {
            actions,
            signer_id,
            signer_public_key,
            ..
        } = &receipt_view.receipt
        {
            let metadata = ActionMetadata {
                receipt_id: receipt_view.receipt_id,
                predecessor_id: receipt_view.predecessor_id.clone(),
                receiver_id: receipt_view.receiver_id.clone(),
                signer_id: signer_id.clone(),
                signer_public_key: signer_public_key.clone(),
            };

            let mut result = Vec::with_capacity(actions.len());

            for action in actions {
                let action_kind = match action {
                    views::ActionView::CreateAccount => Self::CreateAccount(CreateAccount {
                        metadata: metadata.clone(),
                    }),
                    views::ActionView::DeployContract { code } => {
                        Self::DeployContract(DeployContract {
                            metadata: metadata.clone(),
                            code: code.clone(),
                        })
                    }
                    views::ActionView::FunctionCall {
                        method_name,
                        args,
                        gas,
                        deposit,
                    } => Self::FunctionCall(FunctionCall {
                        metadata: metadata.clone(),
                        method_name: method_name.clone(),
                        args: args.clone(),
                        gas: *gas,
                        deposit: *deposit,
                    }),
                    views::ActionView::Transfer { deposit } => Self::Transfer(Transfer {
                        metadata: metadata.clone(),
                        deposit: *deposit,
                    }),
                    views::ActionView::Stake { stake, public_key } => Self::Stake(Stake {
                        metadata: metadata.clone(),
                        stake: *stake,
                        public_key: public_key.clone(),
                    }),
                    views::ActionView::AddKey {
                        public_key,
                        access_key,
                    } => Self::AddKey(AddKey {
                        metadata: metadata.clone(),
                        public_key: public_key.clone(),
                        access_key: access_key.clone(),
                    }),
                    views::ActionView::DeleteKey { public_key } => Self::DeleteKey(DeleteKey {
                        metadata: metadata.clone(),
                        public_key: public_key.clone(),
                    }),
                    views::ActionView::DeleteAccount { beneficiary_id } => {
                        Self::DeleteAccount(DeleteAccount {
                            metadata: metadata.clone(),
                            beneficiary_id: beneficiary_id.clone(),
                        })
                    }
                    views::ActionView::Delegate {
                        delegate_action,
                        signature,
                    } => {
                        let delegate_actions =
                            Self::try_from_delegate_action(delegate_action, metadata.clone())?
                                .into_iter()
                                .map(TryInto::try_into)
                                .collect::<Result<Vec<DelegateAction>, &str>>()?;

                        Self::Delegate(Delegate {
                            metadata: metadata.clone(),
                            delegate_action: delegate_actions,
                            signature: signature.clone(),
                        })
                    }
                };
                result.push(action_kind);
            }
            Ok(result)
        } else {
            Err("Only `ReceiptEnumView::Action` can be converted into Vec<Action>")
        }
    }

    // Tries to convert a near_primitives::delegate_action::DelegateAction into a vector of Action.
    pub fn try_from_delegate_action(
        delegate_action: &near_primitives::delegate_action::DelegateAction,
        metadata: ActionMetadata,
    ) -> Result<Vec<Self>, &'static str> {
        let mut actions = Vec::with_capacity(delegate_action.actions.len());

        for nearcore_action in delegate_action.clone().actions {
            let action = match views::ActionView::from(
                <near_primitives::delegate_action::NonDelegateAction as Into<
                    near_primitives::transaction::Action,
                >>::into(nearcore_action),
            ) {
                views::ActionView::CreateAccount => Self::CreateAccount(CreateAccount {
                    metadata: metadata.clone(),
                }),
                views::ActionView::DeployContract { code } => {
                    Self::DeployContract(DeployContract {
                        metadata: metadata.clone(),
                        code,
                    })
                }
                views::ActionView::FunctionCall {
                    method_name,
                    args,
                    gas,
                    deposit,
                } => Self::FunctionCall(FunctionCall {
                    metadata: metadata.clone(),
                    method_name,
                    args,
                    gas,
                    deposit,
                }),
                views::ActionView::Transfer { deposit } => Self::Transfer(Transfer {
                    metadata: metadata.clone(),
                    deposit,
                }),
                views::ActionView::Stake { stake, public_key } => Self::Stake(Stake {
                    metadata: metadata.clone(),
                    stake,
                    public_key,
                }),
                views::ActionView::AddKey {
                    public_key,
                    access_key,
                } => Self::AddKey(AddKey {
                    metadata: metadata.clone(),
                    public_key,
                    access_key,
                }),
                views::ActionView::DeleteKey { public_key } => Self::DeleteKey(DeleteKey {
                    metadata: metadata.clone(),
                    public_key,
                }),
                views::ActionView::DeleteAccount { beneficiary_id } => {
                    Self::DeleteAccount(DeleteAccount {
                        metadata: metadata.clone(),
                        beneficiary_id,
                    })
                }
                _ => return Err("Cannot delegate DelegateAction"),
            };
            actions.push(action);
        }
        Ok(actions)
    }

    // Tries to convert a IndexerTransactionWithOutcome to a Vec<Action>
    pub fn try_vec_from_transaction_outcome(
        transaction_with_outcome: &IndexerTransactionWithOutcome,
    ) -> Result<Vec<Self>, &'static str> {
        let metadata = ActionMetadata {
            receipt_id: *transaction_with_outcome
                .outcome
                .execution_outcome
                .outcome
                .receipt_ids
                .get(0)
                .ok_or("Transaction conversion ReceiptId is missing")?,
            predecessor_id: transaction_with_outcome.transaction.signer_id.clone(),
            receiver_id: transaction_with_outcome.transaction.receiver_id.clone(),
            signer_id: transaction_with_outcome.transaction.signer_id.clone(),
            signer_public_key: transaction_with_outcome.transaction.public_key.clone(),
        };

        let mut actions: Vec<Self> = vec![];

        for nearcore_action in &transaction_with_outcome.transaction.actions {
            let action = match nearcore_action {
                views::ActionView::CreateAccount => Self::CreateAccount(CreateAccount {
                    metadata: metadata.clone(),
                }),
                views::ActionView::DeployContract { code } => {
                    Self::DeployContract(DeployContract {
                        metadata: metadata.clone(),
                        code: code.to_vec(),
                    })
                }
                views::ActionView::FunctionCall {
                    method_name,
                    args,
                    gas,
                    deposit,
                } => Self::FunctionCall(FunctionCall {
                    metadata: metadata.clone(),
                    method_name: method_name.to_string(),
                    args: args.to_vec(),
                    gas: *gas,
                    deposit: *deposit,
                }),
                views::ActionView::Transfer { deposit } => Self::Transfer(Transfer {
                    metadata: metadata.clone(),
                    deposit: *deposit,
                }),
                views::ActionView::Stake { stake, public_key } => Self::Stake(Stake {
                    metadata: metadata.clone(),
                    stake: *stake,
                    public_key: public_key.clone(),
                }),
                views::ActionView::AddKey {
                    public_key,
                    access_key,
                } => Self::AddKey(AddKey {
                    metadata: metadata.clone(),
                    public_key: public_key.clone(),
                    access_key: access_key.clone(),
                }),
                views::ActionView::DeleteKey { public_key } => Self::DeleteKey(DeleteKey {
                    metadata: metadata.clone(),
                    public_key: public_key.clone(),
                }),
                views::ActionView::DeleteAccount { beneficiary_id } => {
                    Self::DeleteAccount(DeleteAccount {
                        metadata: metadata.clone(),
                        beneficiary_id: beneficiary_id.clone(),
                    })
                }
                views::ActionView::Delegate {
                    delegate_action,
                    signature,
                } => Self::Delegate(Delegate {
                    metadata: metadata.clone(),
                    delegate_action: Self::try_from_delegate_action(
                        delegate_action,
                        metadata.clone(),
                    )?
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<Vec<DelegateAction>, &str>>()?,
                    signature: signature.clone(),
                }),
            };

            actions.push(action);
        }

        Ok(actions)
    }
}
