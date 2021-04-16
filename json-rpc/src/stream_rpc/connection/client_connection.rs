// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    errors::JsonRpcError,
    stream_rpc::{
        connection::{ConnectionContext, StreamSender, Task},
        counters,
        json_rpc::{JsonRpcRequest, JsonRpcResponse, Method},
        logging,
        subscriptions::SubscriptionConfig,
    },
};
use diem_infallible::Mutex;
use diem_json_rpc_types::Id;
use diem_logger::debug;
use std::{collections::HashMap, str::FromStr, sync::Arc};
use storage_interface::DbReader;

#[derive(Debug)]
pub struct RunningTasks {
    pub tasks: Mutex<HashMap<Id, Task>>,
    pub client_id: u64,
}

impl RunningTasks {
    pub fn new(client_id: u64) -> Self {
        Self {
            tasks: Mutex::new(HashMap::new()),
            client_id,
        }
    }

    pub fn stop(&self, id: &Id) -> Option<()> {
        let mut lock = self.tasks.lock();
        if let Some(task) = lock.get(id) {
            task.abort();
            lock.remove(&id);
            Some(())
        } else {
            None
        }
    }

    pub fn stop_all(&self) {
        let mut tasks = self.tasks.lock();
        debug!(
            "Aborting {} tasks for Client#{}",
            tasks.len(),
            &self.client_id
        );
        tasks.iter().for_each(|(_id, task)| task.abort());
        tasks.clear();
    }
}

// This will get cloned for every subscription, so lets keep it light :-)
#[derive(Debug, Clone)]
pub struct ClientConnection {
    pub id: u64,
    pub sender: StreamSender,
    pub running_tasks: Arc<RunningTasks>,
    pub connection_context: Arc<ConnectionContext>,
    pub config: Arc<SubscriptionConfig>,
}

impl ClientConnection {
    pub fn new(
        id: u64,
        sender: StreamSender,
        connection_context: ConnectionContext,
        config: Arc<SubscriptionConfig>,
    ) -> Self {
        Self {
            id,
            sender,
            running_tasks: Arc::new(RunningTasks::new(id)),
            connection_context: Arc::new(connection_context),
            config,
        }
    }

    pub fn disconnect(&self) {
        self.running_tasks.stop_all();
    }

    pub async fn send_raw(&self, message: String) -> anyhow::Result<()> {
        if self.is_closed() {
            anyhow::bail!("Client#{} is closed", self.id);
        }
        match self.sender.send(Ok(message)).await {
            Ok(_) => Ok(()),
            Err(e) => match e.0 {
                Ok(_) => Ok(()),
                Err(e) => Err(e),
            },
        }
    }

    pub async fn send_response(&self, message: JsonRpcResponse) -> anyhow::Result<()> {
        if let Ok(response) = serde_json::to_string(&message) {
            return self.send_raw(response).await;
        }
        anyhow::bail!("Could not convert {:?} to string", message)
    }

    pub async fn send_success<T: serde::Serialize>(
        &self,
        id: Id,
        message: &T,
    ) -> anyhow::Result<()> {
        let message = serde_json::to_value(message).unwrap();
        self.send_response(JsonRpcResponse::result(Some(id), Some(message)))
            .await
    }

    pub async fn send_error(
        &self,
        id: Option<Id>,
        error: Option<JsonRpcError>,
    ) -> anyhow::Result<()> {
        self.send_response(JsonRpcResponse::error(id, error)).await
    }

    pub async fn send_error_with_metrics(
        &self,
        method: Option<Method>,
        id: Option<Id>,
        error: Option<JsonRpcError>,
    ) -> anyhow::Result<()> {
        counters::INVALID_REQUESTS
            .with_label_values(&[
                self.connection_context.transport.as_str(),
                method.map_or("unknown", |m| m.as_str()),
                error.as_ref().map_or("unknown", |e| e.code_as_str()),
                self.connection_context.sdk_info.language.as_str(),
                &self.connection_context.sdk_info.version.to_string(),
            ])
            .inc();
        self.send_error(id, error).await
    }

    pub fn unsubscribe(&self, id: &Id) -> Option<()> {
        self.running_tasks.stop(id)
    }

    pub fn is_closed(&self) -> bool {
        self.sender.is_closed()
    }

    pub async fn received_message(&self, db: Arc<dyn DbReader>, message: String) {
        match JsonRpcRequest::from_str(&message) {
            Ok(mut request) => {
                debug!(
                    logging::SubscriptionRequestLog {
                        transport: self.connection_context.transport.as_str(),
                        remote_addr: self.connection_context.remote_addr.as_deref(),
                        client_id: self.id,
                        rpc_method: Some(request.method_name()),
                    },
                    "subscription request"
                );
                if let Err(err) = self.handle_rpc_request(db, &mut request) {
                    self.send_error_with_metrics(
                        Some(request.method_request.method()),
                        Some(request.id),
                        Some(err),
                    )
                    .await
                    .ok();
                }
            }
            Err((err, method, id)) => {
                // We couldn't parse the request- it's not valid json or an unknown structure
                debug!(
                    logging::SubscriptionRequestLog {
                        transport: self.connection_context.transport.as_str(),
                        remote_addr: self.connection_context.remote_addr.as_deref(),
                        client_id: self.id,
                        rpc_method: method.map(|v| v.as_str()),
                    },
                    "failed to parse subscription request ({})", &err
                );
                metric_subscription_rpc_received(
                    &self,
                    method.map_or("unknown", |m| m.as_str()),
                    counters::RpcResult::Error,
                );
                self.send_error_with_metrics(method, id, Some(err))
                    .await
                    .ok();
            }
        }
    }

    /// - The `tasks` lock is within the scope of one client (subscribing, unsubscribing, disconnecting, or some combination therein)
    /// - The `call_method` is responsible for doing validation on the parameters, and returning a `Result<JoinHandle<()>>` (tokio task) if
    ///     a subscription task was spawned
    /// - The lock must be held until we can determine whether or not we have a subscription because otherwise there is a race condition:
    ///     if a user submits the same rpc id (`RequestIdentifier`) again after we’ve verified it’s not used, but before we insert it,
    ///     which would result in losing track of that subscription task (and leaking green threads)
    ///
    /// When calling `request.method_request.call_method`, communication to the client and
    /// subscription behavior is determined by the `Result<JoinHandle<()>, JsonRpcError>` returned.
    ///
    /// 1. Returning `Err(JsonRpcRequest)` is the way to handle an issue with a parameter value, or
    ///     any other such case where a subscription may not be started or requested data may not be
    ///     returned.
    /// 2. Returning `Ok(JoinHandle<()>)` indicates that the subscription has been successfully created.
    fn handle_rpc_request(
        &self,
        db: Arc<dyn DbReader>,
        request: &mut JsonRpcRequest,
    ) -> Result<(), JsonRpcError> {
        let mut tasks = self.running_tasks.tasks.lock();
        if tasks.contains_key(&request.id) {
            debug!(
                "Client#{} already has a subscription for '{}'",
                self.id, &request.id
            );
            let err = JsonRpcError::invalid_request_with_msg(format!(
                "Subscription for '{}' already exists",
                &request.id
            ));

            return Err(err);
        }

        match request
            .method_request
            .call_method(db.clone(), self.clone(), request.id.clone())
        {
            Ok(task) => {
                tasks.insert(request.id.clone(), task);
                metric_subscription_rpc_received(
                    &self,
                    request.method_name(),
                    counters::RpcResult::Success,
                );
                Ok(())
            }
            Err(err) => {
                // This error comes from within the subscription itself, before the task is started: it's most likely parameter validation.
                metric_subscription_rpc_received(
                    &self,
                    request.method_name(),
                    counters::RpcResult::Error,
                );
                Err(err)
            }
        }
    }
}

fn metric_subscription_rpc_received(
    client: &ClientConnection,
    method: &str,
    result: counters::RpcResult,
) {
    counters::SUBSCRIPTION_RPC_RECEIVED
        .with_label_values(&[
            client.connection_context.transport.as_str(),
            method,
            result.as_str(),
            client.connection_context.sdk_info.language.as_str(),
            &client.connection_context.sdk_info.version.to_string(),
        ])
        .inc();
}
