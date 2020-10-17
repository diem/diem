// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;

pub trait Task<S> {
    fn run(self, state: S) -> Result<S>;
}
