// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::dispatch::NativeReturnStatus;
use crate::{loaded_data::types::Type, value::Local};
use std::collections::VecDeque;

const LENGTH_COST: u64 = 30; // TODO: determine experimentally

#[allow(unreachable_code)]
pub fn native_length(_types: VecDeque<Type>, _arguments: VecDeque<Local>) -> NativeReturnStatus {
    unimplemented!("Computing length of a vector collection");
    let cost = LENGTH_COST;
    let return_values = vec![Local::u64(0)];
    NativeReturnStatus::Success {
        cost,
        return_values,
    }
}
