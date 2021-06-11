// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! # Subscription Handlers
//!
//! ## Implementing the Subscription Trait
//! In order to turn a subscription parameter struct into something which is capable of launching
//! a subscription, you must implement the `Subscription<ParamType>` trait.
//! To do so, there are three methods you must define:
//! 1. `init(&mut self, helper: &SubscriptionHelper) -> Result<(), JsonRpcError>;`
//!     - This is where initial state is set up to keep track of what has been sent to the client.
//!       This could be any incremental id: a sequence number for events, the transaction version for transactions, etc.
//!     - In order to store these variables, fields can be added to the struct:
//!         1. Whose type implements the `Default` trait and has a `::Default` method
//!         2. Are marked with a `#[serde(skip)]`
//!     - Returning `Err(JsonRpcRequest)` is the way to handle an issue with a parameter value, or
//!       any other such case where a subscription may not be started or requested data may not be
//!       returned. Otherwise if the parameters are valid and a subscription could be created, simply
//!       return `Ok(())`
//!
//! 2. `next(&self, helper: &SubscriptionHelper) -> Vec<ParamType>;`
//!     Within this method, use existing state set up in the `init` function to fetch the next batch of data.
//!     The size of the batch should be determined by `helper.client.config.fetch_size`.
//!     If the function returns an array of data, each item will be serialized one by one and sent to the client.
//!     The `on_send` method (below) will be called for each item in the array.
//!     If the function returns an empty array, the thread will sleep for `client.config.poll_interval_ms`.
//!
//! 3. `on_send(&mut self, item: &ParamType);`
//!     If the `next` function returns an array of items, after each the item is serialized and sent to the client
//!     it will be passed to this `on_send` method. This is the perfect place to update state fields
//!     which keep track of what has/hasn't been sent to the client.
//!

use crate::{
    data::{get_events, get_transactions},
    errors::JsonRpcError,
    stream_rpc::{
        connection::ClientConnection,
        counters,
        json_rpc::{
            StreamMethod, SubscribeResult, SubscribeToEventsParams, SubscribeToTransactionsParams,
        },
    },
    views::{EventView, TransactionView},
};
use diem_json_rpc_types::Id;
use diem_logger::{debug, warn};
use serde::Serialize;
use std::{borrow::Borrow, iter::Map, sync::Arc, time::Duration};
use storage_interface::DbReader;
use tokio::task::JoinHandle;
use tokio_retry::strategy::{jitter, ExponentialBackoff};

#[derive(Clone, Debug)]
pub struct SubscriptionConfig {
    pub fetch_size: u64,
    pub poll_interval_ms: u64,
    pub queue_size: usize,
}

type JitterBackoff = Map<ExponentialBackoff, fn(Duration) -> Duration>;

pub fn create_backoff(poll_interval_ms: u64) -> JitterBackoff {
    ExponentialBackoff::from_millis(poll_interval_ms)
        .factor(1000)
        .max_delay(tokio::time::Duration::from_millis(poll_interval_ms * 10))
        .map(jitter)
}

#[derive(Clone)]
pub struct SubscriptionHelper {
    pub db: Arc<dyn DbReader>,
    pub client: ClientConnection,
    pub jsonrpc_id: Id,
    pub method: StreamMethod,
    backoff: JitterBackoff,
}

impl SubscriptionHelper {
    pub fn new(
        db: Arc<dyn DbReader>,
        client: ClientConnection,
        jsonrpc_id: Id,
        method: StreamMethod,
    ) -> Self {
        let poll_interval_ms = client.config.poll_interval_ms;
        Self {
            db,
            client,
            jsonrpc_id,
            method,
            backoff: create_backoff(poll_interval_ms),
        }
    }
    /// This is to be one of the first things that a new subscription task must do, to let the client
    /// know that the subscription has been created (some streams may be very sparse, or start with 0 items)
    pub async fn send_ok(&self) {
        counters::SUBSCRIPTION_OKS
            .with_label_values(&[
                self.client.connection_context.transport.as_str(),
                &self.method.as_str(),
                counters::RpcResult::Success.as_str(),
                self.client.connection_context.sdk_info.language.as_str(),
                &self.client.connection_context.sdk_info.version.to_string(),
            ])
            .inc();
        self.client
            .send_success(
                self.jsonrpc_id.clone(),
                &SubscribeResult::ok(self.db.get_latest_version().unwrap_or(0)),
            )
            .await
            .ok();
    }

    pub fn reset_backoff(&mut self) {
        self.backoff = create_backoff(self.client.config.poll_interval_ms)
    }

    pub async fn sleep_wiggled(&mut self) {
        let sleepy_time = self.backoff.next().unwrap();
        debug!(
            "Client#{} Sleeping for {:?}ms as no results for {}",
            self.client.id,
            sleepy_time,
            self.method.as_str()
        );
        tokio::time::sleep(sleepy_time).await;
    }
}

pub trait Subscription<ParamType>: Send + Sync + Clone + 'static
where
    ParamType: Send + Sync + Serialize + Clone,
{
    fn init(&mut self, helper: &SubscriptionHelper) -> Result<(), JsonRpcError>;
    fn next(&self, helper: &SubscriptionHelper) -> Vec<ParamType>;
    fn on_send(&mut self, item: &ParamType);

    fn run(mut self, mut helper: SubscriptionHelper) -> Result<JoinHandle<()>, JsonRpcError> {
        self.init(&helper)?;

        Ok(tokio::spawn(async move {
            helper.send_ok().await;

            loop {
                let items = self.next(&helper);

                debug!(
                    "Client#{}: fetched {} items for {}",
                    helper.client.id,
                    items.len(),
                    helper.method.as_str()
                );

                if items.is_empty() {
                    helper.sleep_wiggled().await;
                    continue;
                }

                helper.reset_backoff();

                counters::SUBSCRIPTION_RPC_SENT
                    .with_label_values(&[
                        helper.client.connection_context.transport.as_str(),
                        helper.method.as_str(),
                        counters::RpcResult::Success.as_str(),
                        helper.client.connection_context.sdk_info.language.as_str(),
                        &helper
                            .client
                            .connection_context
                            .sdk_info
                            .version
                            .to_string(),
                    ])
                    .inc_by(items.len() as u64);

                for item in items {
                    match helper
                        .client
                        .send_success(helper.jsonrpc_id.clone(), &item)
                        .await
                    {
                        Err(e) => {
                            // client has disconnected
                            debug!("Client#{}: Send error: {:?}", &helper.client.id, e);
                            return;
                        }
                        Ok(_) => self.on_send(&item),
                    };
                }
            }
        }))
    }
}

impl Subscription<TransactionView> for SubscribeToTransactionsParams {
    fn init(&mut self, _helper: &SubscriptionHelper) -> Result<(), JsonRpcError> {
        self.latest_version = self.starting_version;
        Ok(())
    }

    fn next(&self, helper: &SubscriptionHelper) -> Vec<TransactionView> {
        match get_transactions(
            helper.db.borrow(),
            helper.db.get_latest_version().unwrap_or(0),
            self.latest_version,
            helper.client.config.fetch_size,
            self.include_events.unwrap_or(false),
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

    fn on_send(&mut self, _tx: &TransactionView) {
        self.latest_version += 1;
    }
}

impl Subscription<EventView> for SubscribeToEventsParams {
    fn init(&mut self, _helper: &SubscriptionHelper) -> Result<(), JsonRpcError> {
        self.latest_event = self.event_seq_num;
        Ok(())
    }

    fn next(&self, helper: &SubscriptionHelper) -> Vec<EventView> {
        match get_events(
            helper.db.borrow(),
            helper.db.get_latest_version().unwrap_or(0),
            self.event_key,
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

    fn on_send(&mut self, event: &EventView) {
        self.latest_event = event.sequence_number + 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        stream_rpc::{
            connection::ConnectionContext, errors::StreamError, transport::util::Transport,
        },
        tests::utils::mock_db,
    };
    use tokio::sync::mpsc;

    #[tokio::test]
    async fn test_subscription() {
        let mock_db = mock_db();

        let (sender, mut receiver) = mpsc::channel::<Result<String, StreamError>>(1);

        let config = SubscriptionConfig {
            fetch_size: 1,
            poll_interval_ms: 100,
            queue_size: 1,
        };

        let connection_context = ConnectionContext {
            transport: Transport::Websocket,
            sdk_info: Default::default(),
            remote_addr: None,
        };
        let client_connection =
            ClientConnection::new(1337, sender, connection_context, Arc::new(config));

        let subscription_helper = SubscriptionHelper::new(
            Arc::new(mock_db.clone()),
            client_connection,
            Id::Number(1010101),
            StreamMethod::SubscribeToTransactions,
        );

        let params = SubscribeToTransactionsParams {
            starting_version: 0,
            include_events: Some(true),
            latest_version: 0,
        };

        let handle = params.run(subscription_helper).unwrap();
        let ok_msg = receiver.recv().await;

        let result = ok_msg.unwrap().unwrap();
        let expected = serde_json::json!({"jsonrpc": "2.0", "id": 1010101, "result": {"status": "OK", "transaction_version": mock_db.version}}).to_string();
        assert_eq!(expected, result);
        handle.abort();
    }
}
