// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use diem_logger::{prelude::error, Schema};
use move_binary_format::errors::{PartialVMError, VMError};
use move_core_types::vm_status::{StatusCode, StatusType};

// Trait used by the VM to log interesting data.
// Clients are responsible for the implementation of alert.
pub trait LogContext: Schema {
    // Alert is called on critical errors
    fn alert(&self);

    fn as_super(&self) -> &dyn Schema;
}

// Helper `Logger` implementation that does nothing
#[derive(Schema)]
pub struct NoContextLog {
    name: String,
}

impl NoContextLog {
    pub fn new() -> Self {
        Self {
            name: "test".to_string(),
        }
    }
}

impl LogContext for NoContextLog {
    fn alert(&self) {}

    fn as_super(&self) -> &dyn Schema {
        self
    }
}

//
// Utility functions
//

pub fn expect_no_verification_errors(err: VMError, log_context: &impl LogContext) -> VMError {
    match err.status_type() {
        status_type @ StatusType::Deserialization | status_type @ StatusType::Verification => {
            let message = format!(
                "Unexpected verifier/deserialization error! This likely means there is code \
                stored on chain that is unverifiable!\nError: {:?}",
                &err
            );
            let (_old_status, _old_sub_status, _old_message, location, indices, offsets) =
                err.all_data();
            let major_status = match status_type {
                StatusType::Deserialization => StatusCode::UNEXPECTED_DESERIALIZATION_ERROR,
                StatusType::Verification => StatusCode::UNEXPECTED_VERIFIER_ERROR,
                _ => unreachable!(),
            };

            log_context.alert();
            error!(*log_context, "[VM] {}", message);
            PartialVMError::new(major_status)
                .with_message(message)
                .at_indices(indices)
                .at_code_offsets(offsets)
                .finish(location)
        }
        _ => err,
    }
}
