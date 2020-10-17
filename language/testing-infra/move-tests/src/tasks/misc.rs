// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use move_testing_base::{ops::OpGetOutputStream, tasks::Task};
use std::fmt::{Debug, Write};

use crate::ops::OpGetMoveStorage;

/// Task do dump the current state of the storage to the output.
#[derive(Debug)]
pub struct TaskShowMoveStorage;

impl<S> Task<S> for TaskShowMoveStorage
where
    S: OpGetOutputStream + OpGetMoveStorage,
    <S as OpGetMoveStorage>::Storage: Debug,
{
    fn run(self, mut state: S) -> Result<S> {
        let dump = format!("{:?}", state.get_move_storage());
        writeln!(state.get_output_stream(), "{}", dump)?;
        Ok(state)
    }
}
