// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::common::utils::{to_common_result, to_common_success_result};
use async_trait::async_trait;
use serde::Serialize;
use thiserror::Error;

// A common trait for all CLI commands to have consistent outputs
#[async_trait]
pub trait CliCommand<T: Serialize + Send>: Sized + Send {
    /// Returns a name for logging purposes
    fn command_name(&self) -> &'static str;

    /// Executes the command, returning a command specific type
    async fn execute(self) -> CliTypedResult<T>;

    /// Executes the command, and serializes it to the common JSON output type
    async fn execute_serialized(self) -> CliResult {
        to_common_result(self.execute().await).await
    }

    /// Executes the command, and throws away Ok(result) for the string Success
    async fn execute_serialized_success(self) -> CliResult {
        to_common_success_result(self.execute().await).await
    }
}

// A common result to be returned to users
pub type CliResult = Result<String, String>;

// A common result to remove need for typing `Result<T, CliError>`
pub type CliTypedResult<T> = Result<T, CliError>;

// CLI Errors for reporting through telemetry and outputs
#[derive(Debug, Error)]
pub enum CliError {
    #[error("Aborted command")]
    AbortedError,
    #[error("Invalid arguments: {0}")]
    CommandArgumentError(String),
    #[error("Unable to load config: {0} {1}")]
    ConfigLoadError(String, String),
    #[error("Unable to find config {0}, have you run `aptos init`?")]
    ConfigNotFoundError(String),
}

impl CliError {
    pub fn to_str(&self) -> &'static str {
        match self {
            CliError::AbortedError => "AbortedError",
            CliError::CommandArgumentError(_) => "CommandArgumentError",
            CliError::ConfigLoadError(_, _) => "ConfigLoadError",
            CliError::ConfigNotFoundError(_) => "ConfigNotFoundError",
        }
    }
}
