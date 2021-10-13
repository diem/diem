// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use diem_types::account_address::AccountAddress;
use serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer};
use std::{fmt, str::FromStr};

#[derive(Clone, Debug, PartialEq, Copy)]
pub struct Address(AccountAddress);

impl Address {
    pub fn inner(&self) -> &AccountAddress {
        &self.0
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0.to_hex_literal())
    }
}

impl FromStr for Address {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> anyhow::Result<Self, anyhow::Error> {
        let mut ret = AccountAddress::from_hex_literal(s);
        if ret.is_err() {
            ret = AccountAddress::from_hex(s)
        }
        Ok(Self(ret.map_err(|_| {
            anyhow::format_err!("invalid account address: {}", s)
        })?))
    }
}

impl From<AccountAddress> for Address {
    fn from(address: AccountAddress) -> Self {
        Self(address)
    }
}

impl From<Address> for AccountAddress {
    fn from(address: Address) -> Self {
        address.0
    }
}

impl From<&Address> for AccountAddress {
    fn from(address: &Address) -> Self {
        address.0
    }
}

impl Serialize for Address {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if serializer.is_human_readable() {
            self.to_string().serialize(serializer)
        } else {
            self.0.serialize(serializer)
        }
    }
}

impl<'de> Deserialize<'de> for Address {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let address = <String>::deserialize(deserializer)?;
            address.parse().map_err(D::Error::custom)
        } else {
            Ok(Self(<AccountAddress>::deserialize(deserializer)?))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::address::Address;

    use diem_types::account_address::AccountAddress;

    use serde_json::{json, Value};

    #[test]
    fn test_from_and_to_string() {
        let valid_addresses = vec!["0x1", "0x001", "00000000000000000000000000000001"];
        for address in valid_addresses {
            assert_eq!(address.parse::<Address>().unwrap().to_string(), "0x1");
        }

        let invalid_addresses = vec!["invalid", "00x1", "x1", "01", "1"];
        for address in invalid_addresses {
            assert_eq!(
                format!("invalid account address: {}", address),
                address.parse::<Address>().unwrap_err().to_string()
            );
        }
    }

    #[test]
    fn test_from_and_to_json() {
        let address: Address = serde_json::from_value(json!("0x1")).unwrap();
        assert_eq!(address, "0x1".parse().unwrap());

        let val: Value = serde_json::to_value(address).unwrap();
        assert_eq!(val, json!("0x1"));
    }

    #[test]
    fn test_from_and_to_bcs_should_be_same_with_account_address() {
        let address: Address = "0x1".parse().unwrap();
        let account_address: AccountAddress = address.into();

        let deserialized: Address =
            bcs::from_bytes(&bcs::to_bytes(&account_address).unwrap()).unwrap();
        assert_eq!(deserialized, address);

        let deserialized: Address = bcs::from_bytes(&bcs::to_bytes(&address).unwrap()).unwrap();
        assert_eq!(deserialized, address);
    }

    #[test]
    fn test_from_and_to_account_address() {
        let address: Address = serde_json::from_value(json!("0x1")).unwrap();

        let account_address: AccountAddress = address.into();
        assert_eq!(
            account_address.to_string(),
            "00000000000000000000000000000001"
        );

        let new_address: Address = account_address.into();
        assert_eq!(new_address, address);
    }

    #[test]
    fn test_from_and_to_account_address_reference() {
        let address: Address = serde_json::from_value(json!("0x1")).unwrap();

        let account_address: AccountAddress = (&address).into();
        assert_eq!(
            account_address.to_string(),
            "00000000000000000000000000000001"
        );

        let new_address: Address = account_address.into();
        assert_eq!(new_address, address);
    }
}
