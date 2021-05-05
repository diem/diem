// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use diem_client::{views::EventView, Client};
use diem_types::{
    account_address::AccountAddress,
    account_state::AccountState,
    account_state_blob::AccountStateBlob,
    event::{EventHandle, EventKey},
};
use futures::future::join_all;
use std::convert::TryFrom;

const BATCH_SIZE: u64 = 500;

pub struct DiemEventsFetcher(Client);

impl DiemEventsFetcher {
    pub fn new(url: &str) -> Result<Self> {
        Ok(Self(Client::new(url)))
    }

    async fn get_account_state(&self, account: AccountAddress) -> Result<Option<AccountState>> {
        let account_state: AccountStateBlob = {
            let blob = self
                .0
                .get_account_state_with_proof(account, None, None)
                .await?
                .into_inner()
                .blob
                .ok_or_else(|| anyhow::anyhow!("missing account state blob"))?;
            bcs::from_bytes(&blob)?
        };
        Ok(Some(AccountState::try_from(&account_state)?))
    }

    pub async fn get_payment_event_handles(
        &self,
        account: AccountAddress,
    ) -> Result<Option<(EventHandle, EventHandle)>> {
        match self.get_account_state(account).await? {
            Some(account_state) => Ok(account_state.get_account_resource()?.map(|resource| {
                (
                    resource.sent_events().clone(),
                    resource.received_events().clone(),
                )
            })),
            None => Ok(None),
        }
    }

    pub async fn get_events(
        &self,
        event_key: &EventKey,
        start: u64,
        limit: u64,
    ) -> Result<Vec<EventView>> {
        let resp = self
            .0
            .get_events(*event_key, start, limit)
            .await?
            .into_inner();

        Ok(resp)
    }

    pub async fn get_all_events(&self, event_handle: &EventHandle) -> Result<Vec<EventView>> {
        if event_handle.count() == 0 {
            return Ok(vec![]);
        }
        let mut futures = vec![];
        let mut i: u64 = 0;
        while i.wrapping_add(BATCH_SIZE) < event_handle.count() {
            futures.push(self.get_events(event_handle.key(), i, BATCH_SIZE));
            i = i.wrapping_add(BATCH_SIZE);
        }
        futures.push(self.get_events(event_handle.key(), i, event_handle.count().wrapping_sub(i)));

        let mut result = vec![];
        for response in join_all(futures.into_iter()).await {
            match response {
                Ok(mut events) => result.append(&mut events),
                Err(e) => return Err(e),
            }
        }
        Ok(result)
    }
}
