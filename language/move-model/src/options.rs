// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ModelBuilderOptions {
    /// Ignore the "opaque" pragma on function specs when possible. The opaque can be ignored if
    /// - the function can be called by some unknown code (i.e., the function has `public` or
    ///   `public(script)` visibility), or
    /// - the function spec has a property that is marked as `[concrete]`.
    pub ignore_pragma_opaque_when_possible: bool,
}
