// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

mod execution_correctness_manager;
mod local_client;
mod process;
mod remote_service;
mod serializer;
mod spawned_process;
mod thread;

pub use crate::{execution_correctness_manager::ExecutionCorrectnessManager, process::Process};

#[cfg(any(test, feature = "testing"))]
#[path = "process_client_wrapper.rs"]
pub mod process_client_wrapper;

#[cfg(test)]
mod tests;
