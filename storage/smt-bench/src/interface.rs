// Copyright (c) The Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

use crate::{components::commit_queue::DbOp, types::Proof};
use anyhow::Result;
use diem_crypto::HashValue;
use diem_types::{transaction::Version, vec_bool::VecBool};

pub type Position = VecBool;

pub trait Store: 'static + Send + Sync {
    fn get_proof(
        &self,
        target_key: HashValue,
        version: Version,
        starting_depth: usize,
    ) -> Result<Proof>;

    fn put_nodes(&self, nodes: &[DbOp], version: Version) -> Result<()>;
}
