// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![allow(clippy::unit_arg)]

use anyhow::{Error, Result};
#[cfg(any(test, feature = "fuzzing"))]
use proptest::prelude::*;
#[cfg(any(test, feature = "fuzzing"))]
use proptest_derive::Arbitrary;
use std::convert::TryFrom;

/// A `MempoolStatus` is represented as a required status code that is semantic coupled with an optional sub status and message.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
#[cfg_attr(any(test, feature = "fuzzing"), proptest(no_params))]
pub struct MempoolStatus {
    /// insertion status code
    pub code: MempoolStatusCode,
    /// optional message
    pub message: String,
}

impl MempoolStatus {
    pub fn new(code: MempoolStatusCode) -> Self {
        Self {
            code,
            message: "".to_string(),
        }
    }

    /// Adds a message to the Mempool status.
    pub fn with_message(mut self, message: String) -> Self {
        self.message = message;
        self
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
#[repr(u64)]
pub enum MempoolStatusCode {
    // Transaction was accepted by Mempool
    Accepted = 0,
    // Sequence number is old, etc.
    InvalidSeqNumber = 1,
    // Mempool is full (reached max global capacity)
    MempoolIsFull = 2,
    // Account reached max capacity per account
    TooManyTransactions = 3,
    // Invalid update. Only gas price increase is allowed
    InvalidUpdate = 4,
    // transaction didn't pass vm_validation
    VmError = 5,
    UnknownStatus = 6,
}

impl TryFrom<u64> for MempoolStatusCode {
    type Error = &'static str;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(MempoolStatusCode::Accepted),
            1 => Ok(MempoolStatusCode::InvalidSeqNumber),
            2 => Ok(MempoolStatusCode::MempoolIsFull),
            3 => Ok(MempoolStatusCode::TooManyTransactions),
            4 => Ok(MempoolStatusCode::InvalidUpdate),
            5 => Ok(MempoolStatusCode::VmError),
            6 => Ok(MempoolStatusCode::UnknownStatus),
            _ => Err("invalid StatusCode"),
        }
    }
}

impl From<MempoolStatusCode> for u64 {
    fn from(status: MempoolStatusCode) -> u64 {
        status as u64
    }
}

////***********************************
//// Decoding/Encoding to Protobuffers
////***********************************
impl TryFrom<::proto_types::types::MempoolStatus> for MempoolStatus {
    type Error = Error;

    fn try_from(proto: ::proto_types::types::MempoolStatus) -> Result<Self> {
        Ok(MempoolStatus::new(
            MempoolStatusCode::try_from(proto.code).unwrap_or(MempoolStatusCode::UnknownStatus),
        )
        .with_message(proto.message))
    }
}

impl From<MempoolStatus> for ::proto_types::types::MempoolStatus {
    fn from(status: MempoolStatus) -> Self {
        let mut proto_status = Self::default();
        proto_status.code = status.code.into();
        proto_status.message = status.message;
        proto_status
    }
}
