// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    proto::types as jsonrpc,
    views::{
        AccountRoleView, AccountStateProofView, AccountStateWithProofView, AccountView, AmountView,
        BytesView, EventDataView, EventView, MetadataView, MoveAbortExplanationView, ScriptView,
        TransactionDataView, TransactionView, VMStatusView,
    },
};
use anyhow::{bail, Result};
use diem_types::account_state::AccountState;
use std::convert::{TryFrom, TryInto};

impl TryFrom<jsonrpc::Event> for EventView {
    type Error = anyhow::Error;

    fn try_from(item: jsonrpc::Event) -> Result<EventView> {
        Ok(EventView {
            key: item.key.clone().into(),
            sequence_number: item.sequence_number,
            transaction_version: item.transaction_version,
            data: match item.data {
                Some(event_data) => event_data.try_into()?,
                None => bail!("event.data is empty for event={:?}", item),
            },
        })
    }
}

impl TryFrom<jsonrpc::EventData> for EventDataView {
    type Error = anyhow::Error;

    fn try_from(item: jsonrpc::EventData) -> Result<Self> {
        match item.r#type.as_str() {
            "burn" => Ok(Self::Burn {
                amount: item.amount.unwrap().into(),
                preburn_address: item.preburn_address.into(),
            }),
            "cancelburn" => Ok(Self::CancelBurn {
                amount: item.amount.unwrap().into(),
                preburn_address: item.preburn_address.into(),
            }),
            "mint" => Ok(Self::Mint {
                amount: item.amount.unwrap().into(),
            }),
            "to_xdx_exchange_rate_update" => Ok(Self::ToXDXExchangeRateUpdate {
                currency_code: item.currency_code,
                new_to_xdx_exchange_rate: item.new_to_xdx_exchange_rate,
            }),
            "preburn" => Ok(Self::Preburn {
                amount: item.amount.unwrap().into(),
                preburn_address: item.preburn_address.into(),
            }),
            "receivedpayment" => Ok(Self::ReceivedPayment {
                amount: item.amount.unwrap().into(),
                sender: item.sender.into(),
                receiver: item.receiver.into(),
                metadata: item.metadata.into(),
            }),
            "sendpayment" => Ok(Self::SentPayment {
                amount: item.amount.unwrap().into(),
                sender: item.sender.into(),
                receiver: item.receiver.into(),
                metadata: item.metadata.into(),
            }),
            "admintransaction" => Ok(Self::AdminTransaction {
                committed_timestamp_secs: item.committed_timestamp_secs,
            }),
            "newepoch" => Ok(Self::NewEpoch { epoch: item.epoch }),
            "newblock" => Ok(Self::NewBlock {
                round: item.round,
                proposer: item.proposer.into(),
                proposed_time: item.proposed_time,
            }),
            "receivedmint" => Ok(Self::ReceivedMint {
                amount: item.amount.unwrap().into(),
                destination_address: item.destination_address.into(),
            }),
            "compliancekeyrotation" => Ok(Self::ComplianceKeyRotation {
                new_compliance_public_key: item.new_compliance_public_key.into(),
                time_rotated_seconds: item.time_rotated_seconds,
            }),
            "baseurlrotation" => Ok(Self::BaseUrlRotation {
                new_base_url: item.new_base_url,
                time_rotated_seconds: item.time_rotated_seconds,
            }),
            "create_account" => Ok(Self::CreateAccount {
                created_address: item.created_address.into(),
                role_id: item.role_id,
            }),
            other => {
                bail!("Invalid event data type: {}", other)
            }
        }
    }
}

impl From<jsonrpc::Amount> for AmountView {
    fn from(item: jsonrpc::Amount) -> Self {
        Self {
            amount: item.amount,
            currency: item.currency,
        }
    }
}

impl From<String> for BytesView {
    fn from(item: String) -> Self {
        Self(item)
    }
}

impl TryFrom<jsonrpc::Account> for AccountView {
    type Error = anyhow::Error;

    fn try_from(item: jsonrpc::Account) -> Result<Self> {
        Ok(Self {
            address: item.address.into(),
            balances: item.balances.into_iter().map(AmountView::from).collect(),
            sequence_number: item.sequence_number,
            authentication_key: item.authentication_key.into(),
            sent_events_key: item.sent_events_key.into(),
            received_events_key: item.received_events_key.into(),
            delegated_key_rotation_capability: item.delegated_key_rotation_capability,
            delegated_withdrawal_capability: item.delegated_withdrawal_capability,
            is_frozen: item.is_frozen,
            role: match item.role {
                Some(acct) => acct.try_into()?,
                None => AccountRoleView::Unknown,
            },
        })
    }
}

impl TryFrom<jsonrpc::AccountRole> for AccountRoleView {
    type Error = anyhow::Error;

    fn try_from(item: jsonrpc::AccountRole) -> Result<Self> {
        match item.r#type.as_str() {
            "unknown" => Ok(AccountRoleView::Unknown),
            "child_vasp" => Ok(AccountRoleView::ChildVASP {
                parent_vasp_address: item.parent_vasp_address.into(),
            }),
            "parent_vasp" => Ok(AccountRoleView::ParentVASP {
                human_name: item.human_name,
                base_url: item.base_url,
                expiration_time: item.expiration_time,
                compliance_key: item.compliance_key.into(),
                num_children: item.num_children,
                compliance_key_rotation_events_key: item.compliance_key_rotation_events_key.into(),
                base_url_rotation_events_key: item.base_url_rotation_events_key.into(),
            }),
            "designated_dealer" => Ok(AccountRoleView::DesignatedDealer {
                human_name: item.human_name,
                base_url: item.base_url,
                expiration_time: item.expiration_time,
                compliance_key: item.compliance_key.into(),
                preburn_balances: item
                    .preburn_balances
                    .into_iter()
                    .map(AmountView::from)
                    .collect(),
                received_mint_events_key: item.received_mint_events_key.into(),
                compliance_key_rotation_events_key: item.compliance_key_rotation_events_key.into(),
                base_url_rotation_events_key: item.base_url_rotation_events_key.into(),
            }),
            _ => bail!("Invalid account role type for: {:?}", item),
        }
    }
}

impl TryFrom<jsonrpc::AccountStateWithProof> for AccountStateWithProofView {
    type Error = anyhow::Error;

    fn try_from(item: jsonrpc::AccountStateWithProof) -> Result<Self> {
        Ok(Self {
            version: item.version,
            blob: Option::Some(item.blob.clone().into()),
            proof: match item.proof {
                Some(proof) => proof.into(),
                None => bail!(
                    "AccountStateWithProof does not have required field proof. item={:?}",
                    item
                ),
            },
        })
    }
}

impl From<jsonrpc::AccountStateProof> for AccountStateProofView {
    fn from(item: jsonrpc::AccountStateProof) -> Self {
        Self {
            ledger_info_to_transaction_info_proof: item
                .ledger_info_to_transaction_info_proof
                .into(),
            transaction_info: item.transaction_info.into(),
            transaction_info_to_account_proof: item.transaction_info_to_account_proof.into(),
        }
    }
}

impl TryFrom<AccountStateWithProofView> for AccountState {
    type Error = anyhow::Error;

    fn try_from(item: AccountStateWithProofView) -> Result<Self> {
        if let Some(account_state_bytes) = item.blob {
            let account_blob = lcs::from_bytes(&account_state_bytes.into_bytes()?)?;
            return Ok(Self::try_from(&account_blob)?);
        }
        bail!("No account state blob at {:?}", item)
    }
}

impl From<jsonrpc::Metadata> for MetadataView {
    fn from(item: jsonrpc::Metadata) -> Self {
        Self {
            version: item.version,
            accumulator_root_hash: BytesView(item.accumulator_root_hash),
            timestamp: item.timestamp,
            chain_id: item.chain_id as u8,
            script_hash_allow_list: Option::Some(
                item.script_hash_allow_list
                    .into_iter()
                    .map(BytesView::from)
                    .collect(),
            ),
            module_publishing_allowed: Option::Some(item.module_publishing_allowed),
            diem_version: Option::Some(item.diem_version),
            dual_attestation_limit: Option::Some(item.dual_attestation_limit),
        }
    }
}

impl TryFrom<jsonrpc::VmStatus> for VMStatusView {
    type Error = anyhow::Error;

    fn try_from(item: jsonrpc::VmStatus) -> Result<Self> {
        match item.r#type.as_str() {
            "executed" => Ok(Self::Executed),
            "out_of_gas" => Ok(Self::OutOfGas),
            "move_abort" => Ok(Self::MoveAbort {
                location: item.location,
                abort_code: item.abort_code,
                explanation: item.explanation.map(|e| e.into()),
            }),
            _ => bail!("Invalid VMStatus type for item={:?}", item),
        }
    }
}

impl From<jsonrpc::MoveAbortExplaination> for MoveAbortExplanationView {
    fn from(item: jsonrpc::MoveAbortExplaination) -> Self {
        Self {
            category: item.category,
            category_description: item.category_description,
            reason: item.reason,
            reason_description: item.reason_description,
        }
    }
}

impl TryFrom<jsonrpc::Transaction> for TransactionView {
    type Error = anyhow::Error;

    fn try_from(item: jsonrpc::Transaction) -> Result<Self> {
        Ok(Self {
            version: item.version,
            transaction: match item.transaction.clone() {
                Some(transaction) => transaction.try_into()?,
                None => bail!(
                    "Required field transaction is missing for transaction={:?}",
                    item
                ),
            },
            hash: item.hash.clone().into(),
            bytes: item.bytes.clone().into(),
            events: item
                .events
                .clone()
                .into_iter()
                .map(EventView::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            vm_status: match item.vm_status.clone() {
                Some(status) => status.try_into()?,
                None => bail!(
                    "Required field vm_status not present for transaction={:?}",
                    item
                ),
            },
            gas_used: item.gas_used,
        })
    }
}

impl TryFrom<jsonrpc::TransactionData> for TransactionDataView {
    type Error = anyhow::Error;

    fn try_from(item: jsonrpc::TransactionData) -> Result<Self> {
        match item.r#type.as_str() {
            "blockmetadata" => Ok(Self::BlockMetadata {
                timestamp_usecs: item.timestamp_usecs,
            }),
            "writeset" => Ok(Self::WriteSet {}),
            "user" => Ok(Self::UserTransaction {
                sender: item.sender.into(),
                signature_scheme: item.signature_scheme,
                signature: item.signature.into(),
                public_key: item.public_key.into(),
                sequence_number: item.sequence_number,
                chain_id: item.chain_id as u8,
                max_gas_amount: item.max_gas_amount,
                gas_unit_price: item.gas_unit_price,
                gas_currency: item.gas_currency,
                expiration_timestamp_secs: item.expiration_timestamp_secs,
                script_hash: item.script_hash.into(),
                script_bytes: item.script_bytes.into(),
                script: item.script.unwrap().into(),
            }),
            "unknown" => Ok(Self::UnknownTransaction {}),
            _ => bail!("Invalid TransactionData type for item={:?}", item),
        }
    }
}

impl From<jsonrpc::Script> for ScriptView {
    fn from(item: jsonrpc::Script) -> Self {
        Self {
            r#type: item.r#type,
            code: Some(item.code.into()),
            arguments: Some(item.arguments),
            type_arguments: Some(item.type_arguments),
            receiver: Some(item.receiver.into()),
            amount: Some(item.amount),
            currency: Some(item.currency),
            metadata: Some(item.metadata.into()),
            metadata_signature: Some(item.metadata_signature.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::views::EventView;

    #[test]
    fn test_event_conversion() {
        let expected = EventView {
            key: BytesView("key".to_string()),
            sequence_number: 1,
            transaction_version: 2,
            data: EventDataView::ToXDXExchangeRateUpdate {
                currency_code: "USD".to_string(),
                new_to_xdx_exchange_rate: 1.0,
            },
        };
        let source = jsonrpc::Event {
            key: "key".to_string(),
            sequence_number: 1,
            transaction_version: 2,
            data: Some(jsonrpc::EventData {
                r#type: "to_xdx_exchange_rate_update".to_string(),
                currency_code: "USD".to_string(),
                new_to_xdx_exchange_rate: 1.0,
                amount: None,
                preburn_address: String::new(),
                sender: String::new(),
                receiver: String::new(),
                metadata: String::new(),
                epoch: 0,
                round: 0,
                proposer: String::new(),
                proposed_time: 0,
                destination_address: String::new(),
                new_compliance_public_key: String::new(),
                new_base_url: String::new(),
                time_rotated_seconds: 0,
                created_address: String::new(),
                role_id: 0,
                committed_timestamp_secs: 0,
            }),
        };
        assert_eq!(expected, source.clone().try_into().unwrap());
        let mut event_data = source.data.unwrap();
        event_data.r#type = String::from("invalid");
        let result: Result<EventDataView> = event_data.try_into();
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn test_account_view() {
        let expected = AccountView {
            address: BytesView("acctaddr".to_string()),
            balances: vec![AmountView {
                amount: 10,
                currency: "USD".to_string(),
            }],
            sequence_number: 1,
            authentication_key: BytesView("authkey".to_string()),
            sent_events_key: BytesView("sentmsgskey".to_string()),
            received_events_key: BytesView("receivedeventskey".to_string()),
            delegated_key_rotation_capability: true,
            delegated_withdrawal_capability: false,
            is_frozen: false,
            role: AccountRoleView::DesignatedDealer {
                human_name: "humanname".to_string(),
                base_url: "baseurl".to_string(),
                expiration_time: 0,
                compliance_key: BytesView("compliancekey".to_string()),
                preburn_balances: vec![AmountView {
                    amount: 2,
                    currency: "USD".to_string(),
                }],
                received_mint_events_key: BytesView("receivedminteventskey".to_string()),
                compliance_key_rotation_events_key: BytesView(
                    "compliancekeyrotationkey".to_string(),
                ),
                base_url_rotation_events_key: BytesView("baseurlrotationeventskey".to_string()),
            },
        };
        let source = jsonrpc::Account {
            address: expected.address.0.clone(),
            balances: vec![jsonrpc::Amount {
                amount: 10,
                currency: "USD".to_string(),
            }],
            sequence_number: 1,
            authentication_key: expected.authentication_key.0.clone(),
            sent_events_key: expected.sent_events_key.0.clone(),
            received_events_key: expected.received_events_key.0.clone(),
            delegated_key_rotation_capability: expected.delegated_key_rotation_capability,
            delegated_withdrawal_capability: expected.delegated_withdrawal_capability,
            is_frozen: expected.is_frozen,
            role: Some(jsonrpc::AccountRole {
                r#type: "designated_dealer".to_string(),
                parent_vasp_address: String::new(),
                human_name: "humanname".to_string(),
                base_url: "baseurl".to_string(),
                expiration_time: 0,
                compliance_key: "compliancekey".to_string(),
                compliance_key_rotation_events_key: "compliancekeyrotationkey".to_string(),
                base_url_rotation_events_key: "baseurlrotationeventskey".to_string(),
                num_children: 0,
                received_mint_events_key: "receivedminteventskey".to_string(),
                preburn_balances: vec![jsonrpc::Amount {
                    amount: 2,
                    currency: "USD".to_string(),
                }],
            }),
        };
        assert_eq!(expected, source.clone().try_into().unwrap());
        let mut role = source.role.unwrap();
        role.r#type = "invalid".to_string();
        let result: Result<AccountRoleView> = role.try_into();
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn test_metadata_view() {
        let expected = MetadataView {
            version: 1,
            accumulator_root_hash: BytesView("acctroothash".to_string()),
            timestamp: 123,
            chain_id: 6,
            script_hash_allow_list: Some(vec![
                BytesView("hash1".to_string()),
                BytesView("hash2".to_string()),
            ]),
            module_publishing_allowed: Some(true),
            diem_version: Some(1),
            dual_attestation_limit: Some(2),
        };
        let source = jsonrpc::Metadata {
            version: 1,
            timestamp: 123,
            chain_id: 6,
            script_hash_allow_list: vec!["hash1".to_string(), "hash2".to_string()],
            module_publishing_allowed: true,
            diem_version: 1,
            accumulator_root_hash: "acctroothash".to_string(),
            dual_attestation_limit: 2,
        };
        assert_eq!(expected, source.try_into().unwrap());
    }

    #[test]
    fn test_transaction_view() {
        let expected = TransactionView {
            version: 1,
            transaction: TransactionDataView::UserTransaction {
                sender: BytesView("sender".to_string()),
                signature_scheme: "signaturescheme".to_string(),
                signature: BytesView("signature".to_string()),
                public_key: BytesView("publickey".to_string()),
                sequence_number: 1,
                chain_id: 2,
                max_gas_amount: 3,
                gas_unit_price: 4,
                gas_currency: "USD".to_string(),
                expiration_timestamp_secs: 1,
                script_hash: BytesView("scripthash".to_string()),
                script_bytes: BytesView("scriptbytes".to_string()),
                script: ScriptView {
                    r#type: "test".to_string(),
                    code: Some(BytesView("code".to_string())),
                    arguments: Some(vec!["arg1".to_string(), "arg2".to_string()]),
                    type_arguments: Some(vec!["targ1".to_string(), "targ2".to_string()]),
                    receiver: Some(BytesView("receiver".to_string())),
                    amount: Some(1),
                    currency: Some("USD".to_string()),
                    metadata: Some(BytesView(String::new())),
                    metadata_signature: Some(BytesView("signature".to_string())),
                },
            },
            hash: BytesView("hash".to_string()),
            bytes: BytesView("bytes".to_string()),
            events: vec![],
            vm_status: VMStatusView::MoveAbort {
                location: "location".to_string(),
                abort_code: 1,
                explanation: Some(MoveAbortExplanationView {
                    category: "category".to_string(),
                    category_description: "category_description".to_string(),
                    reason: "reason".to_string(),
                    reason_description: "reason_description".to_string(),
                }),
            },
            gas_used: 0,
        };
        let source = jsonrpc::Transaction {
            version: 1,
            hash: "hash".to_string(),
            bytes: "bytes".to_string(),
            events: vec![],
            gas_used: 0,
            vm_status: Some(jsonrpc::VmStatus {
                r#type: "move_abort".to_string(),
                location: "location".to_string(),
                abort_code: 1,
                function_index: 0,
                code_offset: 0,
                explanation: Some(jsonrpc::MoveAbortExplaination {
                    category: "category".to_string(),
                    category_description: "category_description".to_string(),
                    reason: "reason".to_string(),
                    reason_description: "reason_description".to_string(),
                }),
            }),
            transaction: Some(jsonrpc::TransactionData {
                r#type: "user".to_string(),
                timestamp_usecs: 0,
                sender: "sender".to_string(),
                signature_scheme: "signaturescheme".to_string(),
                signature: "signature".to_string(),
                public_key: "publickey".to_string(),
                sequence_number: 1,
                chain_id: 2,
                max_gas_amount: 3,
                gas_unit_price: 4,
                gas_currency: "USD".to_string(),
                expiration_timestamp_secs: 1,
                script_hash: "scripthash".to_string(),
                script_bytes: "scriptbytes".to_string(),
                script: Some(jsonrpc::Script {
                    r#type: "test".to_string(),
                    code: "code".to_string(),
                    arguments: vec!["arg1".to_string(), "arg2".to_string()],
                    type_arguments: vec!["targ1".to_string(), "targ2".to_string()],
                    receiver: "receiver".to_string(),
                    amount: 1,
                    currency: "USD".to_string(),
                    metadata: String::new(),
                    metadata_signature: "signature".to_string(),
                }),
            }),
        };
        assert_eq!(expected, source.try_into().unwrap());
    }
}
