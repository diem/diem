// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::Swarm;

/// Trait used to represent a interface for constructing a launching new networks
pub trait Factory {
    fn launch_swarm(&self) -> Box<dyn Swarm>;
}
