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
    use diem_json_rpc_types::views::{AmountView, PreburnWithMetadataView};
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
}
