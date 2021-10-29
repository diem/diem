// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    data_notification,
    data_notification::{DataClientRequest, DataNotification, DataPayload, NotificationId},
    error::Error,
    logging::{LogEntry, LogEvent, LogSchema},
    stream_progress_tracker::{DataStreamTracker, StreamProgressTracker},
    streaming_client::{NotificationFeedback, StreamRequest},
};
use channel::{diem_channel, message_queues::QueueStyle};
use diem_data_client::{
    AdvertisedData, DataClientPayload, DataClientResponse, DiemDataClient, GlobalDataSummary,
    ResponseError, ResponseId,
};
use diem_infallible::Mutex;
use diem_logger::prelude::*;
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

// Maximum number of retries for a single client request before the stream terminates
pub const MAX_REQUEST_RETRY: u64 = 10;

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
    // The unique ID for this data stream. This is useful for logging.
    data_stream_id: DataStreamId,

    // The data client through which to fetch data from the Diem network
    diem_data_client: T,

    // The fulfillment progress tracker for this data stream
    stream_progress_tracker: StreamProgressTracker,

    // The current queue of data client requests and pending responses. When the
    // request at the head of the queue completes (i.e., we receive a response),
    // a data notification can be created and sent along the stream.
    sent_data_requests: Option<VecDeque<PendingClientResponse>>,

    // TODO(joshlind): garbage collect me!
    // Maps a notification ID (sent along the data stream) to a response ID
    // received from the data client. This is useful for providing feedback.
    notifications_to_responses: HashMap<NotificationId, ResponseId>,

    // The channel on which to send data notifications when they are ready.
    notification_sender: channel::diem_channel::Sender<(), DataNotification>,

    // A unique notification ID generator
    notification_id_generator: Arc<AtomicU64>,

    // Notification ID of the end of stream notification (when it has been sent)
    stream_end_notification_id: Option<NotificationId>,

    // The current failure count of the request at the head of the request queue.
    // If this count becomes too large, the stream is evidently blocked (i.e.,
    // unable to make progress) and will automatically terminate.
    request_failure_count: u64,
}

impl<T: DiemDataClient + Send + Clone + 'static> DataStream<T> {
    pub fn new(
        data_stream_id: DataStreamId,
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
            data_stream_id,
            diem_data_client,
            stream_progress_tracker,
            sent_data_requests: None,
            notifications_to_responses: HashMap::new(),
            notification_sender,
            notification_id_generator,
            stream_end_notification_id: None,
            request_failure_count: 0,
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
        global_data_summary: GlobalDataSummary,
    ) -> Result<(), Error> {
        // Initialize the data client requests queue
        self.sent_data_requests = Some(VecDeque::new());

        // Create and send the data client requests to the network
        self.create_and_send_client_requests(&global_data_summary)
    }

    /// Returns true iff the given `notification_id` was sent by this stream
    pub fn sent_notification(&self, notification_id: &NotificationId) -> bool {
        if let Some(stream_end_notification_id) = self.stream_end_notification_id {
            if stream_end_notification_id == *notification_id {
                return true;
            }
        }

        self.notifications_to_responses
            .get(notification_id)
            .is_some()
    }

    /// Notifies the Diem data client of a bad client response
    pub fn handle_notification_feedback(
        &self,
        notification_id: &NotificationId,
        notification_feedback: &NotificationFeedback,
    ) -> Result<(), Error> {
        if self.stream_end_notification_id == Some(*notification_id) {
            if matches!(notification_feedback, NotificationFeedback::EndOfStream) {
                return Ok(());
            } else {
                return Err(Error::UnexpectedErrorEncountered(format!(
                    "Invalid feedback given for stream end: {:?}",
                    notification_feedback
                )));
            }
        }

        let response_id = *self
            .notifications_to_responses
            .get(notification_id)
            .ok_or_else(|| {
                Error::UnexpectedErrorEncountered(format!(
                    "Response ID missing for notification ID: {:?}",
                    notification_id
                ))
            })?;
        let response_error = extract_response_error(notification_feedback);
        self.notify_bad_response(response_id, response_error);

        Ok(())
    }

    /// Creates and sends a batch of diem data client requests to the network
    fn create_and_send_client_requests(
        &mut self,
        global_data_summary: &GlobalDataSummary,
    ) -> Result<(), Error> {
        // Determine how many requests (at most) can be sent to the network
        let num_sent_requests = self.get_sent_data_requests().len() as u64;
        let max_num_requests_to_send = MAX_CONCURRENT_REQUESTS
            .checked_sub(num_sent_requests)
            .ok_or_else(|| {
                Error::IntegerOverflow("Max number of requests to send has overflown!".into())
            })?;

        if max_num_requests_to_send > 0 {
            for client_request in self
                .stream_progress_tracker
                .create_data_client_requests(max_num_requests_to_send, global_data_summary)?
            {
                // Send the client request
                let pending_client_response = self.send_client_request(client_request.clone());

                // Enqueue the pending response
                self.get_sent_data_requests()
                    .push_back(pending_client_response);
            }
            debug!(
                (LogSchema::new(LogEntry::SendDataRequests)
                    .stream_id(self.data_stream_id)
                    .event(LogEvent::Success)
                    .message(&format!(
                        "Sent {:?} data requests to the network",
                        max_num_requests_to_send
                    )))
            );
        }
        Ok(())
    }

    /// Sends a given request to the data client to be forwarded to the network
    /// and returns a pending client response.
    fn send_client_request(
        &mut self,
        data_client_request: DataClientRequest,
    ) -> PendingClientResponse {
        // Create a new pending client response
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
            DataClientRequest::AccountsWithProof(request) => {
                tokio::spawn(async move {
                    let client_response = diem_data_client.get_account_states_with_proof(
                        request.version,
                        request.start_index,
                        request.end_index,
                    );
                    let client_response = client_response.await;
                    pending_response.lock().client_response = Some(client_response);
                });
            }
            DataClientRequest::EpochEndingLedgerInfos(request) => {
                tokio::spawn(async move {
                    let client_response = diem_data_client
                        .get_epoch_ending_ledger_infos(request.start_epoch, request.end_epoch);
                    let client_response = client_response.await;
                    pending_response.lock().client_response = Some(client_response);
                });
            }
            DataClientRequest::NumberOfAccounts(request) => {
                tokio::spawn(async move {
                    let client_response =
                        diem_data_client.get_number_of_account_states(request.version);
                    let client_response = client_response.await;
                    pending_response.lock().client_response = Some(client_response);
                });
            }
            DataClientRequest::TransactionOutputsWithProof(request) => {
                tokio::spawn(async move {
                    let client_response = diem_data_client.get_transaction_outputs_with_proof(
                        request.max_proof_version,
                        request.start_version,
                        request.end_version,
                    );
                    let client_response = client_response.await;
                    pending_response.lock().client_response = Some(client_response);
                });
            }
            DataClientRequest::TransactionsWithProof(request) => {
                tokio::spawn(async move {
                    let client_response = diem_data_client.get_transactions_with_proof(
                        request.max_proof_version,
                        request.start_version,
                        request.end_version,
                        request.include_events,
                    );
                    let client_response = client_response.await;
                    pending_response.lock().client_response = Some(client_response);
                });
            }
        }

        pending_client_response
    }

    fn send_data_notification(&self, data_notification: DataNotification) -> Result<(), Error> {
        self.notification_sender
            .push((), data_notification)
            .map_err(|error| Error::UnexpectedErrorEncountered(error.to_string()))
    }

    fn send_end_of_stream_notification(&mut self) -> Result<(), Error> {
        // Create end of stream notification
        let notification_id = self
            .notification_id_generator
            .fetch_add(1, Ordering::Relaxed);
        let data_notification = DataNotification {
            notification_id,
            data_payload: DataPayload::EndOfStream,
        };

        // Send the data notification
        info!(
            (LogSchema::new(LogEntry::EndOfStreamNotification)
                .stream_id(self.data_stream_id)
                .event(LogEvent::Pending)
                .message("Sent the end of stream notification"))
        );
        self.stream_end_notification_id = Some(notification_id);
        self.send_data_notification(data_notification)
    }

    /// Processes any data client responses that have been received. Note: the
    /// responses must be processed in FIFO order.
    pub fn process_data_responses(
        &mut self,
        global_data_summary: GlobalDataSummary,
    ) -> Result<(), Error> {
        if self.stream_progress_tracker.is_stream_complete()
            || self.request_failure_count >= MAX_REQUEST_RETRY
        {
            if self.stream_end_notification_id.is_none() {
                self.send_end_of_stream_notification()?;
            }
            return Ok(()); // There's nothing left to do
        }

        // Process any ready data responses
        for _ in 0..MAX_CONCURRENT_REQUESTS {
            if let Some(pending_response) = self.pop_pending_response_queue() {
                let pending_response = pending_response.lock();
                let client_response = pending_response
                    .client_response
                    .as_ref()
                    .expect("The client response should be ready!");
                match client_response {
                    Ok(client_response) => {
                        if sanity_check_client_response(
                            &pending_response.client_request,
                            client_response,
                        ) {
                            self.send_data_notification_to_client(
                                &pending_response.client_request,
                                client_response,
                            )?;
                        } else {
                            self.handle_sanity_check_failure(
                                &pending_response.client_request,
                                client_response,
                            )?;
                            break;
                        }
                    }
                    Err(error) => {
                        self.handle_data_client_error(&pending_response.client_request, error)?;
                        break;
                    }
                }
            } else {
                break; // The first response hasn't arrived yet.
            }
        }

        // Create and send further client requests to the network
        // to ensure we're maximizing the number of concurrent requests.
        self.create_and_send_client_requests(&global_data_summary)
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

    /// Handles a client response that failed sanity checks
    fn handle_sanity_check_failure(
        &mut self,
        data_client_request: &DataClientRequest,
        data_client_response: &DataClientResponse,
    ) -> Result<(), Error> {
        error!(LogSchema::new(LogEntry::ReceivedDataResponse)
            .stream_id(self.data_stream_id)
            .event(LogEvent::Error)
            .message("Encountered a client response that failed the sanity checks!"));

        self.notify_bad_response(
            data_client_response.response_id,
            ResponseError::InvalidPayloadDataType,
        );
        self.resend_data_client_request(data_client_request)
    }

    /// Handles an error returned by the data client in relation to a request
    fn handle_data_client_error(
        &mut self,
        data_client_request: &DataClientRequest,
        data_client_error: &diem_data_client::Error,
    ) -> Result<(), Error> {
        error!(LogSchema::new(LogEntry::ReceivedDataResponse)
            .stream_id(self.data_stream_id)
            .event(LogEvent::Error)
            .error(&data_client_error.clone().into())
            .message("Encountered a data client error!"));

        // TODO(joshlind): can we identify the best way to react to the error?
        self.resend_data_client_request(data_client_request)
    }

    /// Resends a failed data client request and pushes the pending notification
    /// to the head of the pending notifications batch.
    fn resend_data_client_request(
        &mut self,
        data_client_request: &DataClientRequest,
    ) -> Result<(), Error> {
        // Increment the number of client failures for this request
        self.request_failure_count += 1;

        // Resend the client request
        let pending_client_response = self.send_client_request(data_client_request.clone());

        // Push the pending response to the head of the sent requests queue
        self.get_sent_data_requests()
            .push_front(pending_client_response);

        Ok(())
    }

    /// Notifies the Diem data client of a bad client response
    fn notify_bad_response(&self, response_id: ResponseId, response_error: ResponseError) {
        let data_stream_id = self.data_stream_id;
        let diem_data_client = self.diem_data_client.clone();

        info!(LogSchema::new(LogEntry::ReceivedDataResponse)
            .stream_id(data_stream_id)
            .event(LogEvent::Error)
            .message(&format!(
                "Notifying the data client of a bad response. Response id: {:?}, error: {:?}",
                response_id, response_error
            )));

        tokio::spawn(async move {
            let client_response = diem_data_client.notify_bad_response(response_id, response_error);
            if let Err(error) = client_response.await {
                error!(LogSchema::new(LogEntry::DiemDataClient)
                    .stream_id(data_stream_id)
                    .event(LogEvent::Error)
                    .error(&error.into()));
            }
        });
    }

    /// Sends a data notification to the client along the stream
    fn send_data_notification_to_client(
        &mut self,
        data_client_request: &DataClientRequest,
        data_client_response: &DataClientResponse,
    ) -> Result<(), Error> {
        // Create a new data notification
        if let Some(data_notification) = self
            .stream_progress_tracker
            .transform_client_response_into_notification(
                data_client_request,
                data_client_response,
                self.notification_id_generator.clone(),
            )?
        {
            // Save the data notification ID and response ID
            let notification_id = data_notification.notification_id;
            if let Some(response_id) = self
                .notifications_to_responses
                .insert(notification_id, data_client_response.response_id)
            {
                panic!(
                    "Duplicate sent notification ID found! Notification ID: {:?}, Response ID: {:?}",
                    notification_id, response_id,
                );
            }

            // Send the notification along the stream
            debug!(
                (LogSchema::new(LogEntry::StreamNotification)
                    .stream_id(self.data_stream_id)
                    .event(LogEvent::Success)
                    .message(&format!(
                        "Sent a single stream notification! Notification ID: {:?}",
                        notification_id
                    )))
            );
            self.send_data_notification(data_notification)?;

            // Reset the failure count. We've sent a notification and can move on.
            self.request_failure_count = 0;
        }

        Ok(())
    }

    /// Verifies that the data required by the stream can be satisfied using the
    /// currently advertised data in the network. If not, returns an error.
    pub fn ensure_data_is_available(&self, advertised_data: &AdvertisedData) -> Result<(), Error> {
        if !self
            .stream_progress_tracker
            .is_remaining_data_available(advertised_data)
        {
            return Err(Error::DataIsUnavailable(format!(
                "Unable to satisfy stream progress tracker: {:?}, with advertised data: {:?}",
                self.stream_progress_tracker, advertised_data
            )));
        }
        Ok(())
    }

    /// Assumes the caller has already verified that `sent_data_requests` has
    /// been initialized.
    fn get_sent_data_requests(&mut self) -> &mut VecDeque<PendingClientResponse> {
        self.sent_data_requests
            .as_mut()
            .expect("Sent data requests should be initialized!")
    }

    #[cfg(test)]
    /// This is exposed and used only for test purposes.
    pub fn get_sent_requests_and_notifications(
        &mut self,
    ) -> (
        &mut Option<VecDeque<PendingClientResponse>>,
        &mut HashMap<NotificationId, ResponseId>,
    ) {
        let sent_requests = &mut self.sent_data_requests;
        let sent_notifications = &mut self.notifications_to_responses;

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
        DataClientRequest::NumberOfAccounts(_) => {
            matches!(
                data_client_response.response_payload,
                DataClientPayload::NumberOfAccountStates(_)
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

/// Transforms the notification feedback into a specific response error that
/// can be sent to the Diem data client.
fn extract_response_error(notification_feedback: &NotificationFeedback) -> ResponseError {
    match notification_feedback {
        NotificationFeedback::InvalidPayloadData => ResponseError::InvalidData,
        NotificationFeedback::PayloadTypeIsIncorrect => ResponseError::InvalidPayloadDataType,
        NotificationFeedback::PayloadProofFailed => ResponseError::ProofVerificationError,
        _ => {
            panic!(
                "Invalid notification feedback given: {:?}",
                notification_feedback
            )
        }
    }
}
