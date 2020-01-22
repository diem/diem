// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![recursion_limit = "1024"]

//! Admission Control
//!
//! Admission Control (AC) is the public API end point taking public gRPC requests from clients.
//! AC serves two types of request from clients:
//! 1. SubmitTransaction, to submit transaction to associated validator.
//! 2. UpdateToLatestLedger, to query storage, e.g. account state, transaction log, and proofs.

#[cfg(test)]
#[path = "unit_tests/admission_control_service_test.rs"]
mod admission_control_service_test;

#[cfg(feature = "fuzzing")]
/// Fuzzer for admission control
pub mod admission_control_fuzzing;
/// AC gRPC service.
pub mod admission_control_service;
#[cfg(feature = "fuzzing")]
/// Useful Mocks
pub mod mocks;
