// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("index out of range: {} >= {}", index, nodes)]
    IndexError { index: usize, nodes: usize },
    #[error("Invalid SafetyRules backend {0}.")]
    InvalidSafetyRulesBackend(String),
    #[error("Missing configs only found {}", found)]
    MissingConfigs { found: usize },
    #[error("Missing full node network")]
    MissingFullNodeNetwork,
    #[error("Network config is missing network keypairs")]
    MissingNetworkKeyPairs,
    #[error("Specified a service-oriented safety rules but did not provide a host")]
    MissingSafetyRulesHost,
    #[error("Specified a service-oriented safety rules but did not provide a token")]
    MissingSafetyRulesToken,
    #[error("Config does not contain a validator network")]
    MissingValidatorNetwork,
    #[error("Unable to find any configs")]
    NoConfigs,
    #[error("network size should be at least 1")]
    NonZeroNetwork,
}
