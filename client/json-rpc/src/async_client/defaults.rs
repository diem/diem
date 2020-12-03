// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::time::Duration;

pub const MAX_RETRIES: u32 = 5;
pub const WAIT_DELAY: Duration = Duration::from_millis(50);
pub const TIMEOUT: Duration = Duration::from_secs(5);
pub const HTTP_REQUEST_TIMEOUT: Duration = TIMEOUT;
