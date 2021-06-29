// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{Factory, Swarm};
use tokio::runtime::Runtime;

mod node;
mod swarm;
pub use node::K8sNode;
pub use swarm::*;

pub struct K8sFactory {
    root_key: String,
    treasury_compliance_key: String,
}

impl K8sFactory {
    pub fn new(root_key: String, treasury_compliance_key: String) -> Self {
        Self {
            root_key,
            treasury_compliance_key,
        }
    }
}

impl Factory for K8sFactory {
    fn launch_swarm(&self, _node_num: usize) -> Box<dyn Swarm> {
        let rt = Runtime::new().unwrap();
        let swarm = rt
            .block_on(K8sSwarm::new(
                self.root_key.clone(),
                self.treasury_compliance_key.clone(),
            ))
            .unwrap();
        Box::new(swarm)
    }
}
