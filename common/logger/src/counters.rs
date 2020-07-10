// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use once_cell::sync::Lazy;
use prometheus::{register_int_counter, IntCounter};

/// Count of the struct logs.
pub static STRUCT_LOG_COUNT: Lazy<IntCounter> = Lazy::new(|| {
    register_int_counter!("libra_struct_log_count", "Count of the struct logs.").unwrap()
});
