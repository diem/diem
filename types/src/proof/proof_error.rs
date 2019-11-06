// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module declares ProofError, a failure that represents a proof failure.

use failure::Fail;
use std::fmt::{Debug, Display, Formatter, Result};

// Represents an instance of proof failure
//
// proof is the String representation of the failed proof,
// msg is the error message that provides context for the failed proof
#[derive(Debug)]
pub struct ProofError {
    pub proof: String,
    pub msg: String,
}

impl ProofError {
    pub fn new(proof: String, msg: String) -> ProofError {
        ProofError { proof, msg }
    }
}

impl Fail for ProofError {}

impl Display for ProofError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(
            f,
            "ProofError (msg: {:?}, proof: {:?})",
            self.msg, self.proof
        )
    }
}
