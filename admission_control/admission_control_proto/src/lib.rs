// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub mod proto;

use failure::prelude::*;
use logger::prelude::*;
use mempool_shared_proto::MempoolAddTransactionStatus;
use proto_conv::{FromProto, IntoProto};
use types::vm_error::VMStatus;

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

impl IntoProto for AdmissionControlStatus {
    type ProtoType = crate::proto::admission_control::AdmissionControlStatus;

    fn into_proto(self) -> Self::ProtoType {
        use crate::proto::admission_control::AdmissionControlStatusCode as ProtoStatusCode;
        let mut admission_control_status = Self::ProtoType::new();
        match self {
            AdmissionControlStatus::Accepted => {
                admission_control_status.set_code(ProtoStatusCode::Accepted)
            }
            AdmissionControlStatus::Blacklisted(msg) => {
                admission_control_status.set_message(msg);
                admission_control_status.set_code(ProtoStatusCode::Blacklisted)
            }
            AdmissionControlStatus::Rejected(msg) => {
                admission_control_status.set_message(msg);
                admission_control_status.set_code(ProtoStatusCode::Rejected)
            }
        }
        admission_control_status
    }
}

impl FromProto for AdmissionControlStatus {
    type ProtoType = crate::proto::admission_control::AdmissionControlStatus;

    fn from_proto(mut proto_admission_control_status: Self::ProtoType) -> Result<Self> {
        use crate::proto::admission_control::AdmissionControlStatusCode as ProtoStatusCode;
        let ret = match proto_admission_control_status.get_code() {
            ProtoStatusCode::Accepted => AdmissionControlStatus::Accepted,
            ProtoStatusCode::Blacklisted => {
                let msg = proto_admission_control_status.take_message();
                AdmissionControlStatus::Blacklisted(msg)
            }
            ProtoStatusCode::Rejected => {
                let msg = proto_admission_control_status.take_message();
                AdmissionControlStatus::Rejected(msg)
            }
        };
        Ok(ret)
    }
}

/// Rust structure for SubmitTransactionResponse protobuf definition.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SubmitTransactionResponse {
    /// AC status returned to client if any - it can be one of: accepted, blacklisted, or rejected.
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
            Some(AdmissionControlStatus::from_proto(object.take_ac_status())?)
        } else {
            None
        };
        let mempool_error = if object.has_mempool_status() {
            Some(MempoolAddTransactionStatus::from_proto(
                object.take_mempool_status(),
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
