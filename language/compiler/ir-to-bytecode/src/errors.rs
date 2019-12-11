// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_types::vm_error::VMStatus;
use thiserror::Error;

#[derive(Clone, Debug, Eq, Error, Ord, PartialEq, PartialOrd)]
pub enum InternalCompilerError {
    #[error("Post-compile bounds check errors: {0:?}")]
    BoundsCheckErrors(Vec<VMStatus>),
}
