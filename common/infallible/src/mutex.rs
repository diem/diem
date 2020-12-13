// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::sync::Mutex as StdMutex;

pub use std::sync::MutexGuard;

/// A simple wrapper around the lock() function of a std::sync::Mutex
/// The only difference is that you don't need to call unwrap() on it.
#[derive(Debug)]
pub struct Mutex<T>(StdMutex<T>);

impl<T> Mutex<T> {
    /// creates mutex
    pub fn new(t: T) -> Self {
        Self(StdMutex::new(t))
    }

    /// lock the mutex
    pub fn lock(&self) -> MutexGuard<'_, T> {
        self.0
            .lock()
            .expect("diem cannot currently handle a poisoned lock")
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use std::{sync::Arc, thread};

    #[test]
    fn test_diem_mutex() {
        let a = 7u8;
        let mutex = Arc::new(Mutex::new(a));
        let mutex2 = mutex.clone();
        let mutex3 = mutex.clone();

        let thread1 = thread::spawn(move || {
            let mut b = mutex2.lock();
            *b = 8;
        });
        let thread2 = thread::spawn(move || {
            let mut b = mutex3.lock();
            *b = 9;
        });

        let _ = thread1.join();
        let _ = thread2.join();

        let _locked = mutex.lock();
    }
}
