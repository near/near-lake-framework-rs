use near_crypto::{PublicKey, Signature};
use near_indexer_primitives::{
    types::{AccountId, Balance, Gas},
    views, CryptoHash,
};

use crate::types::delegate_actions;
pub use delegate_actions::{
    DelegateAction, DelegateAddKey, DelegateCreateAccount, DelegateDeleteAccount,
    DelegateDeleteKey, DelegateDeployContract, DelegateFunctionCall, DelegateStake,
    DelegateTransfer,
};

/// Represents the metadata of the action.
/// This is the information that is common to all actions.
#[derive(Debug, Clone)]
pub struct ActionMetadata {
    pub(crate) receipt_id: CryptoHash,
    pub(crate) predecessor_id: AccountId,
    pub(crate) receiver_id: AccountId,
    pub(crate) signer_id: AccountId,
    pub(crate) signer_public_key: PublicKey,
}

impl ActionMetadata {
    /// Returns the [CryptoHash] id of the corresponding Receipt.
    pub fn receipt_id(&self) -> CryptoHash {
        self.receipt_id
    }

    /// Returns the [AccountId] of the predecessor of the action.
    pub fn predecessor_id(&self) -> AccountId {
        self.predecessor_id.clone()
    }

    /// Returns the [AccountId] of the receiver of the action.
    pub fn receiver_id(&self) -> AccountId {
        self.receiver_id.clone()
    }

    /// Returns the [AccountId] of the signer of the action.
    pub fn signer_id(&self) -> AccountId {
        self.signer_id.clone()
    }

    /// Returns the [PublicKey] of the signer of the action.
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

/// High-level representation of the `Action`.
///
/// Action is "registered" in the [Transaction](super::transactions::Transaction) to be performed on the blockchain.
/// There is a predefined set of actions that can be performed on the blockchain.
///
/// #### Important notes on Action enum
///
/// Please, note that each enum variant is a wrapper around the corresponding action struct. Also, we have special methods
/// for each action type that attempts to convert the action to the corresponding struct. For example, if you have an action
/// of type `Action::Transfer`, you can call `action.as_transfer()` to get the `Transfer` struct. If the action is not of
/// the corresponding type, the method will return `None`. This is done to simplify the usage of the `Action` enum.
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

/// Structure representing the `CreateAccount` action.
/// This is a special action that is used to create a new account on the blockchain. It doesn't contain any
/// additional data. The `receiver_id` from the metadata is the name of the account that is created by this action.
#[derive(Debug, Clone)]
pub struct CreateAccount {
    pub(crate) metadata: ActionMetadata,
}

/// Structure representing the `DeployContract` action.
#[derive(Debug, Clone)]
pub struct DeployContract {
    pub(crate) metadata: ActionMetadata,
    pub(crate) code: Vec<u8>,
}

impl DeployContract {
    /// Returns the contract code bytes.
    pub fn code(&self) -> &[u8] {
        &self.code
    }
}

/// Structure representing the `FunctionCall` action.
#[derive(Debug, Clone)]
pub struct FunctionCall {
    pub(crate) metadata: ActionMetadata,
    pub(crate) method_name: String,
    pub(crate) args: Vec<u8>,
    pub(crate) gas: Gas,
    pub(crate) deposit: Balance,
}

impl FunctionCall {
    /// Returns the method name this FunctionCall calls.
    pub fn method_name(&self) -> &str {
        &self.method_name
    }

    /// Returns the arguments bytes.
    pub fn args(&self) -> &[u8] {
        &self.args
    }

    /// Returns the gas attached to this FunctionCall.
    pub fn gas(&self) -> Gas {
        self.gas
    }

    /// Returns the deposit attached to this FunctionCall.
    pub fn deposit(&self) -> Balance {
        self.deposit
    }
}

/// Structure representing the `Transfer` action.
#[derive(Debug, Clone)]
pub struct Transfer {
    pub(crate) metadata: ActionMetadata,
    pub(crate) deposit: Balance,
}

impl Transfer {
    /// Returns the deposit attached to this Transfer.
    pub fn deposit(&self) -> Balance {
        self.deposit
    }
}

/// Structure representing the `Stake` action.
#[derive(Debug, Clone)]
pub struct Stake {
    pub(crate) metadata: ActionMetadata,
    pub(crate) stake: Balance,
    pub(crate) public_key: PublicKey,
}

impl Stake {
    /// Returns the stake attached to this Stake.
    pub fn stake(&self) -> Balance {
        self.stake
    }

    /// Returns the public key attached to this Stake.
    pub fn public_key(&self) -> &PublicKey {
        &self.public_key
    }
}

/// Structure representing the `AddKey` action.
#[derive(Debug, Clone)]
pub struct AddKey {
    pub(crate) metadata: ActionMetadata,
    pub(crate) public_key: PublicKey,
    pub(crate) access_key: views::AccessKeyView,
}

impl AddKey {
    /// Returns the [PublicKey] added with this AddKey.
    pub fn public_key(&self) -> &PublicKey {
        &self.public_key
    }

    /// Returns the [AccessKey](views::AccessKeyView) to the PublicKey being added with this AddKey.
    pub fn access_key(&self) -> &views::AccessKeyView {
        &self.access_key
    }
}

/// Structure representing the `DeleteKey` action.
#[derive(Debug, Clone)]
pub struct DeleteKey {
    pub(crate) metadata: ActionMetadata,
    pub(crate) public_key: PublicKey,
}

impl DeleteKey {
    /// Returns the [PublicKey] deleted with this DeleteKey.
    pub fn public_key(&self) -> &PublicKey {
        &self.public_key
    }
}

/// Structure representing the `DeleteAccount` action.
#[derive(Debug, Clone)]
pub struct DeleteAccount {
    pub(crate) metadata: ActionMetadata,
    pub(crate) beneficiary_id: AccountId,
}

impl DeleteAccount {
    /// Returns the beneficiary account ID of this DeleteAccount.
    pub fn beneficiary_id(&self) -> &AccountId {
        &self.beneficiary_id
    }
}

/// Structure representing the `Delegate` action.
/// This is related to the Meta-Transactions [NEP-366](https://github.com/near/NEPs/blob/master/neps/nep-0366.md).
///
/// This action is used to delegate the right to sign transactions on behalf of the signer to another account.
/// The signer is the account that is signing the transaction that contains this action.
/// The receiver is the account that will be able to sign transactions on behalf of the signer.
/// The `delegate_action` is the action that the receiver will be able to sign on behalf of the signer.
/// The `signature` is the signature of the signer on the hash of the `delegate_action`.
///
/// The `delegate_action` can be any action, except for another `Delegate` action. Thus not allowing the nesting of `Delegate` actions.
#[derive(Debug, Clone)]
pub struct Delegate {
    pub(crate) metadata: ActionMetadata,
    pub(crate) delegate_action: Vec<delegate_actions::DelegateAction>,
    pub(crate) signature: Signature,
}

impl Delegate {
    /// Returns the delegate action that the receiver will be able to sign on behalf of the signer.
    pub fn delegate_action(&self) -> &[delegate_actions::DelegateAction] {
        &self.delegate_action
    }

    /// Returns the signature of the signer on the hash of the `delegate_action`.
    pub fn signature(&self) -> &Signature {
        &self.signature
    }
}
