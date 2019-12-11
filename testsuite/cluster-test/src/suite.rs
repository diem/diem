// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]
use std::cmp::min;

use crate::{
    cluster::Cluster,
    experiments::{
        Experiment, PerformanceBenchmarkNodesDown, PerformanceBenchmarkNodesDownParams,
        PerformanceBenchmarkThreeRegionSimulation, RebootRandomValidators,
        RebootRandomValidatorsParams,
    },
};

pub struct ExperimentSuite {
    pub experiments: Vec<Box<dyn Experiment>>,
}

impl ExperimentSuite {
    pub fn new_pre_release(cluster: &Cluster) -> Self {
        let mut experiments = vec![];
        let count = min(3, cluster.instances().len() / 3);
        // Reboot different sets of 3 validators *100 times
        for _ in 0..20 {
            let b: Box<dyn Experiment> = Box::new(RebootRandomValidators::new(
                RebootRandomValidatorsParams { count },
                cluster,
            ));
            experiments.push(b);
        }
        experiments.push(Box::new(PerformanceBenchmarkNodesDown::new(
            PerformanceBenchmarkNodesDownParams { num_nodes_down: 0 },
            cluster,
        )));
        experiments.push(Box::new(PerformanceBenchmarkNodesDown::new(
            PerformanceBenchmarkNodesDownParams { num_nodes_down: 10 },
            cluster,
        )));
        experiments.push(Box::new(PerformanceBenchmarkThreeRegionSimulation::new(
            cluster,
        )));
        Self { experiments }
    }

    pub fn new_perf_suite(cluster: &Cluster) -> Self {
        let mut experiments: Vec<Box<dyn Experiment>> = vec![];
        experiments.push(Box::new(PerformanceBenchmarkNodesDown::new(
            PerformanceBenchmarkNodesDownParams { num_nodes_down: 0 },
            cluster,
        )));
        experiments.push(Box::new(PerformanceBenchmarkNodesDown::new(
            PerformanceBenchmarkNodesDownParams { num_nodes_down: 10 },
            cluster,
        )));
        experiments.push(Box::new(PerformanceBenchmarkThreeRegionSimulation::new(
            cluster,
        )));
        Self { experiments }
    }
}
