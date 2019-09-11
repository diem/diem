// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::dispatch::NativeReturnStatus;
use crate::value::Local;
use std::collections::VecDeque;

const BORROW_COST: u64 = 30; // TODO: determine experimentally
const EMPTY_COST: u64 = 30; // TODO: determine experimentally
const LENGTH_COST: u64 = 30; // TODO: determine experimentally
const PUSH_BACK_COST: u64 = 30; // TODO: determine experimentally

#[allow(unreachable_code)]
pub fn native_length(_arguments: VecDeque<Local>) -> NativeReturnStatus {
    unimplemented!("Computing length of a vector collection");
    let cost = LENGTH_COST;
    let return_values = vec![Local::u64(0)];
    NativeReturnStatus::Success {
        cost,
        return_values,
    }
}

#[allow(unreachable_code)]
pub fn native_empty(_arguments: VecDeque<Local>) -> NativeReturnStatus {
    unimplemented!("Creating an empty vector");
    let cost = EMPTY_COST;
    // TODO: implement returning empty vector native struct here
    let empty_vector = Local::struct_(vec![]);
    let return_values = vec![empty_vector];
    NativeReturnStatus::Success {
        cost,
        return_values,
    }
}

#[allow(unreachable_code)]
pub fn native_borrow(_arguments: VecDeque<Local>) -> NativeReturnStatus {
    unimplemented!("borrowing an element from a vector");
    let cost = BORROW_COST;
    // TODO: bounds check + implement retrieving reference to element of vector here
    let vector_element = Local::struct_(vec![]);
    let return_values = vec![vector_element];
    NativeReturnStatus::Success {
        cost,
        return_values,
    }
}

#[allow(unreachable_code)]
pub fn native_push_back(_arguments: VecDeque<Local>) -> NativeReturnStatus {
    unimplemented!("Adding an element to a vector");
    let cost = PUSH_BACK_COST;
    let return_values = vec![];
    NativeReturnStatus::Success {
        cost,
        return_values,
    }
}
