// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    data_notification,
    data_notification::{
        DataClientRequest, DataNotification, DataPayload, NotificationId, SentDataNotification,
    },
    error::Error,
    stream_progress_tracker::StreamProgressTracker,
    streaming_client::StreamRequest,
};
use channel::{diem_channel, message_queues::QueueStyle};
use diem_data_client::{
    AdvertisedData, DataClientPayload, DataClientResponse, DiemDataClient, OptimalChunkSizes,
    ResponseError,
};
use diem_infallible::Mutex;
use futures::{stream::FusedStream, Stream};
use std::{
    collections::{HashMap, VecDeque},
    pin::Pin,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    task::{Context, Poll},
};

// Maximum channel sizes for each stream listener. If messages are not
// consumed, they will be dropped (oldest messages first). The remaining
// messages will be retrieved using FIFO ordering.
const DATA_STREAM_CHANNEL_SIZE: usize = 1000;

// Maximum number of concurrent data client requests (per stream)
const MAX_CONCURRENT_REQUESTS: u64 = 3;

/// A unique ID used to identify each stream.
pub type DataStreamId = u64;

/// A pointer to a thread-safe `PendingClientResponse`.
pub type PendingClientResponse = Arc<Mutex<Box<data_notification::PendingClientResponse>>>;

/// Each data stream holds the original stream request from the client and tracks
/// the progress of the data stream to satisfy that request (e.g., the data that
/// has already been sent along the stream to the client and the in-flight diem
/// data client requests that have been sent to the network).
///
/// Note that it is the responsibility of the data stream to send data
/// notifications along the stream in sequential order (e.g., transactions and
/// proofs must be sent with monotonically increasing versions).
#[derive(Debug)]
pub struct DataStream<T> {
    // The data client through which to fetch data from the Diem network
    diem_data_client: T,

    // The fulfillment progress tracker for this data stream
    stream_progress_tracker: StreamProgressTracker,

    // The current queue of data client requests and pending responses. When the
    // request at the head of the queue completes (i.e., we receive a response),
    // a data notification can be created and sent along the stream.
    sent_data_requests: Option<VecDeque<PendingClientResponse>>,

    // The data notifications already sent via this stream.
    sent_notifications: HashMap<NotificationId, SentDataNotification>,

    // The channel on which to send data notifications when they are ready.
    notification_sender: channel::diem_channel::Sender<(), DataNotification>,

    // A unique notification ID generator
    notification_id_generator: Arc<AtomicU64>,
}

impl<T: DiemDataClient + Send + Clone + 'static> DataStream<T> {
    pub fn new(
        stream_request: &StreamRequest,
        diem_data_client: T,
        notification_id_generator: Arc<AtomicU64>,
        advertised_data: &AdvertisedData,
    ) -> Result<(Self, DataStreamListener), Error> {
        // Create a new data stream listener
        let (notification_sender, notification_receiver) =
            diem_channel::new(QueueStyle::KLAST, DATA_STREAM_CHANNEL_SIZE, None);
        let data_stream_listener = DataStreamListener::new(notification_receiver);

        // Create a new stream progress tracker
        let stream_progress_tracker = StreamProgressTracker::new(stream_request, advertised_data)?;

        // Create a new data stream
        let data_stream = Self {
            diem_data_client,
            stream_progress_tracker,
            sent_data_requests: None,
            sent_notifications: HashMap::new(),
            notification_sender,
            notification_id_generator,
        };

        Ok((data_stream, data_stream_listener))
    }

    /// Returns true iff the first batch of data client requests has been sent
    pub fn data_requests_initialized(&self) -> bool {
        self.sent_data_requests.is_some()
    }

    /// Initializes the data client requests by sending out the first batch
    pub fn initialize_data_requests(
        &mut self,
        optimal_chunk_sizes: OptimalChunkSizes,
    ) -> Result<(), Error> {
        // Initialize the data client requests queue
        self.sent_data_requests = Some(VecDeque::new());

        // Create and send the data client requests to the network
        self.create_and_send_client_requests(MAX_CONCURRENT_REQUESTS, &optimal_chunk_sizes)
    }

    /// Creates and sends a batch of diem data client requests (at most
    /// `max_number_of_requests`).
    fn create_and_send_client_requests(
        &mut self,
        max_number_of_requests: u64,
        optimal_chunk_sizes: &OptimalChunkSizes,
    ) -> Result<(), Error> {
        for client_request in
            self.create_data_client_requests(max_number_of_requests, optimal_chunk_sizes)?
        {
            // Send the client request
            let pending_client_response = self.send_client_request(client_request.clone());

            // Push the pending response to the back of the sent requests queue
            self.get_sent_data_requests()
                .push_back(pending_client_response);

            // Update the stream progress tracker
            self.stream_progress_tracker
                .update_request_progress(&client_request)?;
        }
        Ok(())
    }

    /// Creates a batch of diem data client requests (at most `max_number_of_requests`).
    fn create_data_client_requests(
        &mut self,
        max_number_of_requests: u64,
        optimal_chunk_sizes: &OptimalChunkSizes,
    ) -> Result<Vec<DataClientRequest>, Error> {
        match &mut self.stream_progress_tracker {
            StreamProgressTracker::EpochEndingStreamTracker(stream_tracker) => stream_tracker
                .create_epoch_ending_client_requests(
                    max_number_of_requests,
                    optimal_chunk_sizes.epoch_chunk_size,
                ),
        }
    }

    /// Sends a given request to the data client to be forwarded to the network
    /// and returns a pending client response.
    fn send_client_request(
        &mut self,
        data_client_request: DataClientRequest,
    ) -> PendingClientResponse {
        // Save the request in the sent request queue
        let pending_client_response = Arc::new(Mutex::new(Box::new(
            data_notification::PendingClientResponse {
                client_request: data_client_request.clone(),
                client_response: None,
            },
        )));

        // Send the request to the network
        let diem_data_client = self.diem_data_client.clone();
        let pending_response = pending_client_response.clone();
        match data_client_request {
            DataClientRequest::EpochEndingLedgerInfos(request) => {
                tokio::spawn(async move {
                    let client_response = diem_data_client
                        .get_epoch_ending_ledger_infos(request.start_epoch, request.end_epoch);
                    let client_response = client_response.await;
                    pending_response.lock().client_response = Some(client_response);
                });
            }
            _ => {
                panic!("Data client request is currently unsupported!");
            }
        }

        pending_client_response
    }

    /// Processes any data client responses that have been received. Note: the
    /// responses must be processed in FIFO order.
    pub fn process_data_responses(
        &mut self,
        optimal_chunk_sizes: OptimalChunkSizes,
    ) -> Result<(), Error> {
        for _ in 0..MAX_CONCURRENT_REQUESTS {
            // Get the data client response at the head of the queue if it's ready
            if let Some(pending_response) = self.pop_pending_response_queue() {
                let pending_response = pending_response.lock();
                if let Some(client_response) = &pending_response.client_response {
                    match client_response {
                        Ok(client_response) => {
                            if sanity_check_client_response(
                                &pending_response.client_request,
                                client_response,
                            ) {
                                // Send a data notification and make the next data client request
                                self.send_data_notification_to_client(
                                    &pending_response.client_request,
                                    client_response,
                                )?;
                                self.create_and_send_client_requests(1, &optimal_chunk_sizes)?;
                            } else {
                                // Notify the data client and re-fetch the data
                                self.notify_bad_response(client_response);
                                return self
                                    .resend_data_client_request(&pending_response.client_request);
                            }
                        }
                        Err(error) => {
                            return self
                                .handle_data_client_error(&pending_response.client_request, error);
                        }
                    }
                } else {
                    panic!("The client response should be ready!");
                }
            } else {
                return Ok(()); // The first response hasn't arrived yet.
            }
        }
        Ok(())
    }

    /// Pops and returns the first pending client response if the response has
    /// been received. Returns `None` otherwise.
    fn pop_pending_response_queue(&mut self) -> Option<PendingClientResponse> {
        let sent_data_requests = self.get_sent_data_requests();
        if let Some(data_request) = sent_data_requests.front() {
            if data_request.lock().client_response.is_some() {
                // We've received a response! Pop the requests off the queue.
                sent_data_requests.pop_front()
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Handles errors returned by the data client in relation to a request.
    fn handle_data_client_error(
        &mut self,
        data_client_request: &DataClientRequest,
        _data_client_error: &diem_data_client::Error,
    ) -> Result<(), Error> {
        // TODO(joshlind): don't just resend the request. Identify the best
        // way to react based on the error.
        self.resend_data_client_request(data_client_request)
    }

    /// Resends a failed data client request and pushes the pending notification
    /// to the head of the pending notifications batch.
    fn resend_data_client_request(
        &mut self,
        data_client_request: &DataClientRequest,
    ) -> Result<(), Error> {
        // Resend the client request
        let pending_client_response = self.send_client_request(data_client_request.clone());

        // Push the pending response to the head of the sent requests queue as
        // this is a resend.
        self.get_sent_data_requests()
            .push_front(pending_client_response);
        Ok(())
    }

    /// Notifies the Diem data client of a bad client response
    fn notify_bad_response(&self, data_client_response: &DataClientResponse) {
        let response_id = data_client_response.response_id;
        let response_error = ResponseError::InvalidPayloadDataType;
        let diem_data_client = self.diem_data_client.clone();

        tokio::spawn(async move {
            let client_response = diem_data_client.notify_bad_response(response_id, response_error);
            let _ = client_response.await;
            // TODO(joshlind): log the response if it's an error?
        });
    }

    /// Sends a data notification to the client along the stream
    fn send_data_notification_to_client(
        &mut self,
        data_client_request: &DataClientRequest,
        data_client_response: &DataClientResponse,
    ) -> Result<(), Error> {
        // Create a new notification id
        let notification_id = self
            .notification_id_generator
            .fetch_add(1, Ordering::Relaxed);

        // Send a data notification to the client
        let data_notification = DataNotification {
            notification_id,
            data_payload: extract_data_payload(data_client_response),
        };
        self.notification_sender
            .push((), data_notification)
            .map_err(|error| Error::UnexpectedErrorEncountered(error.to_string()))?;

        // Save the sent data notification to track any future re-fetches
        let sent_data_notification = SentDataNotification {
            client_request: data_client_request.clone(),
            client_response: data_client_response.clone(),
        };
        if let Some(existing_notification) = self
            .sent_notifications
            .insert(notification_id, sent_data_notification.clone())
        {
            panic!(
                "Duplicate sent notification found! This should not occur! ID: {}, notification: {:?}",
                notification_id, existing_notification
            );
        }

        // Update the stream progress tracker with the sent notification
        self.stream_progress_tracker
            .update_notification_progress(&sent_data_notification)
    }

    /// Verifies that the data required by the stream can be satisfied using the
    /// currently advertised data in the network. If not, returns an error.
    pub fn ensure_data_is_available(&self, advertised_data: &AdvertisedData) -> Result<(), Error> {
        self.stream_progress_tracker
            .ensure_data_is_available(advertised_data)
    }

    fn get_sent_data_requests(&mut self) -> &mut VecDeque<PendingClientResponse> {
        if let Some(sent_data_requests) = &mut self.sent_data_requests {
            sent_data_requests
        } else {
            panic!("Sent data requests should be initialized!");
        }
    }

    #[cfg(test)]
    /// This is exposed and used only for test purposes.
    pub fn get_sent_requests_and_notifications(
        &mut self,
    ) -> (
        &mut Option<VecDeque<PendingClientResponse>>,
        &mut HashMap<NotificationId, SentDataNotification>,
    ) {
        let sent_requests = &mut self.sent_data_requests;
        let sent_notifications = &mut self.sent_notifications;

        (sent_requests, sent_notifications)
    }
}

/// Allows listening to data streams (i.e., streams of data notifications).
#[derive(Debug)]
pub struct DataStreamListener {
    notification_receiver: channel::diem_channel::Receiver<(), DataNotification>,
}

impl DataStreamListener {
    pub fn new(
        notification_receiver: channel::diem_channel::Receiver<(), DataNotification>,
    ) -> Self {
        Self {
            notification_receiver,
        }
    }
}

impl Stream for DataStreamListener {
    type Item = DataNotification;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.get_mut().notification_receiver).poll_next(cx)
    }
}

impl FusedStream for DataStreamListener {
    fn is_terminated(&self) -> bool {
        self.notification_receiver.is_terminated()
    }
}

/// Extracts the `DataPayload` out of a `DataClientResponse`. Assumes that the
/// response has already been sanity checked.
fn extract_data_payload(data_client_response: &DataClientResponse) -> DataPayload {
    match &data_client_response.response_payload {
        DataClientPayload::AccountStatesWithProof(accounts_chunk) => {
            DataPayload::AccountStatesWithProof(accounts_chunk.clone())
        }
        DataClientPayload::EpochEndingLedgerInfos(ledger_infos) => {
            DataPayload::EpochEndingLedgerInfos(ledger_infos.clone())
        }
        DataClientPayload::TransactionsWithProof(transactions_chunk) => {
            DataPayload::TransactionsWithProof(transactions_chunk.clone())
        }
        DataClientPayload::TransactionOutputsWithProof(transactions_output_chunk) => {
            DataPayload::TransactionOutputsWithProof(transactions_output_chunk.clone())
        }
        _ => {
            panic!(
                "The response was already sanity checked but is now type mismatched: {:?}",
                data_client_response
            );
        }
    }
}

/// Returns true iff the data client response payload matches the expected type
/// of the original request. No other sanity checks are done.
fn sanity_check_client_response(
    data_client_request: &DataClientRequest,
    data_client_response: &DataClientResponse,
) -> bool {
    match data_client_request {
        DataClientRequest::AccountsWithProof(_) => {
            matches!(
                data_client_response.response_payload,
                DataClientPayload::AccountStatesWithProof(_)
            )
        }
        DataClientRequest::EpochEndingLedgerInfos(_) => {
            matches!(
                data_client_response.response_payload,
                DataClientPayload::EpochEndingLedgerInfos(_)
            )
        }
        DataClientRequest::TransactionsWithProof(_) => {
            matches!(
                data_client_response.response_payload,
                DataClientPayload::TransactionsWithProof(_)
            )
        }
        DataClientRequest::TransactionOutputsWithProof(_) => {
            matches!(
                data_client_response.response_payload,
                DataClientPayload::TransactionOutputsWithProof(_)
            )
        }
    }
}
