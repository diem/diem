// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    xcontext::execution_location::project_root, xcontext::project_metadata::Config, Result,
};
use x_core::XCoreContext;

/// Global context shared across x commands.
pub struct XContext {
    core: XCoreContext,
    config: Config,
}

impl XContext {
    /// Creates a new `GlobalContext` by reading the config in the project root.
    pub fn new() -> Result<Self> {
        Ok(Self::with_config(Config::from_project_root()?))
    }

    /// Creates a new `GlobalContext` based on the given config.
    pub fn with_config(config: Config) -> Self {
        Self {
            core: XCoreContext::new(project_root()),
            config,
        }
    }

    /// Returns a reference to the config.
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Returns a reference to the core context.
    pub fn core(&self) -> &XCoreContext {
        &self.core
    }
}
