// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub mod errors;
pub mod response;
pub mod views;

pub mod constants;
pub mod proto;

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
