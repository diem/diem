// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]
use std::cmp::min;
use std::env;

use crate::experiments::ExperimentParam;
use crate::{
    cluster::Cluster,
    experiments::{
        Experiment, PerformanceBenchmarkNodesDownParams,
        PerformanceBenchmarkThreeRegionSimulationParams, RebootRandomValidatorsParams,
        RecoveryTimeParams,
    },
};

pub struct ExperimentSuite {
    pub experiments: Vec<Box<dyn Experiment>>,
}

impl ExperimentSuite {
    pub fn new_pre_release(cluster: &Cluster) -> Self {
        let mut experiments: Vec<Box<dyn Experiment>> = vec![];
        if env::var("RECOVERY_EXP").is_ok() {
            experiments.push(Box::new(
                RecoveryTimeParams {
                    num_accounts_to_mint: 100_000,
                }
                .build(cluster),
            ));
        }
        let count = min(3, cluster.instances().len() / 3);
        // Reboot different sets of 3 validators *100 times
        for _ in 0..10 {
            let b = Box::new(RebootRandomValidatorsParams { count }.build(cluster));
            experiments.push(b);
        }
        experiments.push(Box::new(
            PerformanceBenchmarkNodesDownParams { num_nodes_down: 0 }.build(cluster),
        ));
        experiments.push(Box::new(
            PerformanceBenchmarkNodesDownParams { num_nodes_down: 10 }.build(cluster),
        ));
        experiments.push(Box::new(
            PerformanceBenchmarkThreeRegionSimulationParams {}.build(cluster),
        ));
        Self { experiments }
    }

    pub fn new_perf_suite(cluster: &Cluster) -> Self {
        let mut experiments: Vec<Box<dyn Experiment>> = vec![];
        experiments.push(Box::new(
            PerformanceBenchmarkNodesDownParams { num_nodes_down: 0 }.build(cluster),
        ));
        experiments.push(Box::new(
            PerformanceBenchmarkNodesDownParams { num_nodes_down: 10 }.build(cluster),
        ));
        experiments.push(Box::new(
            PerformanceBenchmarkThreeRegionSimulationParams {}.build(cluster),
        ));
        Self { experiments }
    }
}
