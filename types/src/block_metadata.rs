use crate::account_address::AccountAddress;
use crypto::{ed25519::Ed25519Signature, HashValue};
use std::collections::BTreeMap;

/// Struct that will be persisted on chain to store the information of the current block.
pub struct BlockMetaData {
    id: HashValue,
    timestamp_usec: u64,
    previous_block_votes: BTreeMap<AccountAddress, Ed25519Signature>,
    proposer: AccountAddress,
}

impl BlockMetaData {
    pub fn new(
        id: HashValue,
        timestamp_usec: u64,
        previous_block_votes: BTreeMap<AccountAddress, Ed25519Signature>,
        proposer: AccountAddress,
    ) -> Self {
        Self {
            id,
            timestamp_usec,
            previous_block_votes,
            proposer,
        }
    }

    pub fn into_inner(
        self,
    ) -> (
        HashValue,
        u64,
        BTreeMap<AccountAddress, Ed25519Signature>,
        AccountAddress,
    ) {
        (
            self.id,
            self.timestamp_usec,
            self.previous_block_votes,
            self.proposer,
        )
    }
}
