// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;

use futures::{FutureExt, StreamExt};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use warp::{filters::BoxedFilter, ws::Message, Filter, Rejection, Reply};

use diem_config::config::StreamConfig;
use diem_logger::debug;
use storage_interface::DbReader;

use crate::stream_rpc::{
    connection::{ConnectionContext, ConnectionManager},
    counters, logging,
    subscriptions::SubscriptionConfig,
    transport::util::{get_remote_addr, Transport},
};

pub fn get_websocket_routes(
    config: &StreamConfig,
    content_length_limit: u64,
    diem_db: Arc<dyn DbReader>,
) -> (BoxedFilter<(impl Reply,)>, ConnectionManager) {
    let sub_config = Arc::new(SubscriptionConfig {
        fetch_size: config.subscription_fetch_size,
        poll_interval_ms: config.poll_interval_ms,
        queue_size: config.send_queue_size,
    });

    let connection_manager = ConnectionManager::new(diem_db.clone(), sub_config);

    let cm2 = connection_manager.clone();

    let ws_route = warp::path("v1")
        .and(warp::path("stream"))
        .and(warp::path("ws"))
        .and(warp::path::end())
        .and(warp::body::content_length_limit(content_length_limit))
        .and(warp::filters::header::header::<String>("user-agent"))
        .and(warp::ws())
        .and(warp::any().map(move || connection_manager.clone()))
        .and(warp::any().map(move || content_length_limit as usize))
        .and(warp::header::headers_cloned())
        .and(warp::filters::addr::remote())
        .and_then(handle_websocket_stream)
        .with(warp::cors().allow_any_origin())
        .with(warp::log::custom(|info| {
            debug!(
                logging::HttpRequestLog {
                    remote_addr: info.remote_addr(),
                    path: &info.path().to_string(),
                    status: info.status().as_u16(),
                    referer: info.referer(),
                    user_agent: info.user_agent().unwrap_or(""),
                    forwarded: info
                        .request_headers()
                        .get(warp::http::header::FORWARDED)
                        .and_then(|v| v.to_str().ok()),
                },
                "http request"
            )
        }))
        .boxed();

    (ws_route, cm2)
}

#[derive(Clone)]
pub struct ContextWrapperMapper {
    pub context: ConnectionContext,
}

impl ContextWrapperMapper {
    // Incoming messages
    pub fn message_result_to_string(
        &self,
        result: Result<Message, warp::Error>,
    ) -> Result<Option<String>, anyhow::Error> {
        counters::MESSAGES_RECEIVED
            .with_label_values(&[
                self.context.transport.as_str(),
                self.context.sdk_info.language.as_str(),
                &self.context.sdk_info.version.to_string(),
            ])
            .inc();

        match result {
            Ok(msg) => {
                // TODO: HANDLE PING + CLOSE MESSAGES
                // TODO: DROP MESSAGES THAT ARE TOO BIG
                match msg.to_str() {
                    Ok(s) => Ok(Some(s.to_string())),
                    // Just silently ignore messages that aren't text
                    Err(_) => Ok(None),
                }
            }
            Err(e) => Err(anyhow::Error::from(e)),
        }
    }
    // Outgoing messages
    fn string_result_to_message(&self, result: Result<String, anyhow::Error>) -> Message {
        counters::MESSAGES_SENT
            .with_label_values(&[
                self.context.transport.as_str(),
                self.context.sdk_info.language.as_str(),
                &self.context.sdk_info.version.to_string(),
            ])
            .inc();
        match result {
            Ok(item) => Message::text(item),
            // TODO: this aint right
            Err(_e) => Message::close(),
        }
    }
}

pub async fn handle_websocket_stream(
    user_agent: String,
    socket: warp::ws::Ws,
    cm: ConnectionManager,
    content_length_limit: usize,
    headers: warp::http::HeaderMap,
    remote_socket: Option<std::net::SocketAddr>,
) -> Result<impl Reply, Rejection> {
    let sdk_info = crate::util::sdk_info_from_user_agent(Some(&user_agent));
    counters::HTTP_WEBSOCKET_REQUESTS
        .with_label_values(&[sdk_info.language.as_str(), &sdk_info.version.to_string()])
        .inc();

    Ok(socket
        .max_send_queue(cm.config.queue_size)
        .max_message_size(content_length_limit)
        .on_upgrade(move |socket| {
            async move {
                counters::HTTP_WEBSOCKET_REQUEST_UPGRADES
                    .with_label_values(&[sdk_info.language.as_str(), &sdk_info.version.to_string()])
                    .inc();
                let context = ConnectionContext {
                    transport: Transport::Websocket,
                    sdk_info,
                    remote_addr: get_remote_addr(&headers, remote_socket.as_ref()),
                };

                let (to_client_ws, from_client_ws) = socket.split();
                let (to_client, to_client_rcv) =
                    mpsc::channel::<Result<String, anyhow::Error>>(cm.config.queue_size);

                let cwm = Arc::new(ContextWrapperMapper {
                    context: context.clone(),
                });

                // TODO: REAP CONNECTIONS WITHOUT SUBSCRIPTIONS AND THOSE THAT HAVEN'T ACCEPTED A MESSAGE IN A WHILE
                // Incoming
                let cwm_i = cwm.clone();
                let mapped_from_client_ws = from_client_ws
                    .map(move |result| cwm_i.clone().message_result_to_string(result));

                // Outgoing
                let cwm_o = cwm.clone();
                tokio::task::spawn(
                    ReceiverStream::new(to_client_rcv)
                        .map(move |result| Ok(cwm_o.string_result_to_message(result)))
                        .forward(to_client_ws)
                        .map(move |result: Result<(), warp::Error>| {
                            debug!(
                                logging::WebsocketDisconnect {
                                    remote_addr: remote_socket,
                                    user_agent: &user_agent,
                                    forwarded: headers
                                        .get(warp::http::header::FORWARDED)
                                        .and_then(|v| v.to_str().ok()),
                                },
                                "websocket disconnected ({:?})", result
                            )
                        }),
                );

                // This will keep running until the connection is closed
                cm.client_connection(to_client, Box::new(mapped_from_client_ws), context, false)
                    .await;
            }
        }))
}
