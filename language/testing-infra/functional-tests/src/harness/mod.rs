// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

mod move_harness;
mod mvir_harness;

pub use move_harness::run_move_functional_test;
pub use mvir_harness::run_mvir_functional_test;
