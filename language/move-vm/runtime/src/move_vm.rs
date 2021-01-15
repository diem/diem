// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{data_cache::RemoteCache, runtime::VMRuntime, session::Session};

pub struct MoveVM {
    runtime: VMRuntime,
}

impl MoveVM {
    pub fn new() -> Self {
        Self {
            runtime: VMRuntime::new(),
        }
    }

    /// Create a new Session backed by the given storage.
    ///
    /// Right now it is the caller's responsibility to ensure cache coherence of the Move VM Loader
    ///   - When a new module gets published in a Move VM Session, and then gets used by another
    ///     transaction, it will be loaded into the code cache and stay there even if the resulted
    ///     effects do not get committed back to the storage when the Session ends.
    ///   - However, if a module is updated (e.g., re-published), the loader cache will NOT be
    ///     updated and the cache still contains the old module! Executing scripts or functions
    ///     after the module re-publishing will lead to behaviors in the old module!
    ///   - As a result, if one wants to have multiple sessions at a time, one needs to make sure
    ///     none of them will try to publish a new module. In other words, if there is a module
    ///     publishing Session it must be the only Session existing.
    ///   - In general, a new Move VM needs to be created whenever the storage gets modified by an
    ///     outer environment, or otherwise the states may be out of sync. There are a few exceptional
    ///     cases where this may not be necessary, with the most notable one being the common module
    ///     publishing flow: you can keep using the same Move VM if you publish some modules in a Session
    ///     and apply the effects to the storage when the Session ends.
    ///
    /// Given this is an error-prone process, the Move VM will try to enforce the following policy:
    ///   - It is OK to publish as many *new* modules as proper in one Session or across Sessions
    ///     created from the same VM instance.
    ///   - But only one module update is allowed across the whole Move VM instance, regardless of
    ///     how many Sessions this Move VM may create in its life time.
    ///   - If a VM Session processes a module update already, the VM (i.e., any Session created
    ///     by the VM instance) should stop processing further transaction. Here a transaction can
    ///     be either module publishing or script / function execution. Any transactions sent in
    ///     after a module is re-published will be rejected with a CODE_CACHE_EXPIRED error. The
    ///     only exception is direct invocation of Diem framework functions (e.g., txn epilogue).
    pub fn new_session<'r, R: RemoteCache>(&self, remote: &'r R) -> Session<'r, '_, R> {
        self.runtime.new_session(remote)
    }
}
