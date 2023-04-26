use near_crypto::PublicKey;
use near_indexer_primitives::{
    types::{AccountId, Balance, Gas},
    views::{self, AccessKeyView},
};

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
    pub fn as_delegate_function_call(&self) -> Option<&DelegateFunctionCall> {
        match self {
            DelegateAction::DelegateFunctionCall(action) => Some(action),
            _ => None,
        }
    }

    pub fn as_delegate_create_account(&self) -> Option<&DelegateCreateAccount> {
        match self {
            DelegateAction::DelegateCreateAccount(action) => Some(action),
            _ => None,
        }
    }

    pub fn as_delegate_deploy_contract(&self) -> Option<&DelegateDeployContract> {
        match self {
            DelegateAction::DelegateDeployContract(action) => Some(action),
            _ => None,
        }
    }

    pub fn as_delegate_transfer(&self) -> Option<&DelegateTransfer> {
        match self {
            DelegateAction::DelegateTransfer(action) => Some(action),
            _ => None,
        }
    }

    pub fn as_delegate_stake(&self) -> Option<&DelegateStake> {
        match self {
            DelegateAction::DelegateStake(action) => Some(action),
            _ => None,
        }
    }

    pub fn as_delegate_add_key(&self) -> Option<&DelegateAddKey> {
        match self {
            DelegateAction::DelegateAddKey(action) => Some(action),
            _ => None,
        }
    }

    pub fn as_delegate_delete_key(&self) -> Option<&DelegateDeleteKey> {
        match self {
            DelegateAction::DelegateDeleteKey(action) => Some(action),
            _ => None,
        }
    }

    pub fn as_delegate_delete_account(&self) -> Option<&DelegateDeleteAccount> {
        match self {
            DelegateAction::DelegateDeleteAccount(action) => Some(action),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DelegateCreateAccount;

#[derive(Debug, Clone)]
pub struct DelegateDeployContract {
    pub(crate) code: Vec<u8>,
}

impl DelegateDeployContract {
    pub fn code(&self) -> &[u8] {
        &self.code
    }
}

#[derive(Debug, Clone)]
pub struct DelegateFunctionCall {
    pub(crate) method_name: String,
    pub(crate) args: Vec<u8>,
    pub(crate) gas: Gas,
    pub(crate) deposit: Balance,
}

impl DelegateFunctionCall {
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
pub struct DelegateTransfer {
    pub(crate) deposit: Balance,
}

impl DelegateTransfer {
    pub fn deposit(&self) -> Balance {
        self.deposit
    }
}

#[derive(Debug, Clone)]
pub struct DelegateStake {
    pub(crate) stake: Balance,
    pub(crate) public_key: PublicKey,
}

impl DelegateStake {
    pub fn stake(&self) -> Balance {
        self.stake
    }

    pub fn public_key(&self) -> &PublicKey {
        &self.public_key
    }
}

#[derive(Debug, Clone)]
pub struct DelegateAddKey {
    pub(crate) public_key: PublicKey,
    pub(crate) access_key: AccessKeyView,
}

impl DelegateAddKey {
    pub fn public_key(&self) -> &PublicKey {
        &self.public_key
    }

    pub fn access_key(&self) -> &AccessKeyView {
        &self.access_key
    }
}

#[derive(Debug, Clone)]
pub struct DelegateDeleteKey {
    pub(crate) public_key: PublicKey,
}

impl DelegateDeleteKey {
    pub fn public_key(&self) -> &PublicKey {
        &self.public_key
    }
}

#[derive(Debug, Clone)]
pub struct DelegateDeleteAccount {
    pub(crate) beneficiary_id: AccountId,
}

impl DelegateDeleteAccount {
    pub fn beneficiary_id(&self) -> &AccountId {
        &self.beneficiary_id
    }
}

impl DelegateAction {
    // Tries to convert a near_primitives::delegate_action::DelegateAction into a Vec<DelegateAction>.
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
                    args,
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
