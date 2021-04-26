// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub mod errors;
pub mod request;
pub mod response;
pub mod views;

pub mod constants;
pub mod proto;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
enum JsonRpcVersion {
    #[serde(rename = "2.0")]
    V2,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(untagged)]
pub enum Id {
    /// Numeric id
    Number(u64),
    /// String id
    String(Box<str>),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Method {
    Submit,
    GetMetadata,
    GetAccount,
    GetTransactions,
    GetAccountTransaction,
    GetAccountTransactions,
    GetEvents,
    GetCurrencies,
    GetNetworkStatus,

    //
    // Experimental APIs
    //
    GetStateProof,
    GetAccountStateWithProof,
    GetTransactionsWithProofs,
    GetEventsWithProofs,
}

impl Method {
    pub fn as_str(&self) -> &str {
        match self {
            Method::Submit => "submit",
            Method::GetMetadata => "get_metadata",
            Method::GetAccount => "get_account",
            Method::GetTransactions => "get_transactions",
            Method::GetAccountTransaction => "get_account_transaction",
            Method::GetAccountTransactions => "get_account_transactions",
            Method::GetEvents => "get_events",
            Method::GetCurrencies => "get_currencies",
            Method::GetNetworkStatus => "get_network_status",
            Method::GetStateProof => "get_state_proof",
            Method::GetAccountStateWithProof => "get_account_state_with_proof",
            Method::GetTransactionsWithProofs => "get_transactions_with_proofs",
            Method::GetEventsWithProofs => "get_events_with_proofs",
        }
    }
}

fn is_default<T: Default + PartialEq>(t: &T) -> bool {
    t == &T::default()
}

#[cfg(test)]
mod tests {
    use crate::{is_default, proto::types::Amount};

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
}
