use near_crypto::{PublicKey, Signature};
use near_indexer_primitives::{
    types::{AccountId, Balance, Gas},
    views, CryptoHash,
};

use crate::types::delegate_actions;
pub use delegate_actions::DelegateAction;

#[derive(Debug, Clone)]
pub struct ActionMetadata {
    pub(crate) receipt_id: CryptoHash,
    pub(crate) predecessor_id: AccountId,
    pub(crate) receiver_id: AccountId,
    pub(crate) signer_id: AccountId,
    pub(crate) signer_public_key: PublicKey,
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
    pub(crate) metadata: ActionMetadata,
}

#[derive(Debug, Clone)]
pub struct DeployContract {
    pub(crate) metadata: ActionMetadata,
    pub(crate) code: Vec<u8>,
}

impl DeployContract {
    pub fn code(&self) -> &[u8] {
        &self.code
    }
}

#[derive(Debug, Clone)]
pub struct FunctionCall {
    pub(crate) metadata: ActionMetadata,
    pub(crate) method_name: String,
    pub(crate) args: Vec<u8>,
    pub(crate) gas: Gas,
    pub(crate) deposit: Balance,
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
    pub(crate) metadata: ActionMetadata,
    pub(crate) deposit: Balance,
}

impl Transfer {
    pub fn deposit(&self) -> Balance {
        self.deposit
    }
}

#[derive(Debug, Clone)]
pub struct Stake {
    pub(crate) metadata: ActionMetadata,
    pub(crate) stake: Balance,
    pub(crate) public_key: PublicKey,
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
    pub(crate) metadata: ActionMetadata,
    pub(crate) public_key: PublicKey,
    pub(crate) access_key: views::AccessKeyView,
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
    pub(crate) metadata: ActionMetadata,
    pub(crate) public_key: PublicKey,
}

impl DeleteKey {
    pub fn public_key(&self) -> &PublicKey {
        &self.public_key
    }
}

#[derive(Debug, Clone)]
pub struct DeleteAccount {
    pub(crate) metadata: ActionMetadata,
    pub(crate) beneficiary_id: AccountId,
}

impl DeleteAccount {
    pub fn beneficiary_id(&self) -> &AccountId {
        &self.beneficiary_id
    }
}

#[derive(Debug, Clone)]
pub struct Delegate {
    pub(crate) metadata: ActionMetadata,
    pub(crate) delegate_action: Vec<delegate_actions::DelegateAction>,
    pub(crate) signature: Signature,
}

impl Delegate {
    pub fn delegate_action(&self) -> &[delegate_actions::DelegateAction] {
        &self.delegate_action
    }

    pub fn signature(&self) -> &Signature {
        &self.signature
    }
}
