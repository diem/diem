// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![allow(clippy::unit_arg)]

use anyhow::{Error, Result};
use libra_types::mempool_status::{MempoolStatus, MempoolStatusCode};
use std::convert::TryFrom;

////***********************************
//// Decoding/Encoding to Protobuffers
////***********************************
impl TryFrom<crate::proto::types::MempoolStatus> for MempoolStatus {
    type Error = Error;

    fn try_from(proto: crate::proto::types::MempoolStatus) -> Result<Self> {
        Ok(MempoolStatus::new(
            MempoolStatusCode::try_from(proto.code).unwrap_or(MempoolStatusCode::UnknownStatus),
        )
        .with_message(proto.message))
    }
}

impl From<MempoolStatus> for crate::proto::types::MempoolStatus {
    fn from(status: MempoolStatus) -> Self {
        let mut proto_status = Self::default();
        proto_status.code = status.code.into();
        proto_status.message = status.message;
        proto_status
    }
}
