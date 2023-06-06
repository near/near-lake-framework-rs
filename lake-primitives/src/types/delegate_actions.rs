use near_crypto::PublicKey;
use near_indexer_primitives::{
    types::{AccountId, Balance, Gas},
    views::{self, AccessKeyView},
};

/// Similarly to the [Action](super::actions::Action) enum, this enum represents the different types of actions that can be
/// delegated to a contract.
///
/// `DelegateAction` enum has a corresponding `Action` variant for every possible `Action` except the `DelegateAction` itself.
/// Thus forbidding the nesting of `DelegateActions` and making the `Action` enum exhaustive.
/// Another difference is that `DelegateAction` itself and it's variants do not hold metadata and don't implement `ActionMetaDataExt`.
#[derive(Debug, Clone)]
pub enum DelegateAction {
    DelegateCreateAccount(DelegateCreateAccount),
    DelegateDeployContract(DelegateDeployContract),
    DelegateFunctionCall(DelegateFunctionCall),
    DelegateTransfer(DelegateTransfer),
    DelegateStake(DelegateStake),
    DelegateAddKey(DelegateAddKey),
    DelegateDeleteKey(DelegateDeleteKey),
    DelegateDeleteAccount(DelegateDeleteAccount),
}

impl DelegateAction {
    /// Attempts to return the [DelegateFunctionCall](struct@DelegateFunctionCall) struct if the variant is [DelegateAction::DelegateFunctionCall]. Otherwise returns `None`.
    pub fn as_delegate_function_call(&self) -> Option<&DelegateFunctionCall> {
        match self {
            DelegateAction::DelegateFunctionCall(action) => Some(action),
            _ => None,
        }
    }

    /// Attempts to return the [DelegateCreateAccount] struct if the variant is [DelegateAction::DelegateCreateAccount]. Otherwise returns `None`.
    pub fn as_delegate_create_account(&self) -> Option<&DelegateCreateAccount> {
        match self {
            DelegateAction::DelegateCreateAccount(action) => Some(action),
            _ => None,
        }
    }

    /// Attempts to return the [DelegateDeployContract] struct if the variant is [DelegateAction::DelegateDeployContract]. Otherwise returns `None`.
    pub fn as_delegate_deploy_contract(&self) -> Option<&DelegateDeployContract> {
        match self {
            DelegateAction::DelegateDeployContract(action) => Some(action),
            _ => None,
        }
    }

    /// Attempts to return the [DelegateTransfer] struct if the variant is [DelegateAction::DelegateTransfer]. Otherwise returns `None`.
    pub fn as_delegate_transfer(&self) -> Option<&DelegateTransfer> {
        match self {
            DelegateAction::DelegateTransfer(action) => Some(action),
            _ => None,
        }
    }

    /// Attempts to return the [DelegateStake] struct if the variant is [DelegateAction::DelegateStake]. Otherwise returns `None`.
    pub fn as_delegate_stake(&self) -> Option<&DelegateStake> {
        match self {
            DelegateAction::DelegateStake(action) => Some(action),
            _ => None,
        }
    }

    /// Attempts to return the [DelegateAddKey] struct if the variant is [DelegateAction::DelegateAddKey]. Otherwise returns `None`.
    pub fn as_delegate_add_key(&self) -> Option<&DelegateAddKey> {
        match self {
            DelegateAction::DelegateAddKey(action) => Some(action),
            _ => None,
        }
    }

    /// Attempts to return the [DelegateDeleteKey] struct if the variant is [DelegateAction::DelegateDeleteKey]. Otherwise returns `None`.
    pub fn as_delegate_delete_key(&self) -> Option<&DelegateDeleteKey> {
        match self {
            DelegateAction::DelegateDeleteKey(action) => Some(action),
            _ => None,
        }
    }

    /// Attempts to return the [DelegateDeleteAccount] struct if the variant is [DelegateAction::DelegateDeleteAccount]. Otherwise returns `None`.
    pub fn as_delegate_delete_account(&self) -> Option<&DelegateDeleteAccount> {
        match self {
            DelegateAction::DelegateDeleteAccount(action) => Some(action),
            _ => None,
        }
    }
}

/// Similarly to [CreateAccount](super::actions::CreateAccount), this struct represents the `CreateAccount` action that is delegated.
#[derive(Debug, Clone)]
pub struct DelegateCreateAccount;

/// Similarly to [DeployContract](super::actions::DeployContract), this struct represents the `DeployContract` action that is delegated.
#[derive(Debug, Clone)]
pub struct DelegateDeployContract {
    pub(crate) code: Vec<u8>,
}

impl DelegateDeployContract {
    /// Returns the bytes of the contract code that is being deployed.
    pub fn code(&self) -> &[u8] {
        &self.code
    }
}

/// Similarly to [FunctionCall](super::actions::FunctionCall), this struct represents the `FunctionCall` action that is delegated.
#[derive(Debug, Clone)]
pub struct DelegateFunctionCall {
    pub(crate) method_name: String,
    pub(crate) args: Vec<u8>,
    pub(crate) gas: Gas,
    pub(crate) deposit: Balance,
}

impl DelegateFunctionCall {
    /// Returns the name of the method that is being called.
    pub fn method_name(&self) -> &str {
        &self.method_name
    }

    /// Returns the bytes of the arguments that are being passed to the method.
    pub fn args(&self) -> &[u8] {
        &self.args
    }

    /// Returns the amount of gas that is being used for the method call.
    pub fn gas(&self) -> Gas {
        self.gas
    }

    /// Returns the amount of tokens that are being deposited to the contract.
    pub fn deposit(&self) -> Balance {
        self.deposit
    }
}

/// Similarly to [Transfer](super::actions::Transfer), this struct represents the `Transfer` action that is delegated.
#[derive(Debug, Clone)]
pub struct DelegateTransfer {
    pub(crate) deposit: Balance,
}

impl DelegateTransfer {
    /// Returns the amount of tokens that are being transferred.
    pub fn deposit(&self) -> Balance {
        self.deposit
    }
}

/// Similarly to [Stake](super::actions::Stake), this struct represents the `Stake` action that is delegated.
#[derive(Debug, Clone)]
pub struct DelegateStake {
    pub(crate) stake: Balance,
    pub(crate) public_key: PublicKey,
}

impl DelegateStake {
    /// Returns the amount of tokens that are being staked.
    pub fn stake(&self) -> Balance {
        self.stake
    }

    /// Returns the public key of the staking pool.
    pub fn public_key(&self) -> &PublicKey {
        &self.public_key
    }
}

/// Similarly to [AddKey](super::actions::AddKey), this struct represents the `AddKey` action that is delegated.
#[derive(Debug, Clone)]
pub struct DelegateAddKey {
    pub(crate) public_key: PublicKey,
    pub(crate) access_key: AccessKeyView,
}

impl DelegateAddKey {
    /// Returns the public key that is being added.
    pub fn public_key(&self) -> &PublicKey {
        &self.public_key
    }

    /// Returns the access key that is being added.
    pub fn access_key(&self) -> &AccessKeyView {
        &self.access_key
    }
}

/// Similarly to [DeleteKey](super::actions::DeleteKey), this struct represents the `DeleteKey` action that is delegated.
#[derive(Debug, Clone)]
pub struct DelegateDeleteKey {
    pub(crate) public_key: PublicKey,
}

impl DelegateDeleteKey {
    /// Returns the public key that is being deleted.
    pub fn public_key(&self) -> &PublicKey {
        &self.public_key
    }
}

/// Similarly to [DeleteAccount](super::actions::DeleteAccount), this struct represents the `DeleteAccount` action that is delegated.
#[derive(Debug, Clone)]
pub struct DelegateDeleteAccount {
    pub(crate) beneficiary_id: AccountId,
}

impl DelegateDeleteAccount {
    /// Returns the account ID of the beneficiary.
    pub fn beneficiary_id(&self) -> &AccountId {
        &self.beneficiary_id
    }
}

impl DelegateAction {
    // Tries to convert a `near_primitives::delegate_action::DelegateAction` into a [Vec<DelegateAction>].
    pub fn try_from_delegate_action(
        delegate_action: &near_primitives::delegate_action::DelegateAction,
    ) -> Result<Vec<Self>, &'static str> {
        let mut actions = Vec::with_capacity(delegate_action.actions.len());

        for nearcore_action in delegate_action.clone().actions {
            let action = match views::ActionView::from(
                <near_primitives::delegate_action::NonDelegateAction as Into<
                    near_primitives::transaction::Action,
                >>::into(nearcore_action),
            ) {
                views::ActionView::CreateAccount => {
                    Self::DelegateCreateAccount(DelegateCreateAccount)
                }
                views::ActionView::DeployContract { code } => {
                    Self::DelegateDeployContract(DelegateDeployContract { code })
                }
                views::ActionView::FunctionCall {
                    method_name,
                    args,
                    gas,
                    deposit,
                } => Self::DelegateFunctionCall(DelegateFunctionCall {
                    method_name,
                    args: args.into(),
                    gas,
                    deposit,
                }),
                views::ActionView::Transfer { deposit } => {
                    Self::DelegateTransfer(DelegateTransfer { deposit })
                }
                views::ActionView::Stake { stake, public_key } => {
                    Self::DelegateStake(DelegateStake { stake, public_key })
                }
                views::ActionView::AddKey {
                    public_key,
                    access_key,
                } => Self::DelegateAddKey(DelegateAddKey {
                    public_key,
                    access_key,
                }),
                views::ActionView::DeleteKey { public_key } => {
                    Self::DelegateDeleteKey(DelegateDeleteKey { public_key })
                }
                views::ActionView::DeleteAccount { beneficiary_id } => {
                    Self::DelegateDeleteAccount(DelegateDeleteAccount { beneficiary_id })
                }
                _ => return Err("Cannot delegate DelegateAction"),
            };
            actions.push(action);
        }
        Ok(actions)
    }
}
