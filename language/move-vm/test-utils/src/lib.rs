// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![allow(clippy::new_without_default)]

mod storage;

pub use storage::{BlankStorage, DeltaStorage, InMemoryStorage};
