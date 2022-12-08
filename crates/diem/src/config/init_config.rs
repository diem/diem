// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::common::{
    config::{Config, ConfigPath},
    types::{CliError, Command},
};
use async_trait::async_trait;
use clap::Parser;

#[derive(Debug, Parser)]
pub struct InitConfig {
    #[arg(short = 'r', long = "rpc-endpoint")]
    pub rpc_endpoint: Option<String>,
    pub faucet_endpoint: Option<String>,
}

#[async_trait]
impl Command<String> for InitConfig {
    fn command_name(&self) -> &'static str {
        "InitConfig"
    }

    async fn execute(self) -> Result<String, CliError> {
        let config_path = ConfigPath::default();
        if let Some(path) = &config_path.config_path {
            if !std::path::Path::new(&path).exists() {
                match std::fs::create_dir_all(&path.parent().unwrap()) {
                    Ok(_) => (),
                    Err(e) => {
                        panic! {"Error creating directory to save config file: {}", e}
                    }
                };

                match std::fs::File::create(&path) {
                    Ok(_) => (),
                    Err(e) => {
                        panic! {"Error creating config file: {}", e}
                    }
                };
            }
        }
        let mut config = Config::default();
        if let Some(rpc_endpoint) = &self.rpc_endpoint {
            config.rpc_endpoint = rpc_endpoint.clone();
        }
        if let Some(faucet_endpoint) = &self.faucet_endpoint {
            config.faucet_endpoint = faucet_endpoint.clone();
        }
        config_path.save(config)
    }
}
