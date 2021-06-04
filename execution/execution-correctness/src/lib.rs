// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

mod execution_correctness;
mod execution_correctness_manager;
mod local;
mod process;
mod remote_service;
mod serializer;
mod thread;

pub use crate::{
    execution_correctness::ExecutionCorrectness,
    execution_correctness_manager::ExecutionCorrectnessManager, process::Process,
};

#[cfg(test)]
mod tests;
