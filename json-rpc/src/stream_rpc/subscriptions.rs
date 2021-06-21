// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    data::{get_events, get_transactions},
    errors::JsonRpcError,
    stream_rpc::subscription_types::{Subscription, SubscriptionHelper},
    views::{EventView, TransactionView},
};
use diem_json_rpc_types::stream::request::{
    SubscribeToEventsParams, SubscribeToTransactionsParams,
};
use diem_logger::warn;
use std::borrow::Borrow;

#[derive(Clone, Copy, Debug, Default)]
pub struct TransactionsSubscription {
    pub(crate) latest_version: u64,
}

impl Subscription<SubscribeToTransactionsParams, TransactionView> for TransactionsSubscription {
    fn init(
        &mut self,
        _helper: &SubscriptionHelper,
        params: &SubscribeToTransactionsParams,
    ) -> Result<(), JsonRpcError> {
        self.latest_version = params.starting_version;
        Ok(())
    }

    fn next(
        &self,
        helper: &SubscriptionHelper,
        params: &SubscribeToTransactionsParams,
    ) -> Vec<TransactionView> {
        match get_transactions(
            helper.db.borrow(),
            helper.db.get_latest_version().unwrap_or(0),
            self.latest_version,
            helper.client.config.fetch_size,
            params.include_events.unwrap_or(false),
        ) {
            Ok(transactions) => transactions.0,
            Err(e) => {
                warn!(
                    "Client#{} Could not fetch transactions: {}",
                    helper.client.id, e
                );
                vec![]
            }
        }
    }

    fn on_send(&mut self, tx: Option<&TransactionView>) {
        if tx.is_some() {
            self.latest_version += 1;
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct EventsSubscription {
    pub(crate) latest_event: u64,
}

impl Subscription<SubscribeToEventsParams, EventView> for EventsSubscription {
    fn init(
        &mut self,
        _helper: &SubscriptionHelper,
        params: &SubscribeToEventsParams,
    ) -> Result<(), JsonRpcError> {
        self.latest_event = params.event_seq_num;
        Ok(())
    }

    fn next(
        &self,
        helper: &SubscriptionHelper,
        params: &SubscribeToEventsParams,
    ) -> Vec<EventView> {
        match get_events(
            helper.db.borrow(),
            helper.db.get_latest_version().unwrap_or(0),
            params.event_key,
            self.latest_event,
            helper.client.config.fetch_size,
        ) {
            Ok(events) => events,
            Err(e) => {
                warn!("Client#{} Could not fetch events: {}", helper.client.id, e);
                vec![]
            }
        }
    }

    fn on_send(&mut self, event: Option<&EventView>) {
        if let Some(event) = event {
            self.latest_event = event.sequence_number + 1
        }
    }
}
