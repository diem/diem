// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use futures::channel::{mpsc::SendError, oneshot::Canceled};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Debug, Deserialize, Error, PartialEq, Serialize)]
pub enum Error {
    #[error("The requested data is unavailable and cannot be found in the network! Error: {0}")]
    DataIsUnavailable(String),
    #[error("Error returned by the diem data client: {0}")]
    DiemDataClientError(String),
    #[error("The response from the diem data client is invalid! Error: {0}")]
    DiemDataClientResponseIsInvalid(String),
    #[error("An integer overflow has occurred: {0}")]
    IntegerOverflow(String),
    #[error("No data to fetch: {0}")]
    NoDataToFetch(String),
    #[error("Unexpected error encountered: {0}")]
    UnexpectedErrorEncountered(String),
    #[error("Encountered an unsupported request: {0}")]
    UnsupportedRequestEncountered(String),
}

impl From<diem_data_client::Error> for Error {
    fn from(error: diem_data_client::Error) -> Self {
        Error::DiemDataClientError(error.to_string())
    }
}

impl From<SendError> for Error {
    fn from(error: SendError) -> Self {
        Error::UnexpectedErrorEncountered(error.to_string())
    }
}

impl From<Canceled> for Error {
    fn from(canceled: Canceled) -> Self {
        Error::UnexpectedErrorEncountered(canceled.to_string())
    }
}
