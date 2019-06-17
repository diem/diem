// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::stream_utils::{start_event_processing_loop, EventBasedActor};
use futures::{channel::mpsc, executor::block_on, Future, FutureExt, SinkExt, StreamExt};
use logger::prelude::*;
use std::{
    pin::Pin,
    sync::{Arc, RwLock},
};
use tokio::runtime;

struct FibonacciActor {
    a: u32,
    b: u32,
    output_stream: Option<mpsc::Sender<u32>>,
}

impl FibonacciActor {
    pub fn new() -> Self {
        Self {
            a: 0,
            b: 1,
            output_stream: None,
        }
    }
}

impl EventBasedActor for RwLock<FibonacciActor> {
    type InputEvent = ();
    type OutputEvent = u32;

    fn init(
        &mut self,
        _: mpsc::Sender<Self::InputEvent>,
        output_stream: mpsc::Sender<Self::OutputEvent>,
    ) {
        self.write().unwrap().output_stream = Some(output_stream);
    }

    fn process_event(&self, _: Self::InputEvent) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        let mut guard = self.write().unwrap();
        let next = guard.a + guard.b;
        let mut sender = guard.output_stream.as_ref().unwrap().clone();
        let a = guard.a;
        let send_fut = async move {
            if let Err(e) = sender.send(a).await {
                debug!("Error in sending output event {:?}", e);
            }
        };
        guard.a = guard.b;
        guard.b = next;
        send_fut.boxed()
    }
}

#[test]
fn test_event_loop() {
    let runtime = runtime::Builder::new()
        .build()
        .expect("Failed to create Tokio runtime!");
    let mut fib_actor = Arc::new(RwLock::new(FibonacciActor::new()));
    let (mut input_tx, output_rx) = start_event_processing_loop(&mut fib_actor, runtime.executor());

    block_on(async move {
        for _ in 0..5 {
            input_tx.send(()).await.unwrap();
        }
        assert_eq!(
            output_rx.take(5).collect::<Vec<_>>().await,
            vec![0, 1, 1, 2, 3,]
        );
    });
}
