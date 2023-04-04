use near_indexer_primitives::{views, IndexerTransactionWithOutcome};

use crate::actions::{Action, ActionMetadata, DelegateAction};

impl Action {
    // Tries to convert a &ReceiptView into a vector of Action.
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
                    views::ActionView::CreateAccount => {
                        Self::CreateAccount(crate::actions::CreateAccount {
                            metadata: metadata.clone(),
                        })
                    }
                    views::ActionView::DeployContract { code } => {
                        Self::DeployContract(crate::actions::DeployContract {
                            metadata: metadata.clone(),
                            code: code.clone(),
                        })
                    }
                    views::ActionView::FunctionCall {
                        method_name,
                        args,
                        gas,
                        deposit,
                    } => Self::FunctionCall(crate::actions::FunctionCall {
                        metadata: metadata.clone(),
                        method_name: method_name.clone(),
                        args: args.clone(),
                        gas: *gas,
                        deposit: *deposit,
                    }),
                    views::ActionView::Transfer { deposit } => {
                        Self::Transfer(crate::actions::Transfer {
                            metadata: metadata.clone(),
                            deposit: *deposit,
                        })
                    }
                    views::ActionView::Stake { stake, public_key } => {
                        Self::Stake(crate::actions::Stake {
                            metadata: metadata.clone(),
                            stake: *stake,
                            public_key: public_key.clone(),
                        })
                    }
                    views::ActionView::AddKey {
                        public_key,
                        access_key,
                    } => Self::AddKey(crate::actions::AddKey {
                        metadata: metadata.clone(),
                        public_key: public_key.clone(),
                        access_key: access_key.clone(),
                    }),
                    views::ActionView::DeleteKey { public_key } => {
                        Self::DeleteKey(crate::actions::DeleteKey {
                            metadata: metadata.clone(),
                            public_key: public_key.clone(),
                        })
                    }
                    views::ActionView::DeleteAccount { beneficiary_id } => {
                        Self::DeleteAccount(crate::actions::DeleteAccount {
                            metadata: metadata.clone(),
                            beneficiary_id: beneficiary_id.clone(),
                        })
                    }
                    views::ActionView::Delegate {
                        delegate_action,
                        signature,
                    } => {
                        let delegate_actions =
                            DelegateAction::try_from_delegate_action(delegate_action)?;

                        Self::Delegate(crate::actions::Delegate {
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
                views::ActionView::CreateAccount => {
                    Self::CreateAccount(crate::actions::CreateAccount {
                        metadata: metadata.clone(),
                    })
                }
                views::ActionView::DeployContract { code } => {
                    Self::DeployContract(crate::actions::DeployContract {
                        metadata: metadata.clone(),
                        code: code.to_vec(),
                    })
                }
                views::ActionView::FunctionCall {
                    method_name,
                    args,
                    gas,
                    deposit,
                } => Self::FunctionCall(crate::actions::FunctionCall {
                    metadata: metadata.clone(),
                    method_name: method_name.to_string(),
                    args: args.to_vec(),
                    gas: *gas,
                    deposit: *deposit,
                }),
                views::ActionView::Transfer { deposit } => {
                    Self::Transfer(crate::actions::Transfer {
                        metadata: metadata.clone(),
                        deposit: *deposit,
                    })
                }
                views::ActionView::Stake { stake, public_key } => {
                    Self::Stake(crate::actions::Stake {
                        metadata: metadata.clone(),
                        stake: *stake,
                        public_key: public_key.clone(),
                    })
                }
                views::ActionView::AddKey {
                    public_key,
                    access_key,
                } => Self::AddKey(crate::actions::AddKey {
                    metadata: metadata.clone(),
                    public_key: public_key.clone(),
                    access_key: access_key.clone(),
                }),
                views::ActionView::DeleteKey { public_key } => {
                    Self::DeleteKey(crate::actions::DeleteKey {
                        metadata: metadata.clone(),
                        public_key: public_key.clone(),
                    })
                }
                views::ActionView::DeleteAccount { beneficiary_id } => {
                    Self::DeleteAccount(crate::actions::DeleteAccount {
                        metadata: metadata.clone(),
                        beneficiary_id: beneficiary_id.clone(),
                    })
                }
                views::ActionView::Delegate {
                    delegate_action,
                    signature,
                } => Self::Delegate(crate::actions::Delegate {
                    metadata: metadata.clone(),
                    delegate_action: DelegateAction::try_from_delegate_action(delegate_action)?,
                    signature: signature.clone(),
                }),
            };

            actions.push(action);
        }

        Ok(actions)
    }
}
