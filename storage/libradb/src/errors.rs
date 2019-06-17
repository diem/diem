// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module defines error types used by [`LibraDB`](crate::LibraDB).

use failure::Fail;

/// This enum defines errors commonly used among [`LibraDB`](crate::LibraDB) APIs.
#[derive(Debug, Fail)]
pub enum LibraDbError {
    /// A requested item is not found.
    #[fail(display = "{} not found.", _0)]
    NotFound(String),
}
