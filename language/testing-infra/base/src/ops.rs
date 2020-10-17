// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::fmt::Write;

pub trait OpGetOutputStream {
    type Output: Write;

    fn get_output_stream(&mut self) -> &mut Self::Output;
}

pub trait OpIntoOutput {
    fn into_output(self) -> String;
}

pub trait OpGetItem<T> {
    fn get(&self) -> &T;
}

pub trait OpGetItemMut<T>: OpGetItem<T> {
    fn get_mut(&mut self) -> &mut T;
}
