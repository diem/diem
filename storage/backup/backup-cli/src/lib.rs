// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub mod adapter;
pub mod backup;
pub mod restore;

pub type FileHandle = String;

#[cfg(test)]
mod tests;
