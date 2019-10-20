// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::sync::Mutex;
use typed_arena::Arena as TypedArena;

/// A thread-safe variant of `typed_arena::Arena`.
///
/// This implements `Send` and `Sync` if `T` is `Send`.
pub struct Arena<T> {
    inner: Mutex<TypedArena<T>>,
}

impl<T> Default for Arena<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Arena<T> {
    #[inline]
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(TypedArena::new()),
        }
    }

    #[inline]
    pub fn with_capacity(n: usize) -> Self {
        Self {
            inner: Mutex::new(TypedArena::with_capacity(n)),
        }
    }

    // This is safe because it's part of the API design.
    #[allow(clippy::mut_from_ref)]
    pub fn alloc(&self, value: T) -> &mut T {
        let arena = self.inner.lock().expect("lock poisoned");
        let value = arena.alloc(value);
        // Extend the lifetime of the value to that of the arena. typed_arena::Arena guarantees
        // that the value will never be moved out from underneath, and this wrapper guarantees
        // that the arena will not be dropped.
        unsafe { ::std::mem::transmute::<&mut T, &mut T>(value) }
    }

    #[inline]
    pub fn into_vec(self) -> Vec<T> {
        let arena = self.inner.into_inner().expect("lock poisoned");
        arena.into_vec()
    }
}

#[test]
fn arena_thread_safe() {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}

    assert_send::<Arena<String>>();
    assert_sync::<Arena<String>>();
}
