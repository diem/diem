// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::fmt;

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
