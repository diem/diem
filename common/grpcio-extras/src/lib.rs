// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use futures::Future;

pub trait Cancelable {
    fn cancel(&mut self);
}

impl<T: Future> Cancelable for T {
    fn cancel(&mut self) {
        unimplemented!();
    }
}
