// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{config::Config, installer::Installer, utils::project_root, Result};
use anyhow::Context;
use x_core::XCoreContext;

/// Global context shared across x commands.
pub struct XContext {
    core: XCoreContext,
    config: Config,
    installer: Installer,
}

impl XContext {
    /// Creates a new `GlobalContext` by reading the config in the project root.
    pub fn new() -> Result<Self> {
        Self::with_config(Config::from_project_root()?)
    }

    /// Creates a new `GlobalContext` based on the given config.
    pub fn with_config(config: Config) -> Result<Self> {
        let current_dir =
            std::env::current_dir().with_context(|| "error while fetching current dir")?;
        Ok(Self {
            core: XCoreContext::new(project_root(), current_dir)?,
            installer: Installer::new(config.cargo_config().clone(), config.tools()),
            config,
        })
    }

    /// Returns a reference to the config.
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Returns a reference to the core context.
    pub fn core(&self) -> &XCoreContext {
        &self.core
    }

    /// Returns a reference to Installer, configured to install versions from config.
    pub fn installer(&self) -> &Installer {
        &self.installer
    }
}
