// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use move_binary_format::errors::VMError;
use thiserror::Error;

#[derive(Clone, Debug, Eq, Error, Ord, PartialEq, PartialOrd)]
pub enum InternalCompilerError {
    #[error("Post-compile bounds check errors: {0:?}")]
    BoundsCheckErrors(VMError),
}
