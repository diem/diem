// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![feature(box_syntax, box_patterns)]
#[macro_use]

pub mod module_generator;
mod bytecode_specifications;
mod common;
pub mod natives;
pub mod stack_generator;
pub mod vm_runner;
