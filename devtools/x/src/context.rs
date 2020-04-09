// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{config::Config, Result};

/// Global context shared across x commands.
pub struct XContext {
    config: Config,
}

impl XContext {
    /// Creates a new `GlobalContext` by reading the config in the project root.
    pub fn new() -> Result<Self> {
        Ok(Self::with_config(Config::from_project_root()?))
    }

    /// Creates a new `GlobalContext` based on the given config.
    pub fn with_config(config: Config) -> Self {
        Self { config }
    }

    /// Returns a reference to the config.
    pub fn config(&self) -> &Config {
        &self.config
    }
}
