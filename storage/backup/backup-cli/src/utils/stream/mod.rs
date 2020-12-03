// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

mod buffered_x;
mod futures_ordered_x;
mod futures_unordered_x;

use crate::utils::stream::buffered_x::BufferedX;
use futures::{Future, Stream};

pub(crate) trait StreamX: Stream {
    fn buffered_x(self, n: usize, max_in_progress: usize) -> BufferedX<Self>
    where
        Self::Item: Future,
        Self: Sized,
    {
        BufferedX::new(self, n, max_in_progress)
    }
}

impl<T: ?Sized> StreamX for T where T: Stream {}
