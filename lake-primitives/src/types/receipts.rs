use std::convert::TryFrom;

use crate::near_indexer_primitives::{
    near_primitives,
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

// =================================================================================================

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

pub trait Action {
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

    fn as_create_account(&self) -> Option<&CreateAccount> {
        None
    }

    fn as_deploy_contract(&self) -> Option<&DeployContract> {
        None
    }

    fn as_function_call(&self) -> Option<&FunctionCall> {
        None
    }

    fn as_transfer(&self) -> Option<&Transfer> {
        None
    }

    fn as_stake(&self) -> Option<&Stake> {
        None
    }

    fn as_add_key(&self) -> Option<&AddKey> {
        None
    }

    fn as_delete_key(&self) -> Option<&DeleteKey> {
        None
    }

    fn as_delete_account(&self) -> Option<&DeleteAccount> {
        None
    }

    fn as_delegate_action(&self) -> Option<&DelegateAction> {
        None
    }
}

#[derive(Debug, Clone)]
pub enum ActionKind {
    CreateAccount(CreateAccount),
    DeployContract(DeployContract),
    FunctionCall(FunctionCall),
    Transfer(Transfer),
    Stake(Stake),
    AddKey(AddKey),
    DeleteKey(DeleteKey),
    DeleteAccount(DeleteAccount),
    DelegateAction(DelegateAction),
}

macro_rules! impl_as_action_for {
    ($action_type:ident) => {
        paste::paste! {
            fn [< as_ $action_type:snake:lower >](&self) -> Option<&$action_type> {
                match self {
                    Self::$action_type(action) => Some(action),
                    _ => None,
                }
            }
        }
    };
}

impl Action for ActionKind {
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
            Self::DelegateAction(action) => action.metadata(),
        }
    }

    impl_as_action_for!(CreateAccount);
    impl_as_action_for!(DeployContract);
    impl_as_action_for!(FunctionCall);
    impl_as_action_for!(Transfer);
    impl_as_action_for!(Stake);
    impl_as_action_for!(AddKey);
    impl_as_action_for!(DeleteKey);
    impl_as_action_for!(DeleteAccount);
    impl_as_action_for!(DelegateAction);
}

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
pub struct DelegateAction {
    metadata: ActionMetadata,
    delegate_action: Vec<DelegateActionKind>,
    signature: Signature,
}

impl DelegateAction {
    pub fn delegate_action(&self) -> &[DelegateActionKind] {
        &self.delegate_action
    }

    pub fn signature(&self) -> &Signature {
        &self.signature
    }
}

#[derive(Debug, Clone)]
pub enum DelegateActionKind {
    CreateAccount(CreateAccount),
    DeployContract(DeployContract),
    FunctionCall(FunctionCall),
    Transfer(Transfer),
    Stake(Stake),
    AddKey(AddKey),
    DeleteKey(DeleteKey),
    DeleteAccount(DeleteAccount),
}

impl TryFrom<ActionKind> for DelegateActionKind {
    type Error = &'static str;

    fn try_from(action: ActionKind) -> Result<Self, Self::Error> {
        match action {
            ActionKind::CreateAccount(ca) => Ok(DelegateActionKind::CreateAccount(ca)),
            ActionKind::DeployContract(dc) => Ok(DelegateActionKind::DeployContract(dc)),
            ActionKind::FunctionCall(fc) => Ok(DelegateActionKind::FunctionCall(fc)),
            ActionKind::Transfer(t) => Ok(DelegateActionKind::Transfer(t)),
            ActionKind::Stake(s) => Ok(DelegateActionKind::Stake(s)),
            ActionKind::AddKey(ak) => Ok(DelegateActionKind::AddKey(ak)),
            ActionKind::DeleteKey(dk) => Ok(DelegateActionKind::DeleteKey(dk)),
            ActionKind::DeleteAccount(da) => Ok(DelegateActionKind::DeleteAccount(da)),
            ActionKind::DelegateAction(_) => {
                Err("Cannot convert DelegateAction to DelegateActionKind")
            }
        }
    }
}

impl ActionKind {
    // Tries to convert a ReceiptView into a vector of ActionKind.
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
                    views::ActionView::CreateAccount => ActionKind::CreateAccount(CreateAccount {
                        metadata: metadata.clone(),
                    }),
                    views::ActionView::DeployContract { code } => {
                        ActionKind::DeployContract(DeployContract {
                            metadata: metadata.clone(),
                            code: code.clone(),
                        })
                    }
                    views::ActionView::FunctionCall {
                        method_name,
                        args,
                        gas,
                        deposit,
                    } => ActionKind::FunctionCall(FunctionCall {
                        metadata: metadata.clone(),
                        method_name: method_name.clone(),
                        args: args.clone(),
                        gas: *gas,
                        deposit: *deposit,
                    }),
                    views::ActionView::Transfer { deposit } => ActionKind::Transfer(Transfer {
                        metadata: metadata.clone(),
                        deposit: *deposit,
                    }),
                    views::ActionView::Stake { stake, public_key } => ActionKind::Stake(Stake {
                        metadata: metadata.clone(),
                        stake: *stake,
                        public_key: public_key.clone(),
                    }),
                    views::ActionView::AddKey {
                        public_key,
                        access_key,
                    } => ActionKind::AddKey(AddKey {
                        metadata: metadata.clone(),
                        public_key: public_key.clone(),
                        access_key: access_key.clone(),
                    }),
                    views::ActionView::DeleteKey { public_key } => {
                        ActionKind::DeleteKey(DeleteKey {
                            metadata: metadata.clone(),
                            public_key: public_key.clone(),
                        })
                    }
                    views::ActionView::DeleteAccount { beneficiary_id } => {
                        ActionKind::DeleteAccount(DeleteAccount {
                            metadata: metadata.clone(),
                            beneficiary_id: beneficiary_id.clone(),
                        })
                    }
                    views::ActionView::Delegate {
                        delegate_action,
                        signature,
                    } => {
                        let delegate_action_kind =
                            Self::try_from_delegate_action(delegate_action, metadata.clone())?
                                .into_iter()
                                .map(TryInto::try_into)
                                .collect::<Result<Vec<DelegateActionKind>, &str>>()?;

                        ActionKind::DelegateAction(DelegateAction {
                            metadata: metadata.clone(),
                            delegate_action: delegate_action_kind,
                            signature: signature.clone(),
                        })
                    }
                };
                result.push(action_kind);
            }
            Ok(result)
        } else {
            Err("Only `ReceiptEnumView::Action` can be converted into Vec<ActionKind>")
        }
    }

    // Tries to convert a near_primitives::delegate_action::DelegateAction into a vector of ActionKind.
    pub fn try_from_delegate_action(
        delegate_action: &near_primitives::delegate_action::DelegateAction,
        metadata: ActionMetadata,
    ) -> Result<Vec<Self>, &'static str> {
        let mut actions = Vec::with_capacity(delegate_action.actions.len());

        for action in delegate_action.clone().actions {
            let action_kind = match views::ActionView::from(
                <near_primitives::delegate_action::NonDelegateAction as Into<
                    near_primitives::transaction::Action,
                >>::into(action),
            ) {
                views::ActionView::CreateAccount => ActionKind::CreateAccount(CreateAccount {
                    metadata: metadata.clone(),
                }),
                views::ActionView::DeployContract { code } => {
                    ActionKind::DeployContract(DeployContract {
                        metadata: metadata.clone(),
                        code,
                    })
                }
                views::ActionView::FunctionCall {
                    method_name,
                    args,
                    gas,
                    deposit,
                } => ActionKind::FunctionCall(FunctionCall {
                    metadata: metadata.clone(),
                    method_name,
                    args,
                    gas,
                    deposit,
                }),
                views::ActionView::Transfer { deposit } => ActionKind::Transfer(Transfer {
                    metadata: metadata.clone(),
                    deposit,
                }),
                views::ActionView::Stake { stake, public_key } => ActionKind::Stake(Stake {
                    metadata: metadata.clone(),
                    stake,
                    public_key,
                }),
                views::ActionView::AddKey {
                    public_key,
                    access_key,
                } => ActionKind::AddKey(AddKey {
                    metadata: metadata.clone(),
                    public_key,
                    access_key,
                }),
                views::ActionView::DeleteKey { public_key } => ActionKind::DeleteKey(DeleteKey {
                    metadata: metadata.clone(),
                    public_key,
                }),
                views::ActionView::DeleteAccount { beneficiary_id } => {
                    ActionKind::DeleteAccount(DeleteAccount {
                        metadata: metadata.clone(),
                        beneficiary_id,
                    })
                }
                views::ActionView::Delegate { .. } => {
                    return Err("Cannot delegate DelegateAction variant")
                }
            };
            actions.push(action_kind);
        }
        Ok(actions)
    }
}

// Macro to implement Action trait for each ActionKind variant.
macro_rules! impl_action {
    ($action:ident) => {
        impl Action for $action {
            fn metadata(&self) -> &ActionMetadata {
                &self.metadata
            }

            paste::paste! {
                fn [< as_ $action:snake:lower >](&self) -> Option<&$action> {
                    if matches!(self, $action { .. }) {
                        Some(self)
                    } else {
                        None
                    }
                }
            }
        }
    };
}

impl_action!(CreateAccount);
impl_action!(DeployContract);
impl_action!(FunctionCall);
impl_action!(Transfer);
impl_action!(Stake);
impl_action!(AddKey);
impl_action!(DeleteKey);
impl_action!(DeleteAccount);
impl_action!(DelegateAction);

// =================================================================================================

// #[derive(Debug, Clone)]
// pub struct Action {
//     receipt_id: CryptoHash,
//     predecessor_id: AccountId,
//     receiver_id: AccountId,
//     signer_id: AccountId,
//     signer_public_key: PublicKey,
//     operations: Vec<Operation>,
// }

// impl Action {
//     pub fn receipt_id(&self) -> CryptoHash {
//         self.receipt_id
//     }

//     pub fn predecessor_id(&self) -> AccountId {
//         self.predecessor_id.clone()
//     }

//     pub fn receiver_id(&self) -> AccountId {
//         self.receiver_id.clone()
//     }

//     pub fn signer_id(&self) -> AccountId {
//         self.signer_id.clone()
//     }

//     pub fn signer_public_key(&self) -> PublicKey {
//         self.signer_public_key.clone()
//     }

//     pub fn operations(&self) -> Vec<Operation> {
//         self.operations.clone()
//     }
// }

// impl TryFrom<&views::ReceiptView> for Action {
//     type Error = &'static str;

//     fn try_from(receipt: &views::ReceiptView) -> Result<Self, Self::Error> {
//         if let views::ReceiptEnumView::Action {
//             signer_id,
//             signer_public_key,
//             actions,
//             ..
//         } = &receipt.receipt
//         {
//             Ok(Self {
//                 receipt_id: receipt.receipt_id,
//                 predecessor_id: receipt.predecessor_id.clone(),
//                 receiver_id: receipt.receiver_id.clone(),
//                 signer_id: signer_id.clone(),
//                 signer_public_key: signer_public_key.clone(),
//                 operations: actions.iter().map(Into::into).collect(),
//             })
//         } else {
//             Err("Only `ReceiptEnumView::Action` can be converted into Action")
//         }
//     }
// }

// #[derive(Debug, Clone)]
// pub enum Operation {
//     CreateAccount,
//     DeployContract {
//         code: Vec<u8>,
//     },
//     FunctionCall {
//         method_name: String,
//         args: Vec<u8>,
//         gas: Gas,
//         deposit: Balance,
//     },
//     Transfer {
//         deposit: Balance,
//     },
//     Stake {
//         stake: Balance,
//         public_key: PublicKey,
//     },
//     AddKey {
//         public_key: PublicKey,
//         access_key: views::AccessKeyView,
//     },
//     DeleteKey {
//         public_key: PublicKey,
//     },
//     DeleteAccount {
//         beneficiary_id: AccountId,
//     },
//     Delegate {
//         delegate_action: DelegateAction,
//         signature: Signature,
//     },
// }

// impl From<&views::ActionView> for Operation {
//     fn from(action: &views::ActionView) -> Self {
//         match action {
//             &views::ActionView::CreateAccount => Self::CreateAccount,
//             views::ActionView::DeployContract { code } => {
//                 Self::DeployContract { code: code.clone() }
//             }
//             views::ActionView::FunctionCall {
//                 method_name,
//                 args,
//                 gas,
//                 deposit,
//             } => Self::FunctionCall {
//                 method_name: method_name.to_string(),
//                 args: args.clone(),
//                 gas: *gas,
//                 deposit: *deposit,
//             },
//             views::ActionView::Transfer { deposit } => Self::Transfer { deposit: *deposit },
//             views::ActionView::Stake { stake, public_key } => Self::Stake {
//                 stake: *stake,
//                 public_key: public_key.clone(),
//             },
//             views::ActionView::AddKey {
//                 public_key,
//                 access_key,
//             } => Self::AddKey {
//                 public_key: public_key.clone(),
//                 access_key: access_key.clone(),
//             },
//             views::ActionView::DeleteKey { public_key } => Self::DeleteKey {
//                 public_key: public_key.clone(),
//             },
//             views::ActionView::DeleteAccount { beneficiary_id } => Self::DeleteAccount {
//                 beneficiary_id: beneficiary_id.clone(),
//             },
//             views::ActionView::Delegate {
//                 delegate_action,
//                 signature,
//             } => Self::Delegate {
//                 delegate_action: delegate_action.into(),
//                 signature: signature.clone(),
//             },
//         }
//     }
// }

// #[derive(Debug, Clone)]
// pub struct DelegateAction {
//     pub sender_id: AccountId,
//     pub receiver_id: AccountId,
//     /// List of actions to be executed.
//     ///
//     /// With the meta transactions MVP defined in NEP-366, nested
//     /// DelegateActions are not allowed. A separate type is used to enforce it.
//     pub operations: Vec<NonDelegateOperation>,
//     pub nonce: u64,
//     /// The maximal height of the block in the blockchain below which the given DelegateAction is valid.
//     pub max_block_height: u64,
//     pub public_key: PublicKey,
// }

// impl From<&near_primitives::delegate_action::DelegateAction> for DelegateAction {
//     fn from(delegate_action: &near_primitives::delegate_action::DelegateAction) -> Self {
//         let operations: Vec<NonDelegateOperation> = delegate_action
//             .actions
//             .iter()
//             .cloned()
//             .map(Into::into)
//             .collect();

//         Self {
//             sender_id: delegate_action.sender_id.clone(),
//             receiver_id: delegate_action.receiver_id.clone(),
//             operations,
//             nonce: delegate_action.nonce,
//             max_block_height: delegate_action.max_block_height,
//             public_key: delegate_action.public_key.clone(),
//         }
//     }
// }

// impl From<near_primitives::delegate_action::NonDelegateAction> for NonDelegateOperation {
//     fn from(non_delegate_action: near_primitives::delegate_action::NonDelegateAction) -> Self {
//         non_delegate_action.into()
//     }
// }

// #[derive(Debug, Clone)]
// pub struct NonDelegateOperation(Operation);

// impl From<NonDelegateOperation> for Operation {
//     fn from(operation: NonDelegateOperation) -> Self {
//         operation.0
//     }
// }

// #[derive(Debug, thiserror::Error)]
// #[error("attempted to construct NonDelegateOperation from Operation::Delegate")]
// pub struct IsDelegateAction;

// impl TryFrom<Operation> for NonDelegateOperation {
//     type Error = IsDelegateAction;

//     fn try_from(operation: Operation) -> Result<Self, IsDelegateAction> {
//         if matches!(operation, Operation::Delegate { .. }) {
//             Err(IsDelegateAction)
//         } else {
//             Ok(Self(operation))
//         }
//     }
// }
