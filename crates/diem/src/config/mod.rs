// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::common::types::Command;
use clap::Parser;

pub mod init_config;

//TODO Create .diem folder, config.yaml within .diem folder, and basic yaml structure
#[derive(Debug, Parser)]
pub enum ConfigTool {
    Init(init_config::InitConfig),
}

impl ConfigTool {
    pub async fn execute(self) -> Result<String, String> {
        match self {
            ConfigTool::Init(tool) => tool.execute_serialized().await,
        }
    }
}
