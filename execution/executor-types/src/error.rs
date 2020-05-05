// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_crypto::HashValue;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Deserialize, Error, PartialEq, Serialize)]
/// Different reasons for proposal rejection
pub enum Error {
    #[error("Cannot find speculation result for block id {0}")]
    BlockNotFound(HashValue),

    #[error("Internal error: {:?}", error)]
    InternalError { error: String },

    #[error("Serialization error: {0}")]
    SerializationError(String),
}

impl From<anyhow::Error> for Error {
    fn from(error: anyhow::Error) -> Self {
        Self::InternalError {
            error: format!("{}", error),
        }
    }
}

impl From<lcs::Error> for Error {
    fn from(error: lcs::Error) -> Self {
        Self::SerializationError(format!("{}", error))
    }
}

impl From<libra_secure_net::Error> for Error {
    fn from(error: libra_secure_net::Error) -> Self {
        Self::InternalError {
            error: format!("{}", error),
        }
    }
}
