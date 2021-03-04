// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

/// This is a copy of `futures::stream::buffered` from `futures 0.3.6`, except that it uses
/// `FuturesOrderedX` which provides concurrency control. So we can buffer more results without
/// too many futures driven at the same time.
use crate::utils::stream::futures_ordered_x::FuturesOrderedX;
use futures::{
    ready,
    stream::Fuse,
    task::{Context, Poll},
    Future, Stream, StreamExt,
};
use pin_project::pin_project;
use std::pin::Pin;

#[pin_project]
#[must_use = "streams do nothing unless polled"]
pub struct BufferedX<St>
where
    St: Stream,
    St::Item: Future,
{
    #[pin]
    stream: Fuse<St>,
    in_progress_queue: FuturesOrderedX<St::Item>,
    max: usize,
}

impl<St> std::fmt::Debug for BufferedX<St>
where
    St: Stream + std::fmt::Debug,
    St::Item: Future,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BufferedX")
            .field("stream", &self.stream)
            .field("in_progress_queue", &self.in_progress_queue)
            .field("max", &self.max)
            .finish()
    }
}

impl<St> BufferedX<St>
where
    St: Stream,
    St::Item: Future,
{
    pub(super) fn new(stream: St, n: usize, max_in_progress: usize) -> BufferedX<St> {
        assert!(n > 0);

        BufferedX {
            stream: stream.fuse(),
            in_progress_queue: FuturesOrderedX::new(max_in_progress),
            max: n,
        }
    }
}

impl<St> Stream for BufferedX<St>
where
    St: Stream,
    St::Item: Future,
{
    type Item = <St::Item as Future>::Output;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        // First up, try to spawn off as many futures as possible by filling up
        // our queue of futures.
        while this.in_progress_queue.len() < *this.max {
            match this.stream.as_mut().poll_next(cx) {
                Poll::Ready(Some(fut)) => this.in_progress_queue.push(fut),
                Poll::Ready(None) | Poll::Pending => break,
            }
        }

        // Attempt to pull the next value from the in_progress_queue
        let res = this.in_progress_queue.poll_next_unpin(cx);
        if let Some(val) = ready!(res) {
            return Poll::Ready(Some(val));
        }

        // If more values are still coming from the stream, we're not done yet
        if this.stream.is_done() {
            Poll::Ready(None)
        } else {
            Poll::Pending
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let queue_len = self.in_progress_queue.len();
        let (lower, upper) = self.stream.size_hint();
        let lower = lower.saturating_add(queue_len);
        let upper = match upper {
            Some(x) => x.checked_add(queue_len),
            None => None,
        };
        (lower, upper)
    }
}

#[cfg(test)]
mod tests {
    use super::super::StreamX;
    use futures::StreamExt;
    use proptest::{collection::vec, prelude::*};
    use tokio::{runtime::Runtime, time::Duration};

    proptest! {
        #[test]
        fn test_run(
            sleeps_ms in vec(0u64..10, 0..100),
            buffer_size in 1usize..100,
            max_in_progress in 1usize..100,
        ) {
            let rt = Runtime::new().unwrap();
            rt.block_on(async {
                let num_sleeps = sleeps_ms.len();

                let outputs = futures::stream::iter(
                    sleeps_ms.into_iter().enumerate().map(|(n, sleep_ms)| async move {
                        tokio::time::sleep(Duration::from_millis(sleep_ms)).await;
                        n
                    })
                ).buffered_x(buffer_size, max_in_progress)
                .collect::<Vec<_>>().await;

                assert_eq!(
                    outputs,
                    (0..num_sleeps).collect::<Vec<_>>()
                );
            });
        }
    }
}
