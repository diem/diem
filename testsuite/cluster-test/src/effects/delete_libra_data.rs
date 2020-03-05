// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use crate::{effects::Action, instance::Instance};
use anyhow::Result;

use async_trait::async_trait;
use libra_logger::info;
use std::fmt;

pub struct DeleteLibraData {
    instance: Instance,
}

impl DeleteLibraData {
    pub fn new(instance: Instance) -> Self {
        Self { instance }
    }
}

#[async_trait]
impl Action for DeleteLibraData {
    async fn apply(&self) -> Result<()> {
        info!("DeleteLibraData {}", self.instance);
        self.instance
            .run_cmd(vec!["sudo rm -rf /data/libra/common"])
            .await
    }
}

impl fmt::Display for DeleteLibraData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "DeleteLibraData {}", self.instance)
    }
}
