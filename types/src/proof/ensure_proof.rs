// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module declares the macro ensure_proof!,
//! which checks a condition for a proof and throws a ProofError
//! if the proof fails.
//! Currently, this macro only supports SparseMerkleProofs

use crate::proof::definition::SparseMerkleProof;
use crate::proof::proof_error::ProofError;

macro_rules! ensure_proof {
    ($proof:ident, $cond:expr, $e:expr) => {
        match $proof {
            SparseMerkleProof { .. } => {
                if !($cond) {
                    return Err(ProofError::new($proof.to_string(), $e.to_string()))?;
                }
            }
        }
        ensure!($cond, $e);
    };
    ($proof:ident, $cond:expr, $fmt:expr, $($arg:tt)+) => {
        match $proof {
            SparseMerkleProof { .. } => {
                if !($cond) {
                    return Err(ProofError::new($proof.to_string(), format!($fmt, $($arg)+)))?;
                }
            }
        }
        ensure!($cond, $fmt, $($arg)+);
    };
}
