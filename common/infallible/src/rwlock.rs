// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::sync::RwLock as StdRwLock;

pub use std::sync::{RwLockReadGuard, RwLockWriteGuard};

/// A simple wrapper around the lock() function of a std::sync::RwLock
/// The only difference is that you don't need to call unwrap() on it.
#[derive(Debug, Default)]
pub struct RwLock<T>(StdRwLock<T>);

impl<T> RwLock<T> {
    /// creates a read-write lock
    pub fn new(t: T) -> Self {
        Self(StdRwLock::new(t))
    }

    /// lock the rwlock in read mode
    pub fn read(&self) -> RwLockReadGuard<'_, T> {
        self.0
            .read()
            .expect("diem cannot currently handle a poisoned lock")
    }

    /// lock the rwlock in write mode
    pub fn write(&self) -> RwLockWriteGuard<'_, T> {
        self.0
            .write()
            .expect("diem cannot currently handle a poisoned lock")
    }

    /// return the owned type consuming the lock
    pub fn into_inner(self) -> T {
        self.0
            .into_inner()
            .expect("diem cannot currently handle a poisoned lock")
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use std::{sync::Arc, thread};

    #[test]
    fn test_diem_rwlock() {
        let a = 7u8;
        let rwlock = Arc::new(RwLock::new(a));
        let rwlock2 = rwlock.clone();
        let rwlock3 = rwlock.clone();

        let thread1 = thread::spawn(move || {
            let mut b = rwlock2.write();
            *b = 8;
        });
        let thread2 = thread::spawn(move || {
            let mut b = rwlock3.write();
            *b = 9;
        });

        let _ = thread1.join();
        let _ = thread2.join();

        let _read = rwlock.read();
    }
}
