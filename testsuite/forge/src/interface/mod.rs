// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

mod admin;
pub use admin::*;
mod network;
pub use network::*;
mod test;
pub use test::*;
mod public;
pub use public::*;
mod factory;
pub use factory::*;
mod swarm;
pub use swarm::*;
mod node;
pub use node::*;
mod chain_info;
pub use chain_info::*;

use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Version(usize, String);

impl Version {
    pub fn new(version: usize, display_string: String) -> Self {
        Self(version, display_string)
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.1)
    }
}
