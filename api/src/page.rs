// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::param::{Param, TransactionVersionParam};

use diem_api_types::{Error, TransactionId};

use anyhow::Result;
use serde::Deserialize;

const DEFAULT_PAGE_SIZE: u16 = 25;
const MAX_PAGE_SIZE: u16 = 1000;

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct Page {
    start: Option<TransactionVersionParam>,
    limit: Option<Param<u16>>,
}

impl Page {
    pub fn start(&self, latest_ledger_version: u64) -> Result<u64, Error> {
        let version = self
            .start
            .clone()
            .map(|v| v.parse("start"))
            .unwrap_or_else(|| Ok(latest_ledger_version))?;
        if version > latest_ledger_version {
            return Err(Error::not_found(
                "transaction",
                TransactionId::Version(version),
                latest_ledger_version,
            ));
        }
        Ok(version)
    }

    pub fn limit(&self) -> Result<u16, Error> {
        let limit = self
            .limit
            .clone()
            .map(|v| v.parse("limit"))
            .unwrap_or_else(|| Ok(DEFAULT_PAGE_SIZE))?;
        if limit > MAX_PAGE_SIZE {
            return Err(Error::invalid_param(
                "limit",
                format!("{}, exceed limit {}", limit, MAX_PAGE_SIZE),
            ));
        }
        Ok(limit)
    }
}
