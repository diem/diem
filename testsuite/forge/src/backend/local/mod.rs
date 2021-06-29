// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{Factory, Swarm};

mod node;
mod swarm;
pub use node::LocalNode;
pub use swarm::{LocalSwarm, LocalSwarmBuilder};

pub struct LocalFactory {
    diem_node_bin: String,
}

impl LocalFactory {
    pub fn new(diem_node_bin: &str) -> Self {
        Self {
            diem_node_bin: diem_node_bin.into(),
        }
    }
}

impl Factory for LocalFactory {
    fn launch_swarm(&self, node_num: usize) -> Box<dyn Swarm> {
        let mut swarm = LocalSwarm::builder(&self.diem_node_bin)
            .number_of_validators(node_num)
            .build()
            .unwrap();
        swarm.launch().unwrap();

        Box::new(swarm)
    }
}
