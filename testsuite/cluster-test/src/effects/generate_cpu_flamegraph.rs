// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use crate::{effects::Action, instance::Instance};
use anyhow::Result;
use async_trait::async_trait;
use libra_logger::info;
use std::fmt;

pub struct GenerateCpuFlamegraph {
    instance: Instance,
    duration_secs: usize,
}

impl GenerateCpuFlamegraph {
    pub fn new(instance: Instance, duration_secs: usize) -> Self {
        Self {
            instance,
            duration_secs,
        }
    }
}

#[async_trait]
impl Action for GenerateCpuFlamegraph {
    async fn apply(&self) -> Result<()> {
        info!("GenerateCpuFlamegraph {}", self.instance);
        let cmd = format!(
            r#"
                set -e;
                rm -rf /tmp/perf-data;
                mkdir /tmp/perf-data;
                cd /tmp/perf-data;
                sudo perf record -F 99 -p $(ps aux | grep libra-node | grep -v grep | awk '{{print $2}}') --output=perf.data --call-graph dwarf -- sleep {};
                sudo perf script --input=perf.data | sudo /usr/local/etc/FlameGraph/stackcollapse-perf.pl > out.perf-folded;
                sudo /usr/local/etc/FlameGraph/flamegraph.pl out.perf-folded > perf-kernel.svg;
                "#,
            self.duration_secs
        );
        self.instance.run_cmd(vec![cmd.as_str()]).await?;
        self.instance
            .scp("/tmp/perf-data/perf-kernel.svg", "perf-kernel.svg")
            .await?;
        Ok(())
    }
}

impl fmt::Display for GenerateCpuFlamegraph {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "GenerateCpuFlamegraph {}", self.instance)
    }
}
