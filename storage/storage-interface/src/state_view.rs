// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::DbReader;
use anyhow::{format_err, Result};
use diem_crypto::{hash::SPARSE_MERKLE_PLACEHOLDER_HASH, HashValue};
use diem_state_view::{StateView, StateViewId};
use diem_types::{
    access_path::AccessPath,
    account_address::{AccountAddress, HashAccountAddress},
    account_state::AccountState,
    account_state_blob::AccountStateBlob,
    proof::{definition::LeafCount, SparseMerkleProof},
    transaction::{Version, PRE_GENESIS_VERSION},
    write_set::WriteSet,
};
use parking_lot::RwLock;
use scratchpad::{AccountStatus, FrozenSparseMerkleTree, SparseMerkleTree};
use std::{
    collections::{hash_map::Entry, HashMap, HashSet},
    convert::TryInto,
    sync::Arc,
};

// TODO(aldenhu): doc
#[derive(Clone, Debug)]
pub struct InMemoryState {
    pub checkpoint: SparseMerkleTree<AccountStateBlob>,
    pub checkpoint_num_transactions: LeafCount,
    pub current: SparseMerkleTree<AccountStateBlob>,
    ///
    /// n.b. Keep only the addresses to avoid heavy mem copying when cloned.
    ///      (The values themselves are heap managed on the SMT)
    pub updated_since_checkpoint: HashSet<AccountAddress>,
}

impl InMemoryState {
    pub fn new(
        checkpoint: SparseMerkleTree<AccountStateBlob>,
        checkpoint_num_transactions: LeafCount,
        current: SparseMerkleTree<AccountStateBlob>,
        updated_since_checkpoint: HashSet<AccountAddress>,
    ) -> Self {
        Self {
            checkpoint,
            checkpoint_num_transactions,
            current,
            updated_since_checkpoint,
        }
    }

    pub fn new_empty() -> Self {
        let smt = SparseMerkleTree::new_empty();
        Self::new(smt.clone(), 0, smt, HashSet::new())
    }

    pub fn new_at_checkpoint(root_hash: HashValue, checkpoint_num_transactions: LeafCount) -> Self {
        let smt = SparseMerkleTree::new(root_hash);
        Self::new(
            smt.clone(),
            checkpoint_num_transactions,
            smt,
            HashSet::new(),
        )
    }

    pub fn checkpoint_version(&self) -> Option<Version> {
        self.checkpoint_num_transactions.checked_sub(1)
    }

    pub fn checkpoint_root_hash(&self) -> HashValue {
        self.checkpoint.root_hash()
    }

    pub fn root_hash(&self) -> HashValue {
        self.current.root_hash()
    }
}

/// `VerifiedStateView` is like a snapshot of the global state comprised of state view at two
/// levels, persistent storage and memory.
pub struct VerifiedStateView {
    /// For logging and debugging purpose, identifies what this view is for.
    id: StateViewId,

    /// A gateway implementing persistent storage interface, which can be a RPC client or direct
    /// accessor.
    reader: Arc<dyn DbReader>,

    /// The most recent state checkpoint version in persistent storage.
    latest_persistent_version: Option<Version>,

    /// The most recent state checkpoint root hash in persistent storage.
    latest_persistent_state_root: HashValue,

    /// Speculative state.
    speculative_state: FrozenSparseMerkleTree<AccountStateBlob>,

    /// The cache of verified account states from `reader` and `speculative_state_view`,
    /// represented by a hashmap with an account address as key and a pair of an ordered
    /// account state map and an an optional account state proof as value. When the VM queries an
    /// `access_path`, this cache will first check whether `reader_cache` is hit. If hit, it
    /// will return the corresponding value of that `access_path`; otherwise, the account state
    /// will be loaded into the cache from scratchpad or persistent storage in order as a
    /// deserialized ordered map and then be returned. If the VM queries this account again,
    /// the cached data can be read directly without bothering storage layer. The proofs in
    /// cache are needed by ScratchPad after VM execution to construct an in-memory sparse Merkle
    /// tree.
    /// ```text
    ///                      +----------------------------+
    ///                      | In-memory SparseMerkleTree <------+
    ///                      +-------------^--------------+      |
    ///                                    |                     |
    ///                                write sets                |
    ///                                    |          cached account state map
    ///                            +-------+-------+           proof
    ///                            |      V M      |             |
    ///                            +-------^-------+             |
    ///                                    |                     |
    ///                      value of `account_address/path`     |
    ///                                    |                     |
    ///        +---------------------------+---------------------+-------+
    ///        | +-------------------------+---------------------+-----+ |
    ///        | |    account_to_state_cache, account_to_proof_cache   | |
    ///        | +---------------^---------------------------^---------+ |
    ///        |                 |                           |           |
    ///        |     account state blob only        account state blob   |
    ///        |                 |                         proof         |
    ///        |                 |                           |           |
    ///        | +---------------+--------------+ +----------+---------+ |
    ///        | |      speculative_state       | |       reader       | |
    ///        | +------------------------------+ +--------------------+ |
    ///        +---------------------------------------------------------+
    /// ```
    account_to_state_cache: RwLock<HashMap<AccountAddress, AccountState>>,
    account_to_proof_cache: RwLock<HashMap<HashValue, SparseMerkleProof<AccountStateBlob>>>,
}

impl VerifiedStateView {
    /// Constructs a [`VerifiedStateView`] with persistent state view represented by
    /// `latest_persistent_state_root` plus a storage reader, and the in-memory speculative state
    /// on top of it represented by `speculative_state`.
    pub fn new(
        id: StateViewId,
        reader: Arc<dyn DbReader>,
        latest_persistent_version: Option<Version>,
        latest_persistent_state_root: HashValue,
        speculative_state: SparseMerkleTree<AccountStateBlob>,
    ) -> Self {
        // Hack: When there's no transaction in the db but state tree root hash is not the
        // placeholder hash, it implies that there's pre-genesis state present.
        let latest_persistent_version = latest_persistent_version.or_else(|| {
            if latest_persistent_state_root != *SPARSE_MERKLE_PLACEHOLDER_HASH {
                Some(PRE_GENESIS_VERSION)
            } else {
                None
            }
        });

        Self {
            id,
            reader,
            latest_persistent_version,
            latest_persistent_state_root,
            speculative_state: speculative_state.freeze(),
            account_to_state_cache: RwLock::new(HashMap::new()),
            account_to_proof_cache: RwLock::new(HashMap::new()),
        }
    }

    pub fn prime_cache(&self, accounts: HashSet<AccountAddress>) -> Result<()> {
        // TODO(aldenhu): add concurrency
        accounts
            .iter()
            .try_for_each(|addr| self.get(&AccessPath::new(*addr, Vec::new())).map(|_| ()))
    }

    pub fn prime_cache_by_write_set(&self, write_sets: &[&WriteSet]) -> Result<()> {
        // collect all accounts touched and dedup
        let accounts: HashSet<_> = write_sets
            .iter()
            .map(|write_set| write_set.iter())
            .flatten()
            .map(|(path, _)| path.address)
            .collect();
        self.prime_cache(accounts)
    }

    pub fn into_state_cache(self) -> StateCache {
        StateCache {
            _frozen_base: self.speculative_state,
            accounts: self.account_to_state_cache.into_inner(),
            proofs: self.account_to_proof_cache.into_inner(),
        }
    }
}

pub struct StateCache {
    pub _frozen_base: FrozenSparseMerkleTree<AccountStateBlob>,
    pub accounts: HashMap<AccountAddress, AccountState>,
    pub proofs: HashMap<HashValue, SparseMerkleProof<AccountStateBlob>>,
}

impl StateView for VerifiedStateView {
    fn id(&self) -> StateViewId {
        self.id
    }

    fn get(&self, access_path: &AccessPath) -> Result<Option<Vec<u8>>> {
        let address = access_path.address;
        let path = &access_path.path;

        // Lock for read first:
        if let Some(contents) = self.account_to_state_cache.read().get(&address) {
            return Ok(contents.get(path).cloned());
        }

        // Do most of the work outside the write lock.
        let account_blob_option = match self.speculative_state.get(address.hash()) {
            AccountStatus::ExistsInScratchPad(blob) => Some(blob),
            AccountStatus::DoesNotExist => None,
            // No matter it is in db or unknown, we have to query from db since even the
            // former case, we don't have the blob data but only its hash.
            AccountStatus::ExistsInDB | AccountStatus::Unknown => {
                let (blob, proof) = match self.latest_persistent_version {
                    Some(version) => self
                        .reader
                        .get_account_state_with_proof_by_version(address, version)?,
                    None => (None, SparseMerkleProof::new(None, vec![])),
                };
                proof
                    .verify(
                        self.latest_persistent_state_root,
                        address.hash(),
                        blob.as_ref(),
                    )
                    .map_err(|err| {
                        format_err!(
                            "Proof is invalid for address {:?} with state root hash {:?}: {}",
                            address,
                            self.latest_persistent_state_root,
                            err
                        )
                    })?;

                // multiple threads may enter this code, and another thread might add
                // an address before this one. Thus the insertion might return a None here.
                self.account_to_proof_cache
                    .write()
                    .insert(address.hash(), proof);

                blob
            }
        };

        // Now enter the locked region, and write if still empty.
        let new_account_blob = account_blob_option
            .as_ref()
            .map(TryInto::try_into)
            .transpose()?
            .unwrap_or_default();

        match self.account_to_state_cache.write().entry(address) {
            Entry::Occupied(occupied) => Ok(occupied.get().get(path).cloned()),
            Entry::Vacant(vacant) => Ok(vacant.insert(new_account_blob).get(path).cloned()),
        }
    }

    fn is_genesis(&self) -> bool {
        self.latest_persistent_version.is_none()
    }
}
