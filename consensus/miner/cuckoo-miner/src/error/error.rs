// Copyright 2017 The Grin Developers

use std::io;
use std::string;

#[derive(Debug)]
pub enum CuckooMinerError {
    /// Occurs when trying to call a plugin function when a
    /// mining plugin is not loaded.
    PluginNotLoadedError(String),

    /// Occurs when trying to load plugin function that doesn't exist
    PluginSymbolNotFoundError(String),

    /// Occurs when attempting to load a plugin that doesn't exist
    PluginNotFoundError(String),

    /// Occurs when trying to load a plugin directory that doesn't
    /// contain any plugins
    NoPluginsFoundError(String),

    /// Unexpected return code from a plugin
    UnexpectedResultError(u32),

    /// Error setting a parameter
    ParameterError(String),

    /// IO Error
    PluginIOError(String),

    /// Plugin processing can't start
    PluginProcessingError(String),

    /// Error getting stats or stats not implemented
    StatsError(String),
}

impl From<io::Error> for CuckooMinerError {
    fn from(error: io::Error) -> Self {
        CuckooMinerError::PluginIOError(String::from(format!("Error loading plugin: {}", error)))
    }
}

impl From<string::FromUtf8Error> for CuckooMinerError {
    fn from(error: string::FromUtf8Error) -> Self {
        CuckooMinerError::PluginIOError(String::from(format!(
            "Error loading plugin description: {}",
            error
        )))
    }
}
