// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use diem_types::trusted_state::TrustedState;
use std::{
    convert::Infallible,
    sync::{Arc, RwLock},
};

/// A `StateStore` provides persistent, durable storage for
/// [`VerifyingClient`](crate::VerifyingClient)s' latest [`TrustedState`].
///
/// ### Implementor Guarantees
///
/// Critically, the `StateStore` must provide certain properties in order to
/// uphold the client's safety guarantees. We say that a client is "safe" so long
/// as their observed durable states are monotonically increasing by version.
///
/// Stores should be atomic and durable. A state is considered durable after a
/// successful `store`. A client should never observe a stale state after a
/// crash or restart. Stores should be atomic, i.e., a crash or restart during
/// a store should not corrupt the existing stored state.
///
/// A `StateStore` only cares about storing the _latest_ [`TrustedState`]. Calls
/// to `StateStore::store` with a [`TrustedState`] older than the latest durable
/// state version must not affect the latest stored state and are safe to ignore.
///
/// At the minimum, a client should "Read-my-Writes". In other words, a client's
/// `state_store.store(s1)` followed by `s2 = state_store.latest_state()` should
/// satisfy `s1.version <= s2.version`.
///
/// ### Concurrency
///
/// A user may have multiple concurrent threads with `VerifyingClient`s attempting
/// to store new `TrustedState`s in the same `StateStore`, which should be supported
/// without compromising the above guarantees.
///
/// If a client process (with potentially many threads) is the sole reader and
/// writer, a `StateStore` can be wrapped in a `WriteThroughCache`, which removes
/// the need to read from the store and avoids some stale writes to the store.
/// The provided `InMemoryStateStore` and `FileStateStore` already do this.
///
/// ### Example
///
/// For instance, a SQLite-based `StateStore` might be implemented using queries
/// like:
///
/// ```sql
/// -- a table always containing the highest trusted_state
/// CREATE TABLE trusted_state (
///     id INTEGER NOT NULL PRIMARY KEY,
///     version INTEGER NOT NULL,
///     state_blob BLOB NOT NULL
/// );
///
/// -- StateStore::latest_state()
/// -- reading the latest state
/// SELECT id, version, state_blob FROM trusted_state WHERE id = 0;
///
/// -- StateStore::store(new_state: TrustedState)
/// -- maybe insert a new trusted_state, where ?1 is the new version and ?2 is the new
/// -- serialized state blob. ensures that the stored trusted state is always
/// -- the one with the greatest version.
/// INSERT OR REPLACE INTO trusted_state (id, version, state_blob)
/// SELECT id, version, state_blob FROM trusted_state UNION SELECT 0, ?1, ?2 ORDER BY version DESC LIMIT 1
/// ```
pub trait StateStore {
    type Error: std::error::Error + Send + Sync + 'static;

    /// Get the latest durable, committed state. Returns `None` if there is no
    /// state yet in the underlying store.
    fn latest_state(&self) -> Result<Option<TrustedState>, Self::Error>;

    /// Get the version of the latest durable, committed state. Returns `None` if
    /// there is no state yet in the underlying store.
    ///
    /// Note: `StateStore` provides a default impl using `Self::latest_state`.
    /// Implementors may want override this method if they can do it more efficiently.
    fn latest_state_version(&self) -> Result<Option<u64>, Self::Error> {
        Ok(self.latest_state()?.map(|s| s.version()))
    }

    /// Store a new [`TrustedState`] in this `StateStore`.
    ///
    /// If there is already a newer, durable `TrustedState` in the store, we can
    /// ignore `new_state`.
    ///
    /// When this call returns, clients will assume that `new_state` or some newer
    /// state is durable.
    fn store(&self, new_state: &TrustedState) -> Result<(), Self::Error>;
}

/// An in-memory `StateStore`. Used for testing.
#[derive(Debug, Clone)]
pub struct InMemoryStateStore(Arc<WriteThroughCache<NoopStateStore>>);

/// This `StateStore` ignores all stores and always returns `Ok(None)`.
/// Used for testing.
#[derive(Debug)]
struct NoopStateStore;

/// A write-through cache around an underlying durable [`StateStore`].
///
/// As a write-through cache:
/// 1. Reads are serviced directly from the cache.
/// 2. Writes are committed to the underlying store before updating the cache.
#[derive(Debug)]
pub struct WriteThroughCache<S> {
    durable_state_cache: RwLock<Option<TrustedState>>,
    state_store: S,
}

///////////////////////
// WriteThroughCache //
///////////////////////

impl<S: StateStore> WriteThroughCache<S> {
    pub fn new(state_store: S) -> Result<Self, S::Error> {
        // Read the latest state from the underlying store to initialize the cache.
        let latest_state = state_store.latest_state()?;

        Ok(Self {
            durable_state_cache: RwLock::new(latest_state),
            state_store,
        })
    }

    fn ratchet_cache(&self, new_state: &TrustedState) {
        let mut durable_state_cache = self.durable_state_cache.write().unwrap();
        let cache_version = durable_state_cache.as_ref().map(|s| s.version());

        if Some(new_state.version()) > cache_version {
            *durable_state_cache = Some(new_state.clone());
        }
    }

    pub fn as_inner(&self) -> &S {
        &self.state_store
    }
}

impl<S: StateStore> StateStore for WriteThroughCache<S> {
    type Error = S::Error;

    fn latest_state(&self) -> Result<Option<TrustedState>, Self::Error> {
        Ok(self.durable_state_cache.read().unwrap().clone())
    }

    fn latest_state_version(&self) -> Result<Option<u64>, Self::Error> {
        // avoids a clone while holding the lock :)
        Ok(self
            .durable_state_cache
            .read()
            .unwrap()
            .as_ref()
            .map(|s| s.version()))
    }

    fn store(&self, new_state: &TrustedState) -> Result<(), Self::Error> {
        // we already have a durable state that's newer than this version; we can
        // exit early here since we don't need to do anything.
        if Some(new_state.version()) <= self.latest_state_version()? {
            return Ok(());
        }

        // store the new state and make it durable. we assume that the state is
        // durable when this returns.
        self.state_store.store(new_state)?;

        // at this point, the state is finally durable, so we can safely update
        // the durable state cache
        self.ratchet_cache(new_state);

        // the client now safely observes `new_state`
        Ok(())
    }
}

///////////////////
// InMemoryStore //
///////////////////

impl InMemoryStateStore {
    pub fn new() -> Self {
        Self(Arc::new(WriteThroughCache::new(NoopStateStore).unwrap()))
    }
}

impl StateStore for InMemoryStateStore {
    type Error = Infallible;

    fn latest_state(&self) -> Result<Option<TrustedState>, Self::Error> {
        self.0.latest_state()
    }
    fn latest_state_version(&self) -> Result<Option<u64>, Self::Error> {
        self.0.latest_state_version()
    }
    fn store(&self, new_state: &TrustedState) -> Result<(), Self::Error> {
        self.0.store(new_state)
    }
}

impl StateStore for NoopStateStore {
    type Error = Infallible;

    fn latest_state(&self) -> Result<Option<TrustedState>, Self::Error> {
        Ok(None)
    }
    fn store(&self, _new_state: &TrustedState) -> Result<(), Self::Error> {
        Ok(())
    }
}
