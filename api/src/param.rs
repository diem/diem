// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use diem_api_types::{Address, Error, TransactionId};
use serde::{Deserialize, Deserializer};

use std::{convert::Infallible, str::FromStr};

pub type AddressParam = Param<Address>;
pub type TransactionIdParam = Param<TransactionId>;
pub type TransactionVersionParam = Param<u64>;
pub type LedgerVersionParam = Param<u64>;

#[derive(Clone, Debug)]
pub struct Param<T: FromStr> {
    data: String,
    _value: Option<T>,
}

impl<T: FromStr> FromStr for Param<T> {
    type Err = Infallible;

    fn from_str(data: &str) -> Result<Self, Infallible> {
        Ok(Self {
            data: data.to_owned(),
            _value: None,
        })
    }
}

impl<T: FromStr> Param<T> {
    pub fn parse(self, name: &str) -> Result<T, Error> {
        self.data
            .parse()
            .map_err(|_| Error::invalid_param(name, &self.data))
    }
}

impl<'de, T: FromStr> Deserialize<'de> for Param<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let data = <String>::deserialize(deserializer)?;
        Ok(Self { data, _value: None })
    }
}
