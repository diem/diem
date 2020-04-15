// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

pub mod annotations;
pub mod dataflow_analysis;
pub mod eliminate_imm_refs;
pub mod function_target;
pub mod function_target_pipeline;
pub mod lifetime_analysis;
pub mod reaching_def_analysis;
pub mod stackless_bytecode;
pub mod stackless_bytecode_generator;
pub mod stackless_control_flow_graph;
