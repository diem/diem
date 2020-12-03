// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use std::time::{Duration, SystemTime};

/// Gives the duration since the Unix epoch, notice the expect.
pub fn duration_since_epoch() -> Duration {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("System time is before the UNIX_EPOCH")
}
