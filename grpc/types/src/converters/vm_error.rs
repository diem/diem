// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![allow(clippy::unit_arg)]

use anyhow::{Error, Result};
use libra_types::vm_error::{StatusCode, VMStatus};
use std::convert::TryFrom;

impl TryFrom<crate::proto::types::VmStatus> for VMStatus {
    type Error = Error;

    fn try_from(proto: crate::proto::types::VmStatus) -> Result<Self> {
        let mut status = VMStatus::new(
            StatusCode::try_from(proto.major_status).unwrap_or(StatusCode::UNKNOWN_STATUS),
        );

        if proto.has_sub_status {
            status.set_sub_status(proto.sub_status);
        }

        if proto.has_message {
            status.set_message(proto.message);
        }

        Ok(status)
    }
}

impl From<VMStatus> for crate::proto::types::VmStatus {
    fn from(status: VMStatus) -> Self {
        let mut proto_status = Self::default();

        proto_status.has_sub_status = false;
        proto_status.has_message = false;

        // Set major status
        proto_status.major_status = status.major_status.into();

        // Set minor status if there is one
        if let Some(sub_status) = status.sub_status {
            proto_status.has_sub_status = true;
            proto_status.sub_status = sub_status;
        }

        // Set info string
        if let Some(string) = status.message {
            proto_status.has_message = true;
            proto_status.message = string;
        }

        proto_status
    }
}
