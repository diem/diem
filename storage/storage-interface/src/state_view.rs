// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::DbReader;
use anyhow::{format_err, Result};
use libra_crypto::{
    hash::{CryptoHash, SPARSE_MERKLE_PLACEHOLDER_HASH},
    HashValue,
};
use libra_state_view::{StateView, StateViewId};
use libra_types::{
    access_path::AccessPath,
    account_address::AccountAddress,
    account_state::AccountState,
    proof::SparseMerkleProof,
    transaction::{Version, PRE_GENESIS_VERSION},
};
use scratchpad::{AccountStatus, SparseMerkleTree};
use std::{
    cell::RefCell,
    collections::{hash_map::Entry, HashMap},
    convert::TryInto,
    sync::Arc,
};

/// `VerifiedStateView` is like a snapshot of the global state comprised of state view at two
/// levels, persistent storage and memory.
pub struct VerifiedStateView<'a> {
    /// For logging and debugging purpose, identifies what this view is for.
    id: StateViewId,

    /// A gateway implementing persistent storage interface, which can be a RPC client or direct
    /// accessor.
    reader: Arc<dyn DbReader>,

    /// The most recent version in persistent storage.
    latest_persistent_version: Option<Version>,

    /// The most recent state root hash in persistent storage.
    latest_persistent_state_root: HashValue,

    /// The in-momery version of sparse Merkle tree of which the states haven't been committed.
    speculative_state: &'a SparseMerkleTree,

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
    account_to_state_cache: RefCell<HashMap<AccountAddress, AccountState>>,
    account_to_proof_cache: RefCell<HashMap<HashValue, SparseMerkleProof>>,
}

impl<'a> VerifiedStateView<'a> {
    /// Constructs a [`VerifiedStateView`] with persistent state view represented by
    /// `latest_persistent_state_root` plus a storage reader, and the in-memory speculative state
    /// on top of it represented by `speculative_state`.
    pub fn new(
        id: StateViewId,
        reader: Arc<dyn DbReader>,
        latest_persistent_version: Option<Version>,
        latest_persistent_state_root: HashValue,
        speculative_state: &'a SparseMerkleTree,
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
            speculative_state,
            account_to_state_cache: RefCell::new(HashMap::new()),
            account_to_proof_cache: RefCell::new(HashMap::new()),
        }
    }
}

impl<'a>
    Into<(
        HashMap<AccountAddress, AccountState>,
        HashMap<HashValue, SparseMerkleProof>,
    )> for VerifiedStateView<'a>
{
    fn into(
        self,
    ) -> (
        HashMap<AccountAddress, AccountState>,
        HashMap<HashValue, SparseMerkleProof>,
    ) {
        (
            self.account_to_state_cache.into_inner(),
            self.account_to_proof_cache.into_inner(),
        )
    }
}

impl<'a> StateView for VerifiedStateView<'a> {
    fn id(&self) -> StateViewId {
        self.id
    }

    fn get(&self, access_path: &AccessPath) -> Result<Option<Vec<u8>>> {
        let address = access_path.address;
        let path = &access_path.path;
        match self.account_to_state_cache.borrow_mut().entry(address) {
            Entry::Occupied(occupied) => Ok(occupied.get().get(path).cloned()),
            Entry::Vacant(vacant) => {
                let address_hash = address.hash();
                let account_blob_option = match self.speculative_state.get(address_hash) {
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
                        assert!(self
                            .account_to_proof_cache
                            .borrow_mut()
                            .insert(address_hash, proof)
                            .is_none());
                        blob
                    }
                };
                Ok(vacant
                    .insert(
                        account_blob_option
                            .as_ref()
                            .map(TryInto::try_into)
                            .transpose()?
                            .unwrap_or_default(),
                    )
                    .get(path)
                    .cloned())
            }
        }
    }

    fn multi_get(&self, _access_paths: &[AccessPath]) -> Result<Vec<Option<Vec<u8>>>> {
        unimplemented!();
    }

    fn is_genesis(&self) -> bool {
        self.latest_persistent_version.is_none()
    }
}
