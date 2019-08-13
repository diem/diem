// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use failure::Fail;
use vm::errors::VerificationError;

#[derive(Clone, Debug, Eq, Fail, Ord, PartialEq, PartialOrd)]
pub enum InternalCompilerError {
    #[fail(display = "Post-compile bounds check errors: {:?}", _0)]
    BoundsCheckErrors(Vec<VerificationError>),
}
