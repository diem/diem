// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::rate_limit::Bucket;
use diem_infallible::Mutex;
use futures::{
    io::AsyncRead,
    task::{Context, Poll},
};
use pin_project::pin_project;
use std::{pin::Pin, sync::Arc};
use tokio::time::delay_until;

/// A rate limiter for `AsyncRead` interfaces to rate limit read bytes
///
/// This will pause and wait to send any future bytes until it's permitted to in the future
#[pin_project]
pub struct AsyncReadRateLimiter<T: AsyncRead + Unpin> {
    #[pin]
    reader: T,
    rate_limiter: Arc<Mutex<Bucket>>,
}

impl<T: AsyncRead + Unpin> AsyncReadRateLimiter<T> {
    pub fn new(reader: T, rate_limiter: Option<Arc<Mutex<Bucket>>>) -> Self {
        let rate_limiter = rate_limiter.unwrap_or_else(|| Arc::new(Mutex::new(Bucket::open())));

        Self {
            reader,
            rate_limiter,
        }
    }
}

impl<T: AsyncRead + Unpin> AsyncRead for AsyncReadRateLimiter<T> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<std::io::Result<usize>> {
        let requested = buf.len();

        let allowed: usize;
        match self.rate_limiter.lock().acquire_tokens(requested) {
            Ok(num) => allowed = num,
            Err(wait_time) => {
                // Wait until we can actually get a value
                let waker = cx.waker().clone();
                tokio::task::spawn(async move {
                    delay_until(tokio::time::Instant::from_std(wait_time)).await;
                    waker.wake_by_ref();
                });
                return Poll::Pending;
            }
        }

        // Otherwise, let's get as much as we're allowing
        let buf: &mut [u8] = &mut buf[..allowed];

        // In order to have an accurate throttle rate, we remove tokens, and then add ones we don't use
        // The other option is to just throttle on how much we request anyways, which is less accurate, but
        // faster.
        let result = Pin::new(&mut self.reader).poll_read(cx, buf);
        let tokens_to_return = match &result {
            Poll::Ready(Ok(actual)) => allowed.saturating_sub(*actual),
            _ => allowed,
        };

        // Return unused tokens
        self.rate_limiter.lock().add_tokens(tokens_to_return);

        result
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
        let mut reader = AsyncReadRateLimiter::new(source, Some(rate_limiter));

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
