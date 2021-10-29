// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    data_stream::{DataStream, DataStreamId, DataStreamListener},
    error::Error,
    logging::{LogEntry, LogEvent, LogSchema},
    streaming_client::{
        StreamRequest, StreamRequestMessage, StreamingServiceListener, TerminateStreamRequest,
    },
};
use diem_data_client::{DataClientPayload, DiemDataClient, GlobalDataSummary, OptimalChunkSizes};
use diem_logger::prelude::*;
use futures::StreamExt;
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::Duration,
};
use tokio::time::interval;
use tokio_stream::wrappers::IntervalStream;

/// Constants for state management frequencies
const DATA_REFRESH_INTERVAL_MS: u64 = 1000;
const PROGRESS_CHECK_INTERVAL_MS: u64 = 100;

/// The data streaming service that responds to data stream requests.
pub struct DataStreamingService<T> {
    // The data client through which to fetch data from the Diem network
    diem_data_client: T,

    // Cached global data summary
    global_data_summary: GlobalDataSummary,

    // All requested data streams from clients
    data_streams: HashMap<DataStreamId, DataStream<T>>,

    // The listener through which to hear new client stream requests
    stream_requests: StreamingServiceListener,

    // Unique ID generators to maintain unique IDs across streams
    stream_id_generator: AtomicU64,
    notification_id_generator: Arc<AtomicU64>,
}

impl<T: DiemDataClient + Send + Clone + 'static> DataStreamingService<T> {
    pub fn new(diem_data_client: T, stream_requests: StreamingServiceListener) -> Self {
        Self {
            diem_data_client,
            global_data_summary: GlobalDataSummary::empty(),
            data_streams: HashMap::new(),
            stream_requests,
            stream_id_generator: AtomicU64::new(0),
            notification_id_generator: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Starts the dedicated streaming service
    pub async fn start_service(mut self) {
        let mut data_refresh_interval =
            IntervalStream::new(interval(Duration::from_millis(DATA_REFRESH_INTERVAL_MS))).fuse();
        let mut progress_check_interval =
            IntervalStream::new(interval(Duration::from_millis(PROGRESS_CHECK_INTERVAL_MS))).fuse();

        loop {
            ::futures::select! {
                stream_request = self.stream_requests.select_next_some() => {
                    self.handle_stream_request_message(stream_request);
                }
                _ = data_refresh_interval.select_next_some() => {
                    self.refresh_global_data_summary();
                }
                _ = progress_check_interval.select_next_some() => {
                    self.check_progress_of_all_data_streams();
                }
            }
        }
    }

    /// Handles new stream request messages from clients
    fn handle_stream_request_message(&mut self, request_message: StreamRequestMessage) {
        if let StreamRequest::TerminateStream(request) = request_message.stream_request {
            // Process the feedback request
            if let Err(error) = self.process_terminate_stream_request(&request) {
                error!(LogSchema::new(LogEntry::HandleTerminateRequest)
                    .event(LogEvent::Error)
                    .error(&error));
            }
            return;
        }

        // Process the stream request
        let response = self.process_new_stream_request(&request_message);
        if let Err(error) = &response {
            error!(LogSchema::new(LogEntry::HandleStreamRequest)
                .event(LogEvent::Error)
                .error(error));
        }

        // Send the response to the client
        if let Err(error) = request_message.response_sender.send(response) {
            error!(LogSchema::new(LogEntry::RespondToStreamRequest)
                .event(LogEvent::Error)
                .message(&format!(
                    "Failed to send response for stream request: {:?}",
                    error
                )));
        }
    }

    /// Processes a request for terminating the stream that sent a specific
    /// notification ID.
    fn process_terminate_stream_request(
        &mut self,
        terminate_request: &TerminateStreamRequest,
    ) -> Result<(), Error> {
        let notification_id = &terminate_request.notification_id;
        let notification_feedback = &terminate_request.notification_feedback;

        // Find the data stream that sent the notification
        let data_stream_ids = self.get_all_data_stream_ids();
        for data_stream_id in &data_stream_ids {
            let data_stream = self.get_data_stream(data_stream_id);
            if data_stream.sent_notification(notification_id) {
                info!(LogSchema::new(LogEntry::HandleTerminateRequest)
                    .stream_id(*data_stream_id)
                    .event(LogEvent::Success)
                    .message(&format!(
                        "Terminating the stream that sent notification ID: {:?}. Feedback: {:?}",
                        notification_id, notification_feedback,
                    )));
                data_stream.handle_notification_feedback(notification_id, notification_feedback)?;
                self.data_streams.remove(notification_id);
                return Ok(());
            }
        }

        panic!(
            "Unable to find the stream that sent notification ID: {:?}. Feedback: {:?}",
            notification_id, notification_feedback,
        );
    }

    /// Creates a new stream and ensures the data for that stream is available
    fn process_new_stream_request(
        &mut self,
        request_message: &StreamRequestMessage,
    ) -> Result<DataStreamListener, Error> {
        // Refresh the cached global data summary
        self.refresh_global_data_summary();

        // Create a new data stream
        let stream_id = self.stream_id_generator.fetch_add(1, Ordering::Relaxed);
        let (data_stream, stream_listener) = DataStream::new(
            stream_id,
            &request_message.stream_request,
            self.diem_data_client.clone(),
            self.notification_id_generator.clone(),
            &self.global_data_summary.advertised_data,
        )?;

        // Verify the data stream can be fulfilled using the currently advertised data
        data_stream.ensure_data_is_available(&self.global_data_summary.advertised_data)?;

        // Store the data stream internally
        if self.data_streams.insert(stream_id, data_stream).is_some() {
            panic!(
                "Duplicate data stream found! This should not occur! ID: {:?}",
                stream_id,
            );
        }
        info!(LogSchema::new(LogEntry::HandleStreamRequest)
            .stream_id(stream_id)
            .event(LogEvent::Success)
            .message(&format!(
                "Stream created for request: {:?}",
                request_message
            )));

        // Return the listener
        Ok(stream_listener)
    }

    /// Refreshes the global data summary by communicating with the Diem data client
    fn refresh_global_data_summary(&mut self) {
        if let Err(error) = self.fetch_global_data_summary() {
            error!(LogSchema::new(LogEntry::RefreshGlobalData)
                .event(LogEvent::Error)
                .error(&error));
        }
    }

    fn fetch_global_data_summary(&mut self) -> Result<(), Error> {
        match self.diem_data_client.get_global_data_summary() {
            Ok(data_client_response) => {
                match data_client_response.response_payload {
                    DataClientPayload::GlobalDataSummary(global_data_summary) => {
                        verify_optimal_chunk_sizes(&global_data_summary.optimal_chunk_sizes)?;
                        self.global_data_summary = global_data_summary;
                        Ok(())
                    },
                    result => {
                        Err(Error::DiemDataClientResponseIsInvalid(format!(
                            "Response payload type is incorrect! Expected a global data summary, but got: {:?}",
                            result
                        )))
                    }
                }
            }
            Err(error) => {
                Err(Error::DiemDataClientResponseIsInvalid(format!(
                    "Failed to fetch global data summary! Error: {:?}",
                    error
                )))
            }
        }
    }

    /// Ensures that all existing data streams are making progress
    fn check_progress_of_all_data_streams(&mut self) {
        let data_stream_ids = self.get_all_data_stream_ids();
        for data_stream_id in &data_stream_ids {
            if let Err(error) = self.update_progress_of_data_stream(data_stream_id) {
                error!(LogSchema::new(LogEntry::CheckStreamProgress)
                    .stream_id(*data_stream_id)
                    .event(LogEvent::Error)
                    .error(&error));
            }
        }
    }

    /// Ensures that a data stream has in-flight data requests and handles
    /// any new responses that have arrived since we last checked.
    fn update_progress_of_data_stream(
        &mut self,
        data_stream_id: &DataStreamId,
    ) -> Result<(), Error> {
        let global_data_summary = self.global_data_summary.clone();

        let data_stream = self.get_data_stream(data_stream_id);
        if !data_stream.data_requests_initialized() {
            // Initialize the request batch by sending out data client requests
            data_stream.initialize_data_requests(global_data_summary)?;
            info!(
                (LogSchema::new(LogEntry::InitializeStream)
                    .stream_id(*data_stream_id)
                    .event(LogEvent::Success)
                    .message("Data stream initialized."))
            );
        } else {
            // Process any data client requests that have received responses
            data_stream.process_data_responses(global_data_summary)?;
        }

        Ok(())
    }

    fn get_all_data_stream_ids(&self) -> Vec<DataStreamId> {
        self.data_streams
            .keys()
            .cloned()
            .collect::<Vec<DataStreamId>>()
    }

    /// Returns the data stream associated with the given `data_stream_id`.
    /// Note: this method assumes the caller has already verified the stream exists.
    fn get_data_stream(&mut self, data_stream_id: &DataStreamId) -> &mut DataStream<T> {
        self.data_streams
            .get_mut(data_stream_id)
            .unwrap_or_else(|| {
                panic!(
                    "Expected a data stream with ID: {:?}, but found None!",
                    data_stream_id
                )
            })
    }
}

/// Verifies that all optimal chunk sizes are valid (i.e., not zero). Returns an
/// error if a chunk size is 0.
fn verify_optimal_chunk_sizes(optimal_chunk_sizes: &OptimalChunkSizes) -> Result<(), Error> {
    if optimal_chunk_sizes.account_states_chunk_size == 0
        || optimal_chunk_sizes.epoch_chunk_size == 0
        || optimal_chunk_sizes.transaction_chunk_size == 0
        || optimal_chunk_sizes.transaction_output_chunk_size == 0
    {
        Err(Error::DiemDataClientResponseIsInvalid(format!(
            "Found at least one optimal chunk size of zero: {:?}",
            optimal_chunk_sizes
        )))
    } else {
        Ok(())
    }
}
