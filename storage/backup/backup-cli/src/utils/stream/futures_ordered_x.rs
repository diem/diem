// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

/// This is a copy of `futures::stream::futures_ordered` from `futures 0.3.6`, except that it uses
/// `FuturesUnorderedX` which provides concurrency control. So we can manage more futures without
/// too many activated at the same time.
use crate::utils::stream::futures_unordered_x::FuturesUnorderedX;
use futures::{
    ready,
    stream::{FusedStream, StreamExt},
    task::{Context, Poll},
    Future, Stream,
};
use pin_project::pin_project;
use std::{
    cmp::Ordering,
    collections::{binary_heap::PeekMut, BinaryHeap},
    pin::Pin,
};

#[pin_project]
#[must_use = "futures do nothing unless you `.await` or poll them"]
#[derive(Debug)]
struct OrderWrapper<T> {
    #[pin]
    data: T, // A future or a future's output
    index: usize,
}

impl<T> PartialEq for OrderWrapper<T> {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}

impl<T> Eq for OrderWrapper<T> {}

impl<T> PartialOrd for OrderWrapper<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for OrderWrapper<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        // BinaryHeap is a max heap, so compare backwards here.
        other.index.cmp(&self.index)
    }
}

impl<T> Future for OrderWrapper<T>
where
    T: Future,
{
    type Output = OrderWrapper<T::Output>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let index = self.index;
        self.project().data.poll(cx).map(|output| OrderWrapper {
            data: output,
            index,
        })
    }
}

#[must_use = "streams do nothing unless polled"]
pub struct FuturesOrderedX<T: Future> {
    in_progress_queue: FuturesUnorderedX<OrderWrapper<T>>,
    queued_outputs: BinaryHeap<OrderWrapper<T::Output>>,
    next_incoming_index: usize,
    next_outgoing_index: usize,
}

impl<T: Future> Unpin for FuturesOrderedX<T> {}

impl<Fut: Future> FuturesOrderedX<Fut> {
    /// Constructs a new, empty `FuturesOrdered`
    ///
    /// The returned `FuturesOrdered` does not contain any futures and, in this
    /// state, `FuturesOrdered::poll_next` will return `Poll::Ready(None)`.
    pub fn new(max_in_progress: usize) -> FuturesOrderedX<Fut> {
        FuturesOrderedX {
            in_progress_queue: FuturesUnorderedX::new(max_in_progress),
            queued_outputs: BinaryHeap::new(),
            next_incoming_index: 0,
            next_outgoing_index: 0,
        }
    }

    /// Returns the number of futures contained in the queue.
    ///
    /// This represents the total number of in-flight futures, both
    /// those currently processing and those that have completed but
    /// which are waiting for earlier futures to complete.
    pub fn len(&self) -> usize {
        self.in_progress_queue.len() + self.queued_outputs.len()
    }

    /// Returns `true` if the queue contains no futures
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.in_progress_queue.is_empty() && self.queued_outputs.is_empty()
    }

    /// Push a future into the queue.
    ///
    /// This function submits the given future to the internal set for managing.
    /// This function will not call `poll` on the submitted future. The caller
    /// must ensure that `FuturesOrdered::poll` is called in order to receive
    /// task notifications.
    pub fn push(&mut self, future: Fut) {
        let wrapped = OrderWrapper {
            data: future,
            index: self.next_incoming_index,
        };
        self.next_incoming_index += 1;
        self.in_progress_queue.push(wrapped);
    }
}

impl<Fut: Future> Stream for FuturesOrderedX<Fut> {
    type Item = Fut::Output;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = &mut *self;

        // Check to see if we've already received the next value
        if let Some(next_output) = this.queued_outputs.peek_mut() {
            if next_output.index == this.next_outgoing_index {
                this.next_outgoing_index += 1;
                return Poll::Ready(Some(PeekMut::pop(next_output).data));
            }
        }

        loop {
            match ready!(this.in_progress_queue.poll_next_unpin(cx)) {
                Some(output) => {
                    if output.index == this.next_outgoing_index {
                        this.next_outgoing_index += 1;
                        return Poll::Ready(Some(output.data));
                    } else {
                        this.queued_outputs.push(output)
                    }
                }
                None => return Poll::Ready(None),
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl<Fut: Future> std::fmt::Debug for FuturesOrderedX<Fut> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FuturesOrderedX {{ ... }}")
    }
}

impl<Fut: Future> FusedStream for FuturesOrderedX<Fut> {
    fn is_terminated(&self) -> bool {
        self.in_progress_queue.is_terminated() && self.queued_outputs.is_empty()
    }
}

impl<Fut: Future> Extend<Fut> for FuturesOrderedX<Fut> {
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = Fut>,
    {
        for item in iter.into_iter() {
            self.push(item);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::FuturesOrderedX;
    use futures::StreamExt;
    use proptest::{collection::vec, prelude::*};
    use tokio::{runtime::Runtime, time::Duration};

    proptest! {
        #[test]
        fn test_run(
            sleeps_ms in vec(0u64..10, 0..100),
            max_in_progress in 1usize..100,
        ) {
            let rt = Runtime::new().unwrap();
            rt.block_on(async {
                let num_sleeps = sleeps_ms.len();
                let mut futures = FuturesOrderedX::new(max_in_progress);
                assert!(futures.is_empty());

                futures.extend(sleeps_ms.into_iter().enumerate().map(|(n, sleep_ms)| async move {
                        tokio::time::sleep(Duration::from_millis(sleep_ms)).await;
                        n
                }));
                assert!(num_sleeps > 0 || futures.is_empty());
                assert_eq!(
                    futures.collect::<Vec<_>>().await,
                    (0..num_sleeps).collect::<Vec<_>>()
                );
            });
        }
    }
}
