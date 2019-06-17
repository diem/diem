// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use futures::{channel::mpsc, Future, FutureExt, StreamExt, TryFutureExt};
use std::{pin::Pin, sync::Arc};
use tokio::runtime::TaskExecutor;

/// EventBasedActor trait represents the actor style objects that are driven by some input
/// stream of events and that generate an output stream of events in response.
///
/// Please see an example of the usage in stream_utils_test.rs
pub trait EventBasedActor {
    type InputEvent;
    type OutputEvent;

    /// Called synchronously when EventBasedActor startup
    fn init(
        &mut self,
        input_stream_sender: mpsc::Sender<Self::InputEvent>,
        output_stream_sender: mpsc::Sender<Self::OutputEvent>,
    );

    /// Called before the main event loop starts.
    /// The returned future is chained s.t. the first event processing starts after the completion
    /// of the startup preparations.
    fn on_startup(&self) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        async {}.boxed()
    }

    /// Process a new event from the stream.
    /// An implementation can generate new input / output events as a result.
    /// The returned future is chained by the main event processing loop s.t.
    /// the total order of the returned futures is preserved.
    fn process_event(&self, event: Self::InputEvent) -> Pin<Box<dyn Future<Output = ()> + Send>>;
}

/// Starts a loop of event processing for a given actor.
/// Events are received via the given input received, processed one by one by the actor, which is
/// kept in the state of the executor using the 'fold' function.
pub fn start_event_processing_loop<A>(
    actor: &mut Arc<A>,
    executor: TaskExecutor,
) -> (mpsc::Sender<A::InputEvent>, mpsc::Receiver<A::OutputEvent>)
where
    A: EventBasedActor + Send + Sync + 'static + ?Sized,
    A::InputEvent: Send,
{
    let (input_tx, mut input_rx, output_tx, output_rx) = prep_channels::<A>();
    Arc::get_mut(actor)
        .expect("can not clone Arc<dyn EventBasedActor> before start")
        .init(input_tx.clone(), output_tx);

    let actor = Arc::clone(actor);

    let processing_loop = async move {
        actor.on_startup().await;

        while let Some(event) = input_rx.next().await {
            actor.process_event(event).await;
        }
    };
    executor.spawn(processing_loop.boxed().unit_error().compat());
    (input_tx, output_rx)
}

/// Generates the mpsc channels for input and output events.
pub fn prep_channels<A>() -> (
    mpsc::Sender<A::InputEvent>,
    mpsc::Receiver<A::InputEvent>,
    mpsc::Sender<A::OutputEvent>,
    mpsc::Receiver<A::OutputEvent>,
)
where
    A: EventBasedActor + Send + 'static + ?Sized,
{
    let (input_events_tx, input_events_rx) = mpsc::channel(1_024);
    let (output_events_tx, output_events_rx) = mpsc::channel(1_024);
    (
        input_events_tx,
        input_events_rx,
        output_events_tx,
        output_events_rx,
    )
}
