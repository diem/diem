// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use crate::{interface::Position, Smt};
use anyhow::{anyhow, Result};
use diem_types::transaction::{StateTreeNode, Version};
use std::{collections::VecDeque, sync::Arc};

pub type DbOp = (Position, StateTreeNode);

pub struct Commit {
    smt: Smt,
    nodes: Arc<Vec<DbOp>>,
}

pub struct CommitQueue {
    committed_version: Version,
    committed_smt: Smt,
    to_commit: VecDeque<Commit>,
}

impl CommitQueue {
    pub fn new(committed_version: Version, committed_smt: Smt) -> Self {
        Self {
            committed_version,
            committed_smt,
            to_commit: VecDeque::new(),
        }
    }

    pub fn committed_version_and_latest_view(&self) -> (Version, Smt) {
        (
            self.committed_version,
            self.to_commit
                .back()
                .map(|commit| commit.smt.clone())
                .unwrap_or_else(|| self.committed_smt.clone()),
        )
    }

    pub fn next_to_commit(&self) -> Result<(Version, Arc<Vec<DbOp>>)> {
        let nodes = self
            .to_commit
            .front()
            .ok_or_else(|| anyhow!("Commit queue is empty."))?
            .nodes
            .clone();
        let version = self.committed_version + 1;

        Ok((version, nodes))
    }

    pub fn enqueue(&mut self, smt: Smt, nodes: Vec<DbOp>) {
        self.to_commit.push_back(Commit {
            smt,
            nodes: Arc::new(nodes),
        })
    }

    pub fn dequeue(&mut self) -> Result<Smt> {
        let committed = self
            .to_commit
            .pop_front()
            .ok_or_else(|| anyhow!("Commit queue is empty."))?;
        let smt = self.committed_smt.clone();
        self.committed_smt = committed.smt;
        self.committed_version += 1;
        Ok(smt)
    }
}
