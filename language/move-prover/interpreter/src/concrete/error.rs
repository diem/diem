// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use move_binary_format::errors::PartialVMError;
use move_core_types::vm_status::StatusCode;

#[repr(u64)]
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub enum ErrorReason {
    _UNKNOWN = 0,

    //
    // Typing
    //

    // param and local type should be the same for entrypoint functions
    TYPE_MISMATCH_ENTRYPOINT_PARAM_AND_LOCAL,
}

pub struct InvariantViolation {
    reason: ErrorReason,
    messages: Vec<String>,
}

impl InvariantViolation {
    pub fn new(reason: ErrorReason) -> Self {
        Self {
            reason,
            messages: vec![],
        }
    }

    pub fn add_message(self, msg: String) -> Self {
        let Self {
            reason,
            mut messages,
        } = self;
        messages.push(msg);
        Self { reason, messages }
    }

    pub fn into_partial_vm_error(self) -> PartialVMError {
        PartialVMError::new(StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR)
            .with_sub_status(self.reason as u64)
            .with_message(self.messages.join("\n"))
    }
}
