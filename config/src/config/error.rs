// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Invariant violation: {0}")]
    InvariantViolation(String),
    #[error("Error accessing {0}: {1}")]
    IO(String, #[source] std::io::Error),
    #[error("Error (de)serializing {0}: {1}")]
    LCS(&'static str, #[source] lcs::Error),
    #[error("Error (de)serializing {0}: {1}")]
    Yaml(String, #[source] serde_yaml::Error),
    #[error("Config is missing expected value: {0}")]
    Missing(&'static str),
}

pub fn invariant(cond: bool, msg: String) -> Result<(), Error> {
    if !cond {
        Err(Error::InvariantViolation(msg))
    } else {
        Ok(())
    }
}
