// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use futures::StreamExt;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use warp::{filters::BoxedFilter, Filter, Rejection, Reply};

use diem_config::config::StreamConfig;
use diem_logger::debug;
use storage_interface::DbReader;

use crate::{
    errors::JsonRpcError,
    stream_rpc::{
        connection::{ConnectionContext, ConnectionManager},
        counters, logging,
        subscriptions::SubscriptionConfig,
        transport::util::{get_remote_addr, Transport},
    },
};
use warp::sse::Event;

pub fn get_sse_routes(
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

    let sse_route = warp::path("v1")
        .and(warp::path("stream"))
        .and(warp::path("sse"))
        .and(warp::path::end())
        .and(warp::post())
        .and(warp::body::content_length_limit(content_length_limit))
        .and(warp::body::json())
        .and(warp::filters::header::header::<String>("user-agent"))
        .and(warp::any().map(move || connection_manager.clone()))
        .and(warp::header::headers_cloned())
        .and(warp::filters::addr::remote())
        .and_then(handle_sse_stream)
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

    (sse_route, cm2)
}

#[derive(Clone)]
pub struct ContextWrapperMapper {
    pub context: ConnectionContext,
}

impl ContextWrapperMapper {
    // Outgoing messages
    fn string_to_sse_event(
        &self,
        result: Result<std::string::String, anyhow::Error>,
    ) -> Result<Event, JsonRpcError> {
        counters::MESSAGES_SENT
            .with_label_values(&[
                self.context.transport.as_str(),
                self.context.sdk_info.language.as_str(),
                &self.context.sdk_info.version.to_string(),
            ])
            .inc();
        match result {
            Ok(string) => Ok(Event::default().data(string)),
            Err(e) => {
                println!("to_client_rcv_stream ERR: {:?}", e);
                Err(JsonRpcError::internal_error(format!("{:?}", e)))
            }
        }
    }
}

pub async fn handle_sse_stream(
    body: Vec<serde_json::Value>,
    user_agent: String,
    cm: ConnectionManager,
    headers: warp::http::HeaderMap,
    remote_socket: Option<std::net::SocketAddr>,
) -> Result<impl Reply, Rejection> {
    let sdk_info = crate::util::sdk_info_from_user_agent(Some(&user_agent));
    counters::HTTP_WEBSOCKET_REQUEST_UPGRADES
        .with_label_values(&[sdk_info.language.as_str(), &sdk_info.version.to_string()])
        .inc();

    let context = ConnectionContext {
        transport: Transport::SSE,
        sdk_info,
        remote_addr: get_remote_addr(&headers, remote_socket.as_ref()),
    };

    let cwm = Arc::new(ContextWrapperMapper {
        context: context.clone(),
    });

    let (to_client, to_client_rcv) =
        mpsc::channel::<Result<String, anyhow::Error>>(cm.config.queue_size);

    let (from_client, from_client_rcv) =
        mpsc::channel::<Result<Option<String>, anyhow::Error>>(cm.config.queue_size);

    tokio::task::spawn(async move {
        for value in body {
            from_client.send(Ok(Some(value.to_string()))).await.ok();
        }
    });

    let from_client_rcv_stream = ReceiverStream::new(from_client_rcv);
    tokio::task::spawn(async move {
        cm.client_connection(
            to_client.clone(),
            Box::new(from_client_rcv_stream),
            context,
            true,
        )
        .await;
    });

    let to_client_rcv_stream =
        ReceiverStream::new(to_client_rcv).map(move |result| cwm.string_to_sse_event(result));
    let event_stream = warp::sse::keep_alive().stream(to_client_rcv_stream);

    return Ok(warp::sse::reply(event_stream));
}
