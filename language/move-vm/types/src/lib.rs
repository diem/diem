// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

macro_rules! debug_write {
    ($($toks: tt)*) => {
        write!($($toks)*).map_err(|_|
            PartialVMError::new(StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR)
                .with_message("failed to write to buffer".to_string())
        )
    };
}

macro_rules! debug_writeln {
    ($($toks: tt)*) => {
        writeln!($($toks)*).map_err(|_|
            PartialVMError::new(StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR)
                .with_message("failed to write to buffer".to_string())
        )
    };
}

pub mod data_store;
pub mod gas_schedule;
pub mod loaded_data;
pub mod natives;
pub mod values;

#[cfg(test)]
mod unit_tests;
