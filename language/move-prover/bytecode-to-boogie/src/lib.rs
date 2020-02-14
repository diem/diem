// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

//! Move prover modules

#[macro_use]
pub mod code_writer;
pub mod boogie_helpers;
pub mod boogie_wrapper;
pub mod bytecode_translator;
pub mod cli;
pub mod dataflow_analysis;
pub mod driver;
pub mod env;
pub mod spec_translator;
pub mod stackless_control_flow_graph;
