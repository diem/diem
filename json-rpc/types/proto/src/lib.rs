// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![allow(bare_trait_objects)]

#[allow(clippy::large_enum_variant)]
pub mod types {
    include!(concat!(env!("OUT_DIR"), "/jsonrpc.rs"));

    pub use crate::constants::*;
}

pub mod constants;

fn is_default<T: Default + PartialEq>(t: &T) -> bool {
    t == &T::default()
}

#[cfg(test)]
mod tests {
    use crate::{is_default, types as jsonrpc, types::Amount};
    use diem_crypto::HashValue;
    use diem_json_rpc_types::views::{
        AmountView, BytesView, PreburnWithMetadataView, ScriptView, TransactionDataView,
    };
    use diem_types::account_address::AccountAddress;
    use serde_json::json;

    #[test]
    fn test_is_default() {
        let amount = Amount {
            amount: 0,
            currency: "CODE".to_string(),
        };
        assert!(is_default(&amount.amount));
        assert!(!is_default(&amount.currency));

        assert_eq!(
            serde_json::json!({
                "currency": "CODE"
            }),
            serde_json::to_value(&amount).unwrap()
        );

        let des_amount: Amount = serde_json::from_value(serde_json::json!({
            "currency": "CODE"
        }))
        .unwrap();
        assert_eq!(amount, des_amount);
    }

    #[test]
    fn test_serialize_preburn_with_metadata_view() {
        let view = PreburnWithMetadataView {
            preburn: AmountView {
                amount: 1,
                currency: "XUS".to_string(),
            },
            metadata: None,
        };
        let value = serde_json::to_value(&view).unwrap();
        assert_eq!(
            value,
            json!({
                "preburn": {
                    "amount": 1,
                    "currency": "XUS"
                }
            })
        );

        let preburn: jsonrpc::PreburnWithMetadata = serde_json::from_value(value).unwrap();
        assert_eq!(preburn.metadata, "");
    }

    #[test]
    fn test_serialize_transaction_data_view() {
        let bytes = BytesView::from(vec![]);
        let hash = HashValue::random();
        let view = TransactionDataView::UserTransaction {
            sender: AccountAddress::from_hex_literal("0xdd").unwrap(),
            signature_scheme: "Ed25519".to_string(),
            signature: bytes.clone(),
            public_key: bytes.clone(),
            secondary_signers: None,
            secondary_signature_schemes: None,
            secondary_signatures: None,
            secondary_public_keys: None,
            sequence_number: 10,
            chain_id: 4,
            max_gas_amount: 100,
            gas_unit_price: 10,
            gas_currency: "XUS".to_string(),
            expiration_timestamp_secs: 60,
            script_hash: hash,
            script_bytes: bytes,
            script: ScriptView::unknown(),
        };

        let value = serde_json::to_value(&view).unwrap();
        assert_eq!(
            value,
            json!({
                "type": "user",
                "sender": "000000000000000000000000000000dd",
                "signature_scheme": "Ed25519",
                "signature": "",
                "public_key": "",
                "sequence_number": 10,
                "chain_id": 4,
                "max_gas_amount": 100,
                "gas_unit_price": 10,
                "gas_currency": "XUS",
                "expiration_timestamp_secs": 60,
                "script_hash": hash,
                "script_bytes": "",
                "script": {
                    "type": "unknown"
                },
            })
        );

        let txn_data: jsonrpc::TransactionData = serde_json::from_value(value).unwrap();
        assert_eq!(txn_data.secondary_signers, Vec::<String>::new());
        assert_eq!(txn_data.secondary_signature_schemes, Vec::<String>::new());
        assert_eq!(txn_data.secondary_signatures, Vec::<String>::new());
        assert_eq!(txn_data.secondary_public_keys, Vec::<String>::new());
    }

    #[test]
    fn test_serialize_multi_agent_transaction_data_view() {
        let bytes = BytesView::from(vec![42]);
        let hash = HashValue::random();
        let view = TransactionDataView::UserTransaction {
            sender: AccountAddress::from_hex_literal("0xdd").unwrap(),
            signature_scheme: "Ed25519".to_string(),
            signature: bytes.clone(),
            public_key: bytes.clone(),
            secondary_signers: Some(vec![
                AccountAddress::from_hex_literal("0xaa").unwrap(),
                AccountAddress::from_hex_literal("0xbb").unwrap(),
            ]),
            secondary_signature_schemes: Some(vec![
                "Ed25519".to_string(),
                "MultiEd25519".to_string(),
            ]),
            secondary_signatures: Some(vec![BytesView::from(vec![42]), BytesView::from(vec![43])]),
            secondary_public_keys: Some(vec![BytesView::from(vec![44]), BytesView::from(vec![45])]),
            sequence_number: 10,
            chain_id: 4,
            max_gas_amount: 100,
            gas_unit_price: 10,
            gas_currency: "XUS".to_string(),
            expiration_timestamp_secs: 60,
            script_hash: hash,
            script_bytes: bytes.clone(),
            script: ScriptView::unknown(),
        };

        let value = serde_json::to_value(&view).unwrap();
        assert_eq!(
            value,
            json!({
                "type": "user",
                "sender": "000000000000000000000000000000dd",
                "signature_scheme": "Ed25519",
                "signature": "2a",
                "public_key": "2a",
                "secondary_signers": [
                    "000000000000000000000000000000aa",
                    "000000000000000000000000000000bb"
                ],
                "secondary_signature_schemes": ["Ed25519", "MultiEd25519"],
                "secondary_signatures": [ "2a", "2b" ],
                "secondary_public_keys": [ "2c", "2d" ],
                "sequence_number": 10,
                "chain_id": 4,
                "max_gas_amount": 100,
                "gas_unit_price": 10,
                "gas_currency": "XUS",
                "expiration_timestamp_secs": 60,
                "script_hash": hash,
                "script_bytes": bytes,
                "script": {
                    "type": "unknown"
                },
            })
        );

        let txn_data: jsonrpc::TransactionData = serde_json::from_value(value).unwrap();
        assert_eq!(
            txn_data.secondary_signers,
            vec![
                "000000000000000000000000000000aa",
                "000000000000000000000000000000bb"
            ]
        );
        assert_eq!(
            txn_data.secondary_signature_schemes,
            vec!["Ed25519", "MultiEd25519"]
        );
        assert_eq!(txn_data.secondary_signatures, vec!["2a", "2b"]);
        assert_eq!(txn_data.secondary_public_keys, vec!["2c", "2d"]);
    }
}
