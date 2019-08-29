// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

/// This defines how tolerant the generator will be about deviating from
/// the starting stack height. The higher the tolerance, the more likely
/// it is for invalid programs to be generated within a given target number
/// of instructions.
/// Default is 0.9 for generating 1000 instruction sequences.
use std::fmt;

pub const MUTATION_TOLERANCE: f32 = 0.9;

#[derive(Debug)]
pub struct VMError {
    message: String,
}

impl VMError {
    pub fn new(message: String) -> VMError {
        VMError { message }
    }
}

impl fmt::Display for VMError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}
