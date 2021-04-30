// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::Test;
use crate::{CoreContext, Swarm};
use cluster_test::{cluster::Cluster, report::SuiteReport, tx_emitter::TxEmitter};

/// The testing interface which defines a test written with full control over an existing network.
/// Tests written against this interface will have access to both the Root account as well as the
/// nodes which comprise the network.
pub trait NetworkTest: Test {
    /// Executes the test against the given context.
    fn run<'t>(&self, ctx: &mut NetworkContext<'t>) -> anyhow::Result<()>;
}

pub struct NetworkContext<'t> {
    core: CoreContext,

    swarm: &'t mut dyn Swarm,
}

impl<'t> NetworkContext<'t> {
    pub fn new(core: CoreContext, swarm: &'t mut dyn Swarm) -> Self {
        Self { core, swarm }
    }

    pub fn swarm(&mut self) -> &mut dyn Swarm {
        self.swarm
    }

    pub fn core(&mut self) -> &mut CoreContext {
        &mut self.core
    }

    pub fn report(&self) -> SuiteReport {
        SuiteReport::new()
    }

    pub fn tx_emitter(&self, cluster: &Cluster, vasp: bool) -> TxEmitter {
        TxEmitter::new(cluster, vasp)
    }

    pub fn print_report(&self, report: &SuiteReport) {
        let json_report =
            serde_json::to_string_pretty(report).expect("Failed to serialize report to json");
        println!(
            "\n====json-report-begin===\n{}\n====json-report-end===",
            json_report
        );
    }
}
