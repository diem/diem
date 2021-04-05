// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

/// Trait used to represent a running Validator or FullNode
pub trait Node {}

/// Trait used to represent a running Validator
pub trait Validator: Node {}

/// Trait used to represent a running FullNode
pub trait FullNode: Node {}
