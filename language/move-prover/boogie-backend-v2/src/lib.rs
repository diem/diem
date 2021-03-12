// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

pub const DEFAULT_PRELUDE: &[u8] = include_bytes!("prelude.bpl");

mod boogie_helpers;
pub mod boogie_wrapper;
pub mod bytecode_translator;
pub mod options;
mod prover_task_runner;
mod spec_translator;
