// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

// TODO: add diagram

#![allow(dead_code)]
pub mod buffer_manager;
pub mod commit_phase;
pub mod errors;
pub mod execution_phase;
pub mod ordering_state_computer;
pub mod persisting_phase;
pub mod pipeline_phase;
pub mod signing_phase;

#[cfg(test)]
mod tests;
