// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module provides a compatibility shim between traits in the `futures` and `tokio` crate.
use pin_project::pin_project;
use std::{
    io,
    pin::Pin,
    task::{self, Poll},
};

/// `IoCompat` provides a compatibility shim between the `AsyncRead`/`AsyncWrite` traits provided by
/// the `futures` library and those provided by the `tokio` library since they are different and
/// incompatible with one another.
#[pin_project]
#[derive(Copy, Clone, Debug)]
pub struct IoCompat<T> {
    #[pin]
    inner: T,
}

impl<T> IoCompat<T> {
    pub fn new(inner: T) -> Self {
        IoCompat { inner }
    }
}

impl<T> tokio::io::AsyncRead for IoCompat<T>
where
    T: futures::io::AsyncRead,
{
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut task::Context,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        futures::io::AsyncRead::poll_read(self.project().inner, cx, buf)
    }
}

impl<T> futures::io::AsyncRead for IoCompat<T>
where
    T: tokio::io::AsyncRead,
{
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut task::Context,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        tokio::io::AsyncRead::poll_read(self.project().inner, cx, buf)
    }
}

impl<T> tokio::io::AsyncWrite for IoCompat<T>
where
    T: futures::io::AsyncWrite,
{
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut task::Context,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        futures::io::AsyncWrite::poll_write(self.project().inner, cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut task::Context) -> Poll<io::Result<()>> {
        futures::io::AsyncWrite::poll_flush(self.project().inner, cx)
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut task::Context) -> Poll<io::Result<()>> {
        futures::io::AsyncWrite::poll_close(self.project().inner, cx)
    }
}

impl<T> futures::io::AsyncWrite for IoCompat<T>
where
    T: tokio::io::AsyncWrite,
{
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut task::Context,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        tokio::io::AsyncWrite::poll_write(self.project().inner, cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut task::Context) -> Poll<io::Result<()>> {
        tokio::io::AsyncWrite::poll_flush(self.project().inner, cx)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut task::Context) -> Poll<io::Result<()>> {
        tokio::io::AsyncWrite::poll_shutdown(self.project().inner, cx)
    }
}
