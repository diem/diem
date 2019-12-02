// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use anyhow::{format_err, Error, Result};
use futures::{compat::Future01CompatExt, future::Future, prelude::*};
use futures_01::future::Future as Future01;
use grpcio::{ChannelBuilder, EnvBuilder, ServerBuilder};
use libra_logger::prelude::*;
use libra_metrics::counters::SVC_COUNTERS;
use std::{
    str::from_utf8,
    sync::{
        mpsc::{self, Sender},
        Arc,
    },
    thread, time,
};

pub fn default_reply_error_logger<T: std::fmt::Debug>(e: T) {
    error!("Failed to reply error due to {:?}", e)
}

pub fn create_grpc_invalid_arg_status(method: &str, err: ::anyhow::Error) -> ::grpcio::RpcStatus {
    let msg = format!("Request failed {}", err);
    error!("{} failed with {}", method, &msg);
    ::grpcio::RpcStatus::new(::grpcio::RpcStatusCode::INVALID_ARGUMENT, Some(msg))
}

/// This is a helper method to return a response to the GRPC context
/// and signal that the operation is done.
/// It's also logging any errors and incrementing relevant counters.
/// The return value is `bool` to flag externally whether the result
/// is successful (true) or not (false).
pub fn provide_grpc_response<ResponseType: std::fmt::Debug>(
    resp: Result<ResponseType>,
    ctx: ::grpcio::RpcContext<'_>,
    sink: ::grpcio::UnarySink<ResponseType>,
) {
    let mut success = true;
    match resp {
        Ok(resp) => ctx.spawn(sink.success(resp).map_err(default_reply_error_logger)),
        Err(e) => {
            success = false;
            let f = sink
                .fail(create_grpc_invalid_arg_status(
                    from_utf8(ctx.method()).expect("Unable to convert function name to string"),
                    e,
                ))
                .map_err(default_reply_error_logger);
            ctx.spawn(f)
        }
    }
    SVC_COUNTERS.resp(&ctx, success);
}

pub fn spawn_service_thread(
    service: ::grpcio::Service,
    service_host_address: String,
    service_public_port: u16,
    service_name: impl Into<String>,
) -> ServerHandle {
    spawn_service_thread_with_drop_closure(
        service,
        service_host_address,
        service_public_port,
        service_name,
        None, /* service_max_recv_msg_len */
        || { /* no code, to make compiler happy */ },
    )
}

pub fn spawn_service_thread_with_drop_closure<F>(
    service: ::grpcio::Service,
    service_host_address: String,
    service_public_port: u16,
    service_name: impl Into<String>,
    service_max_recv_msg_len: Option<i32>,
    service_drop_closure: F,
) -> ServerHandle
where
    F: FnOnce() + 'static,
{
    let env = Arc::new(EnvBuilder::new().name_prefix(service_name).build());
    let mut builder = ServerBuilder::new(Arc::clone(&env));
    if let Some(len) = service_max_recv_msg_len {
        let args = ChannelBuilder::new(Arc::clone(&env))
            .max_receive_message_len(len)
            .build_args();
        builder = builder.channel_args(args);
    }
    let server = builder
        .register_service(service)
        .bind(service_host_address, service_public_port)
        .build()
        .expect("Unable to create grpc server");
    ServerHandle::setup_with_drop_closure(server, Some(Box::new(service_drop_closure)))
}

pub struct ServerHandle {
    stop_sender: Sender<()>,
    drop_closure: Option<Box<dyn FnOnce()>>,
}

impl ServerHandle {
    pub fn setup_with_drop_closure(
        mut server: ::grpcio::Server,
        drop_closure: Option<Box<dyn FnOnce()>>,
    ) -> Self {
        let (start_sender, start_receiver) = mpsc::channel();
        let (stop_sender, stop_receiver) = mpsc::channel();
        let handle = Self {
            stop_sender,
            drop_closure,
        };
        thread::spawn(move || {
            server.start();
            start_sender.send(()).unwrap();
            loop {
                if stop_receiver.try_recv().is_ok() {
                    return;
                }
                thread::sleep(time::Duration::from_millis(100));
            }
        });

        start_receiver.recv().unwrap();
        handle
    }
    pub fn setup(server: ::grpcio::Server) -> Self {
        Self::setup_with_drop_closure(server, None)
    }
}

impl Drop for ServerHandle {
    fn drop(&mut self) {
        self.stop_sender.send(()).unwrap();
        if let Some(f) = self.drop_closure.take() {
            f()
        }
    }
}

pub fn convert_grpc_response<T>(
    response: grpcio::Result<impl Future01<Item = T, Error = grpcio::Error>>,
) -> impl Future<Output = Result<T>> {
    future::ready(response.map_err(convert_grpc_err))
        .map_ok(Future01CompatExt::compat)
        .and_then(|x| x.map_err(convert_grpc_err))
}

fn convert_grpc_err(e: ::grpcio::Error) -> Error {
    format_err!("grpc error: {}", e)
}
