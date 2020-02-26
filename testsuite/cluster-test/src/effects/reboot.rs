// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use crate::{effects::Action, instance::Instance};
use anyhow::Result;

use async_trait::async_trait;
use slog_scope::info;
use std::{fmt, time::Duration};
use tokio::time;

pub struct Reboot {
    instance: Instance,
}

impl Reboot {
    pub fn new(instance: Instance) -> Self {
        Self { instance }
    }
}

#[async_trait]
impl Action for Reboot {
    async fn apply(&self) -> Result<()> {
        info!("Rebooting {}", self.instance);
        self.instance
            .run_cmd(vec![
                "touch /dev/shm/cluster_test_reboot; nohup sudo /usr/sbin/reboot &",
            ])
            .await?;
        loop {
            time::delay_for(Duration::from_secs(5)).await;
            match self
                .instance
                .run_cmd(vec!["! cat /dev/shm/cluster_test_reboot"])
                .await
            {
                Ok(..) => {
                    info!("Rebooting {} complete", self.instance);
                    return Ok(());
                }
                Err(..) => {
                    info!(
                        "Rebooting {} in progress - did not reboot yet",
                        self.instance
                    );
                }
            }
        }
    }
}

impl fmt::Display for Reboot {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Reboot {}", self.instance)
    }
}
