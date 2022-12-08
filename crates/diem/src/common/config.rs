// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::common::types::CliError;
use serde::{Deserialize, Serialize};
use std::{
    io::Write,
    path::{Path, PathBuf},
    str::FromStr,
};
use structopt::StructOpt;

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct Config {
    pub rpc_endpoint: String,
    pub faucet_endpoint: String,
    pub account_address: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            rpc_endpoint: "127.0.0.1:8080".to_string(),
            faucet_endpoint: "127.0.0.1:8081".to_string(),
            account_address: "TODO".to_string(),
        }
    }
}

impl Config {
    pub fn load(path: &Path) -> Result<Config, CliError> {
        let reader =
            std::fs::File::open(path).map_err(|e| CliError::ConfigNotFoundError(e.to_string()))?;
        serde_yaml::from_reader(reader).map_err(|e| CliError::ConfigLoadError(e.to_string()))
    }

    pub fn save(&self, path: &Path) -> Result<String, CliError> {
        let mut writer = std::fs::File::create(path)
            .map_err(|e| CliError::ConfigNotFoundError(e.to_string()))?;
        let contents =
            serde_yaml::to_vec(&self).map_err(|e| CliError::ConfigSaveError(e.to_string()))?;
        writer
            .write_all(&contents)
            .map_err(|e| CliError::ConfigSaveError(e.to_string()))?;
        Ok("Config saved successfully".to_string())
    }
}

impl FromStr for Config {
    type Err = CliError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let config: Config = serde_yaml::from_str(s).unwrap();
        Ok(config)
    }
}

#[derive(Clone, Debug, StructOpt)]
pub struct ConfigPath {
    #[structopt(long)]
    pub config_path: Option<PathBuf>,
    pub config: Config,
}

impl Default for ConfigPath {
    fn default() -> Self {
        let configfile = match home::home_dir() {
            Some(path) => path.display().to_string() + "/.diem/config.yaml",
            None => "config.yaml".to_string(),
        };
        ConfigPath {
            config_path: Some(PathBuf::from(configfile)),
            config: Config::default(),
        }
    }
}

impl ConfigPath {
    pub fn load(&self) -> Result<Config, CliError> {
        if let Some(path) = &self.config_path {
            if !std::path::Path::new(path).exists() {
                Ok(Config::default())
            } else {
                Config::load(path)
            }
        } else {
            Ok(Config::default())
        }
    }

    pub fn save(&self, config: Config) -> Result<String, CliError> {
        if let Some(path) = &self.config_path {
            config.save(path)
        } else {
            Ok("Could not find config file".to_string())
        }
    }
}
