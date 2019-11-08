// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module declares the macro ensure_proof!,
//! which checks a condition for a proof and throws a ProofError
//! if the proof fails.
//! Currently, this macro only supports SparseMerkleProofs

use crate::proof::{definition::Proof, proof_error::ProofError};
use failure::Error;

macro_rules! ensure_proof {
    ($proof:ident, $cond:expr, $e:expr) => {
        if Proof::is_proof($proof) {
            if !($cond) {
                return Err(Error::from(ProofError::new($proof.to_string(), $e.to_string())));
            }
        } else {
            ensure!($cond, $e);
        }
    };
    ($proof:ident, $cond:expr, $fmt:expr, $($arg:tt)+) => {
        if Proof::is_proof($proof) {
            if !($cond) {
                return Err(Error::from(ProofError::new($proof.to_string(), format!($fmt, $($arg)+))));
            }
        } else {
            ensure!($cond, $fmt, $($arg)+);
        }
    };
}
