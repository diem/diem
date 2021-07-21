// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::{Swarm, Version};
use crate::Result;
use std::num::NonZeroUsize;

/// Trait used to represent a interface for constructing a launching new networks
pub trait Factory {
    fn versions<'a>(&'a self) -> Box<dyn Iterator<Item = Version> + 'a>;

    fn launch_swarm(&self, node_num: NonZeroUsize, version: &Version) -> Result<Box<dyn Swarm>>;
}
