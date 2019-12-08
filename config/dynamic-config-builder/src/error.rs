// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("index out of range: {} >= {}", index, nodes)]
    IndexError { index: usize, nodes: usize },
    #[error("Missing configs only found {}", found)]
    MissingConfigs { found: usize },
    #[error("Config does not contain a validator network")]
    MissingValidatorNetwork,
    #[error("network size should be at least 1")]
    NonZeroNetwork,
}
