// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

// UNSAFE MODULE: This module requires auditing before modifications may land.
// JUSTIFICATION: Arena::alloc below uses unsafe to tie the lifetime of the
// returned type to the lifetime of the arena. This requires preserving the
// invariant that the values do not get deallocated before the arena is
// deallocated. The current code and the underlying typed_arena crate both
// preserve this invariant. Modifications to this module must also maintain
// this invariant, even if they don't touch the unsafe block below.
// AUDITOR: metajack

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
        // UNSAFE CODE: This code requires auditing before modifications may land.
        // JUSTIFICATION: This extends the lifetime of the returned reference
        // to the lifetime of the arena. typed_arena's will not move values
        // and will only deallocate when the arena goes out of scope. This
        // unsafe cast is used by typed_arena itself (see the implementation
        // of alloc_extend).
        // AUDITOR: metajack
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
