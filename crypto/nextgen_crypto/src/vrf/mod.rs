// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module contains implementations of a
//! [verifiable random function](https://en.wikipedia.org/wiki/Verifiable_random_function)
//! (currently only ECVRF). VRFs can be used in the consensus protocol for leader election.

pub mod ecvrf;

#[cfg(test)]
mod unit_tests;
