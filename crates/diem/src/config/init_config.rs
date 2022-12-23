// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::common::{
    config::{Config, ConfigPath},
    types::{CliError, Command},
    utils::prompt_user,
};
use async_trait::async_trait;
use clap::Parser;

#[derive(Debug, Parser)]
pub struct InitConfig {
    #[arg(short = 'r', long = "rpc-endpoint")]
    pub rpc_endpoint: Option<String>,
    #[arg(short = 'f', long = "faucet-endpoint")]
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
        } else {
            // TODO: add input validation for all user prompts
            //       e.g. check if URLs/keys are valid format (or change URL input to network choice)
            let prompt_rpc_endpoint =
                prompt_user("Set RPC endpoint | default to testnet (no input)")?
                    .trim()
                    .to_ascii_lowercase();
            if prompt_rpc_endpoint.is_empty() {
                println!(
                    "No input provided, RPC endpoint set to '{}'",
                    config.rpc_endpoint
                )
            } else {
                config.rpc_endpoint = prompt_rpc_endpoint;
            }
        }
        if let Some(faucet_endpoint) = &self.faucet_endpoint {
            config.faucet_endpoint = faucet_endpoint.clone();
        } else {
            let prompt_faucet_endpoint =
                prompt_user("Set faucet endpoint | default to testnet (no input)")?
                    .trim()
                    .to_ascii_lowercase();
            if prompt_faucet_endpoint.is_empty() {
                println!(
                    "No input provided, faucet endpoint set to '{}'",
                    config.faucet_endpoint
                )
            } else {
                config.faucet_endpoint = prompt_faucet_endpoint;
            }
        }
        config_path.save(config)
    }
}
