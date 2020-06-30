// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use once_cell::sync::Lazy;
use prometheus::{register_int_counter, IntCounter};

/// Count of the error logs.
pub static LOG_ERROR_COUNT: Lazy<IntCounter> = Lazy::new(|| {
    register_int_counter!("libra_log_error_count", "Count of the error logs.").unwrap()
});

/// Count of the warn logs.
pub static LOG_WARN_COUNT: Lazy<IntCounter> =
    Lazy::new(|| register_int_counter!("libra_log_warn_count", "Count of the warn logs.").unwrap());

/// Count of the info logs.
pub static LOG_INFO_COUNT: Lazy<IntCounter> =
    Lazy::new(|| register_int_counter!("libra_log_info_count", "Count of the info logs.").unwrap());

/// Count of the debug logs.
pub static LOG_DEBUG_COUNT: Lazy<IntCounter> = Lazy::new(|| {
    register_int_counter!("libra_log_debug_count", "Count of the debug logs.").unwrap()
});
