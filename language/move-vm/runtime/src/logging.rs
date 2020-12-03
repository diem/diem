// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use diem_logger::Schema;

// Trait used by the VM to log interesting data.
// Clients are responsible for the implementation of alert.
pub trait LogContext: Schema + Clone {
    // Alert is called on critical errors
    fn alert(&self);
}

// Helper `Logger` implementation that does nothing
#[derive(Schema, Clone)]
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
}
