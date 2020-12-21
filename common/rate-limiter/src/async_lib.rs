// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::rate_limit::Bucket;
use diem_infallible::Mutex;
use futures::{
    io::AsyncRead,
    ready,
    task::{Context, Poll},
    AsyncWrite, Future,
};
use pin_project::pin_project;
use std::{io, pin::Pin, sync::Arc};
use tokio::time::{delay_until, Delay};

/// An inner struct for keeping track of the delay, and bucket of rate limiting
struct PollRateLimiter {
    bucket: Arc<Mutex<Bucket>>,
    delay: Option<Delay>,
}

impl PollRateLimiter {
    fn new(bucket: Option<Arc<Mutex<Bucket>>>) -> Self {
        let bucket = bucket.unwrap_or_else(|| Arc::new(Mutex::new(Bucket::open())));
        PollRateLimiter {
            bucket,
            delay: None,
        }
    }

    /// Poll and attempt to acquire the `requested` amount of tokens.
    /// Keep trying until some amount of tokens are acquired.  Note: This doesn't provide
    /// fairness so if two pollers hold the same bucket, one could continually lose.
    fn poll_acquire(&mut self, cx: &mut Context<'_>, requested: usize) -> Poll<usize> {
        loop {
            // Wait until the delay is finished.
            if let Some(ref mut delay) = self.delay {
                ready!(Pin::new(delay).poll(cx));
                self.delay = None;
            }
            // Try to acquire some tokens. If we're rate limited, we have to wait
            // before trying again.
            match self.bucket.lock().acquire_tokens(requested) {
                Ok(allowed) => return Poll::Ready(allowed),
                Err(wait_time) => {
                    self.delay = Some(delay_until(tokio::time::Instant::from_std(wait_time)));
                }
            }
        }
    }

    /// Poll an inner reader or writer, rate limited by the `bucket`.  This will provide an amount
    /// of bytes allowed ot be read including partial reads.  It will return unused tokens back to
    /// the bucket on partial reads / writes.
    pub fn poll_limited<
        T,
        Action: FnOnce(Pin<&mut T>, &mut Context<'_>, usize) -> Poll<io::Result<usize>>,
    >(
        &mut self,
        resource: Pin<&mut T>,
        cx: &mut Context<'_>,
        requested: usize,
        poll_resource: Action,
    ) -> Poll<io::Result<usize>> {
        let allowed = ready!(self.poll_acquire(cx, requested));
        let result = poll_resource(resource, cx, allowed);

        // In order to have an accurate throttle rate, we remove tokens, and then add ones we don't use
        let tokens_to_return = match &result {
            Poll::Ready(Ok(actual)) => allowed.saturating_sub(*actual),
            _ => allowed,
        };
        self.bucket.lock().add_tokens(tokens_to_return);

        result
    }
}

/// A rate limiter for `AsyncRead` or `AsyncWrite` interfaces to rate limit read/write bytes
///
/// This will pause and wait to send any future bytes until it's permitted to in the future
#[pin_project]
pub struct AsyncRateLimiter<T> {
    #[pin]
    inner: T,
    rate_limiter: PollRateLimiter,
}

impl<T> AsyncRateLimiter<T> {
    pub fn new(inner: T, bucket: Option<Arc<Mutex<Bucket>>>) -> Self {
        Self {
            inner,
            rate_limiter: PollRateLimiter::new(bucket),
        }
    }
}

impl<T: AsyncRead> AsyncRead for AsyncRateLimiter<T> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        let this = self.project();
        this.rate_limiter
            .poll_limited(this.inner, cx, buf.len(), |resource, cx, allowed| {
                resource.poll_read(cx, &mut buf[..allowed])
            })
    }
}

impl<T: AsyncWrite> AsyncWrite for AsyncRateLimiter<T> {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        let this = self.project();
        this.rate_limiter
            .poll_limited(this.inner, cx, buf.len(), |resource, cx, allowed| {
                resource.poll_write(cx, &buf[..allowed])
            })
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        self.project().inner.poll_flush(cx)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        self.project().inner.poll_close(cx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::AsyncReadExt;
    use std::num::NonZeroUsize;

    macro_rules! NonZeroUsize {
        ($n:expr) => {
            NonZeroUsize::new($n).unwrap()
        };
    }

    #[tokio::test]
    async fn test_async_read() {
        let source: &[u8] = b"12345678901234567890123456";
        let rate_limiter = Arc::new(Mutex::new(Bucket::new(NonZeroUsize!(15), NonZeroUsize!(5))));
        let mut reader = AsyncRateLimiter::new(source, Some(rate_limiter));

        let mut buf: [u8; 30] = [0; 30];
        assert_eq!(15, reader.read(&mut buf).await.expect("Successful read"));
        assert_eq!(
            5,
            reader.read(&mut buf[15..]).await.expect("Successful read")
        );
        assert_eq!(
            5,
            reader.read(&mut buf[20..]).await.expect("Successful read")
        );
        assert_eq!(
            1,
            reader.read(&mut buf[25..]).await.expect("Successful read")
        );
        assert_eq!(
            0,
            reader.read(&mut buf[26..]).await.expect("Successful read")
        );

        assert_eq!(&buf[..26], source);
    }
}
