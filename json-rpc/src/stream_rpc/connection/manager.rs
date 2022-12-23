// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::stream_rpc::{
    connection::{BoxConnectionStream, ClientConnection, ConnectionContext, StreamSender},
    counters, logging,
    subscription_types::SubscriptionConfig,
};
use diem_infallible::RwLock;
use diem_logger::debug;
use futures::StreamExt;
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};

use storage_interface::MoveDbReader;

#[derive(Clone)]
pub struct ConnectionManager {
    pub clients: Arc<RwLock<HashMap<u64, ClientConnection>>>,
    pub diem_db: Arc<dyn MoveDbReader>,
    pub config: Arc<SubscriptionConfig>,
    /// Our unique user id counter.
    next_user_id: Arc<AtomicU64>,
}

impl ConnectionManager {
    pub fn new(diem_db: Arc<dyn MoveDbReader>, config: Arc<SubscriptionConfig>) -> Self {
        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
            diem_db,
            config,
            next_user_id: Arc::new(AtomicU64::new(0)),
        }
    }

    fn get_db(&self) -> Arc<dyn MoveDbReader> {
        self.diem_db.clone()
    }

    pub fn has_client(&self, client_id: u64) -> bool {
        self.clients.read().contains_key(&client_id)
    }

    #[allow(unused)]
    pub fn get_client(&self, client_id: u64) -> Option<ClientConnection> {
        self.clients.read().get(&client_id).cloned()
    }

    pub fn add_client(&self, client: ClientConnection) {
        self.clients.write().insert(client.id, client);
    }

    pub fn next_user_id(&self) -> u64 {
        // Ensure that if the node is up long enough, and we have enough connections to wrap around, we don't accidentally clobber an existing client
        loop {
            let next_id = self.next_user_id.fetch_add(1, Ordering::Relaxed);
            if !self.has_client(next_id) {
                return next_id;
            }
        }
    }

    /// Create a `ClientConnection`, and then wait for either the sending/receiving connection to
    /// close, before initiating cleanup
    pub async fn client_connection(
        self,
        client_sender: StreamSender,
        mut client_rcv: BoxConnectionStream,
        connection_context: ConnectionContext,
    ) {
        let client_id = self.next_user_id();
        let client = ClientConnection::new(
            client_id,
            client_sender.clone(),
            connection_context,
            self.config.clone(),
        );
        self.add_client(client.clone());

        counters::CLIENT_CONNECTED
            .with_label_values(&[
                client.connection_context.transport.as_str(),
                client.connection_context.sdk_info.language.as_str(),
                &client.connection_context.sdk_info.version.to_string(),
            ])
            .inc();

        debug!(
            logging::StreamRpcLog {
                transport: client.connection_context.transport.as_str(),
                remote_addr: client.connection_context.remote_addr.as_deref(),
                user_agent: None,
                action: logging::StreamRpcAction::ClientConnectionLog(
                    logging::ClientConnectionLog {
                        client_id: Some(client.id),
                        forwarded: None,
                        rpc_method: None,
                    }
                ),
            },
            "client connected"
        );

        // Reap client if we can no longer send to it
        let send_task = tokio::task::spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                if client_sender.is_closed() {
                    break;
                }
            }
        });

        // TODO: reap idle connections without any subscriptions or which haven't accepted a message in a while?
        let task_client = client.clone();
        let task_db = self.get_db();
        let recv_task = tokio::task::spawn(async move {
            while let Some(result) = client_rcv.next().await {
                match result {
                    Ok(msg) => {
                        if let Some(message) = msg {
                            task_client.received_message(task_db.clone(), message).await;
                        }
                    }
                    Err(e) => {
                        debug!(
                            logging::StreamRpcLog {
                                transport: task_client.connection_context.transport.as_str(),
                                remote_addr: task_client.connection_context.remote_addr.as_deref(),
                                user_agent: None,
                                action: logging::StreamRpcAction::ClientConnectionLog(
                                    logging::ClientConnectionLog {
                                        client_id: Some(task_client.id),
                                        forwarded: None,
                                        rpc_method: None,
                                    }
                                ),
                            },
                            "client disconnect start {}", e
                        );
                        break;
                    }
                };
            }
        });

        // Wait for either the side of the stream to be closed
        tokio::select! {
            _ = send_task => (),
            _ = recv_task => (),
        }

        // One of the streams has been closed, so it's time to disconnect & cleanup
        self.clients.write().remove(&client_id);

        debug!(
            logging::StreamRpcLog {
                transport: client.connection_context.transport.as_str(),
                remote_addr: client.connection_context.remote_addr.as_deref(),
                user_agent: None,
                action: logging::StreamRpcAction::ClientConnectionLog(
                    logging::ClientConnectionLog {
                        client_id: Some(client.id),
                        forwarded: None,
                        rpc_method: None,
                    }
                ),
            },
            "client disconnected"
        );
    }
}
