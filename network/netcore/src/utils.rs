// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub trait Captures<'a> {}

impl<'a, T> Captures<'a> for T {}
