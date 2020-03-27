// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! State management for the Move VM.
//!
//! The core Move VM is designed to be stateless -- while it maintains internal caches, data
//! storage is managed externally through the `ChainState` trait. This crate provides
//! implementations for the `ChainState` trait for various scenarios.
//!
//! Using these implementations is not mandatory -- if you are integrating with the Move VM you are
//! welcome to write your own.

pub mod data_cache;
pub mod execution_context;
