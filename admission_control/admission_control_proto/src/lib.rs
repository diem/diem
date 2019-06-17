// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub mod proto;

use crate::proto::admission_control::AdmissionControlStatus as ProtoAdmissionControlStatus;
use failure::prelude::*;
use logger::prelude::*;
use mempool::MempoolAddTransactionStatus;
use proto_conv::{FromProto, IntoProto};
use types::vm_error::VMStatus;

/// AC response status of submit_transaction to clients.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum AdmissionControlStatus {
    /// Validator accepted the transaction.
    Accepted = 0,
    /// The sender is blacklisted.
    Blacklisted = 1,
    /// The transaction is rejected, e.g. due to incorrect signature.
    Rejected = 2,
}

impl IntoProto for AdmissionControlStatus {
    type ProtoType = crate::proto::admission_control::AdmissionControlStatus;

    fn into_proto(self) -> Self::ProtoType {
        match self {
            AdmissionControlStatus::Accepted => ProtoAdmissionControlStatus::Accepted,
            AdmissionControlStatus::Blacklisted => ProtoAdmissionControlStatus::Blacklisted,
            AdmissionControlStatus::Rejected => ProtoAdmissionControlStatus::Rejected,
        }
    }
}

impl FromProto for AdmissionControlStatus {
    type ProtoType = crate::proto::admission_control::AdmissionControlStatus;

    fn from_proto(object: Self::ProtoType) -> Result<Self> {
        let ret = match object {
            ProtoAdmissionControlStatus::Accepted => AdmissionControlStatus::Accepted,
            ProtoAdmissionControlStatus::Blacklisted => AdmissionControlStatus::Blacklisted,
            ProtoAdmissionControlStatus::Rejected => AdmissionControlStatus::Rejected,
        };
        Ok(ret)
    }
}

/// Rust structure for SubmitTransactionResponse protobuf definition.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SubmitTransactionResponse {
    /// AC status returned to client if any, it includes can be either error or accepted status.
    pub ac_status: Option<AdmissionControlStatus>,
    /// Mempool error status if any.
    pub mempool_error: Option<MempoolAddTransactionStatus>,
    /// VM error status if any.
    pub vm_error: Option<VMStatus>,
    /// The id of validator associated with this AC.
    pub validator_id: Vec<u8>,
}

impl IntoProto for SubmitTransactionResponse {
    type ProtoType = crate::proto::admission_control::SubmitTransactionResponse;

    fn into_proto(self) -> Self::ProtoType {
        let mut proto = Self::ProtoType::new();
        if let Some(ac_st) = self.ac_status {
            proto.set_ac_status(ac_st.into_proto());
        } else if let Some(mem_err) = self.mempool_error {
            proto.set_mempool_status(mem_err.into_proto());
        } else if let Some(vm_st) = self.vm_error {
            proto.set_vm_status(vm_st.into_proto());
        } else {
            error!("No status is available in SubmitTransactionResponse!");
        }
        proto.set_validator_id(self.validator_id);
        proto
    }
}

impl FromProto for SubmitTransactionResponse {
    type ProtoType = crate::proto::admission_control::SubmitTransactionResponse;

    fn from_proto(mut object: Self::ProtoType) -> Result<Self> {
        let ac_status = if object.has_ac_status() {
            Some(AdmissionControlStatus::from_proto(object.get_ac_status())?)
        } else {
            None
        };
        let mempool_error = if object.has_mempool_status() {
            Some(MempoolAddTransactionStatus::from_proto(
                object.get_mempool_status(),
            )?)
        } else {
            None
        };
        let vm_error = if object.has_vm_status() {
            Some(VMStatus::from_proto(object.take_vm_status())?)
        } else {
            None
        };

        Ok(SubmitTransactionResponse {
            ac_status,
            mempool_error,
            vm_error,
            validator_id: object.take_validator_id(),
        })
    }
}
