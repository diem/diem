// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

pub mod proto;

use anyhow::{format_err, Error, Result};
use libra_logger::prelude::*;
use libra_types::{mempool_status::MempoolStatus as LibraMempoolStatus, vm_error::VMStatus};
use std::convert::TryFrom;

/// AC response status of submit_transaction to clients.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum AdmissionControlStatus {
    /// Validator accepted the transaction.
    Accepted,
    /// The sender is blacklisted.
    Blacklisted(String),
    /// The transaction is rejected, e.g. due to incorrect signature.
    Rejected(String),
}

impl TryFrom<crate::proto::admission_control::AdmissionControlStatus> for AdmissionControlStatus {
    type Error = Error;

    fn try_from(proto: crate::proto::admission_control::AdmissionControlStatus) -> Result<Self> {
        use crate::proto::admission_control::AdmissionControlStatusCode as ProtoStatusCode;
        let ret = match proto.code() {
            ProtoStatusCode::Accepted => AdmissionControlStatus::Accepted,
            ProtoStatusCode::Blacklisted => {
                let msg = proto.message;
                AdmissionControlStatus::Blacklisted(msg)
            }
            ProtoStatusCode::Rejected => {
                let msg = proto.message;
                AdmissionControlStatus::Rejected(msg)
            }
        };
        Ok(ret)
    }
}

impl From<AdmissionControlStatus> for crate::proto::admission_control::AdmissionControlStatus {
    fn from(status: AdmissionControlStatus) -> Self {
        use crate::proto::admission_control::AdmissionControlStatusCode as ProtoStatusCode;
        let mut admission_control_status = Self::default();
        match status {
            AdmissionControlStatus::Accepted => {
                admission_control_status.set_code(ProtoStatusCode::Accepted)
            }
            AdmissionControlStatus::Blacklisted(msg) => {
                admission_control_status.message = msg;
                admission_control_status.set_code(ProtoStatusCode::Blacklisted)
            }
            AdmissionControlStatus::Rejected(msg) => {
                admission_control_status.message = msg;
                admission_control_status.set_code(ProtoStatusCode::Rejected)
            }
        }
        admission_control_status
    }
}

/// Rust structure for SubmitTransactionResponse protobuf definition.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SubmitTransactionResponse {
    /// AC status returned to client if any - it can be one of: accepted, blacklisted, or rejected.
    pub ac_status: Option<AdmissionControlStatus>,
    /// Mempool error status if any.
    pub mempool_error: Option<LibraMempoolStatus>,
    /// VM error status if any.
    pub vm_error: Option<VMStatus>,
    /// The id of validator associated with this AC.
    pub validator_id: Vec<u8>,
}

impl TryFrom<crate::proto::admission_control::SubmitTransactionResponse>
    for SubmitTransactionResponse
{
    type Error = Error;

    fn try_from(proto: crate::proto::admission_control::SubmitTransactionResponse) -> Result<Self> {
        use crate::proto::admission_control::submit_transaction_response::Status::*;

        let validator_id = proto.validator_id;
        let status = proto.status.ok_or_else(|| format_err!("Missing status"))?;
        let (ac_status, mempool_error, vm_error) = match status {
            VmStatus(status) => (None, None, Some(VMStatus::try_from(status)?)),
            AcStatus(status) => (Some(AdmissionControlStatus::try_from(status)?), None, None),
            MempoolStatus(status) => (None, Some(LibraMempoolStatus::try_from(status)?), None),
        };
        Ok(SubmitTransactionResponse {
            ac_status,
            mempool_error,
            vm_error,
            validator_id,
        })
    }
}

impl From<SubmitTransactionResponse>
    for crate::proto::admission_control::SubmitTransactionResponse
{
    fn from(status: SubmitTransactionResponse) -> Self {
        use crate::proto::admission_control::submit_transaction_response::Status::*;

        let mut proto = Self::default();
        if let Some(ac_st) = status.ac_status {
            proto.status = Some(AcStatus(ac_st.into()));
        } else if let Some(mem_err) = status.mempool_error {
            proto.status = Some(MempoolStatus(mem_err.into()));
        } else if let Some(vm_st) = status.vm_error {
            proto.status = Some(VmStatus(vm_st.into()));
        } else {
            error!("No status is available in SubmitTransactionResponse!");
        }
        proto.validator_id = status.validator_id;
        proto
    }
}
