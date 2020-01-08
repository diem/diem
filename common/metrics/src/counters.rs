// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use lazy_static::lazy_static;
use prometheus::IntCounter;

lazy_static! {
    // Admission Control counters
    pub static ref COUNTER_ADMISSION_CONTROL_CANNOT_SEND_REPLY: IntCounter = register_int_counter!(
        "COUNTER_ADMISSION_CONTROL_CANNOT_SEND_REPLY",
        "Number of errors related to send reply in Admission Control"
    ).unwrap();

    // Client counters
    pub static ref COUNTER_CLIENT_ERRORS: IntCounter = register_int_counter!(
        "COUNTER_CLIENT_ERRORS",
        "Number of errors encountered by Client"
    ).unwrap();
}
