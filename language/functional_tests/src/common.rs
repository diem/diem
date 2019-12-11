// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::convert::AsRef;

/// Wrapper of an inner object with start and end source locations.
#[derive(Debug)]
pub struct Sp<T> {
    pub inner: T,
    pub start: usize,
    pub end: usize,
}

impl<T> Sp<T> {
    pub fn new(inner: T, start: usize, end: usize) -> Self {
        Self { inner, start, end }
    }

    pub fn into_inner(self) -> T {
        self.inner
    }

    pub fn as_inner(&self) -> &T {
        &self.inner
    }

    pub fn map<F, R>(self, f: F) -> Sp<R>
    where
        F: FnOnce(T) -> R,
    {
        Sp {
            inner: f(self.inner),
            start: self.start,
            end: self.end,
        }
    }

    pub fn into_line_sp(self, line: usize) -> LineSp<T> {
        LineSp {
            inner: self.inner,
            line,
            start: self.start,
            end: self.end,
        }
    }
}

impl<T> AsRef<T> for Sp<T> {
    fn as_ref(&self) -> &T {
        self.as_inner()
    }
}

/// Wrapper of an inner object with line, start and end source locations.
#[derive(Debug)]
pub struct LineSp<T> {
    pub inner: T,
    pub line: usize,
    pub start: usize,
    pub end: usize,
}

impl<T> LineSp<T> {
    pub fn new(inner: T, line: usize, start: usize, end: usize) -> Self {
        Self {
            inner,
            line,
            start,
            end,
        }
    }

    pub fn as_inner(&self) -> &T {
        &self.inner
    }

    pub fn map<F, R>(self, f: F) -> LineSp<R>
    where
        F: FnOnce(T) -> R,
    {
        LineSp {
            inner: f(self.inner),
            line: self.line,
            start: self.start,
            end: self.end,
        }
    }
}

impl<T> AsRef<T> for LineSp<T> {
    fn as_ref(&self) -> &T {
        self.as_inner()
    }
}

/// Checks if `s` starts with `prefix`. If yes, returns a reference to the remaining part
/// with the prefix stripped away.
pub fn strip<'a>(s: &'a str, prefix: &str) -> Option<&'a str> {
    if s.starts_with(prefix) {
        Some(&s[prefix.len()..])
    } else {
        None
    }
}
