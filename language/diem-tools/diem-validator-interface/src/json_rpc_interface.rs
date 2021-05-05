// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::DiemValidatorInterface;
use anyhow::Result;
use diem_client::BlockingClient;
use diem_types::{
    account_address::AccountAddress,
    account_state::AccountState,
    account_state_blob::AccountStateBlob,
    contract_event::EventWithProof,
    event::EventKey,
    transaction::{Transaction, Version},
};
use std::convert::TryFrom;

pub struct JsonRpcDebuggerInterface {
    client: BlockingClient,
}

impl JsonRpcDebuggerInterface {
    pub fn new(url: &str) -> Result<Self> {
        Ok(Self {
            client: BlockingClient::new(url),
        })
    }
}

impl DiemValidatorInterface for JsonRpcDebuggerInterface {
    fn get_account_state_by_version(
        &self,
        account: AccountAddress,
        version: Version,
    ) -> Result<Option<AccountState>> {
        let account_state = self
            .client
            .get_account_state_with_proof(account, Some(version), None)?
            .into_inner();

        Ok(match account_state.blob {
            Some(bytes) => {
                let account_state_blob = bcs::from_bytes::<AccountStateBlob>(&bytes)?;
                Some(AccountState::try_from(&account_state_blob)?)
            }
            None => None,
        })
    }

    fn get_events(
        &self,
        key: &EventKey,
        start_seq: u64,
        limit: u64,
    ) -> Result<Vec<EventWithProof>> {
        let events = self
            .client
            .get_events_with_proofs(*key, start_seq, limit)?
            .into_inner();
        let mut result = vec![];
        for event in events {
            result.push(bcs::from_bytes(event.event_with_proof.inner())?);
        }
        Ok(result)
    }

    fn get_committed_transactions(&self, start: Version, limit: u64) -> Result<Vec<Transaction>> {
        let txns = self
            .client
            .get_transactions(start, limit, false)?
            .into_inner();

        let mut output = vec![];
        for txn in txns.into_iter() {
            let raw_bytes = txn.bytes;
            output.push(bcs::from_bytes(&raw_bytes)?);
        }
        Ok(output)
    }

    fn get_latest_version(&self) -> Result<Version> {
        let metadata = self.client.get_metadata()?.into_inner();

        Ok(metadata.version)
    }

    fn get_version_by_account_sequence(
        &self,
        account: AccountAddress,
        seq: u64,
    ) -> Result<Option<Version>> {
        let txn = self
            .client
            .get_account_transaction(account, seq, false)?
            .into_inner();

        Ok(txn.map(|txn| txn.version))
    }
}
