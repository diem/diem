// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::dispatch::{CostedReturnType, NativeReturnType, Result, StackAccessor};

const LENGTH_COST: u64 = 30; // TODO: determine experimentally

#[allow(unreachable_code)]
pub fn native_length<T: StackAccessor>(_accessor: T) -> Result<CostedReturnType> {
    unimplemented!("Computing length of a vector collection");
    let native_return = NativeReturnType::U64(0);
    Ok(CostedReturnType::new(LENGTH_COST, native_return))
}
