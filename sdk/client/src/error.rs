// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use diem_json_rpc_types::errors::JsonRpcError;

pub type Result<T, E = Error> = ::std::result::Result<T, E>;

#[derive(Debug)]
pub struct Error {
    inner: Box<Inner>,
}

type BoxError = Box<dyn std::error::Error + Send + Sync + 'static>;

#[derive(Debug)]
struct Inner {
    kind: Kind,
    source: Option<BoxError>,
    json_rpc_error: Option<JsonRpcError>,
}

#[derive(Debug)]
enum Kind {
    HttpStatus(u16),
    Timeout,
    Request,
    JsonRpcError,
    RpcResponse,
    ChainId,
    StaleResponse,
    Batch,
    Decode,
    Unknown,
}

impl Error {
    pub fn json_rpc_error(&self) -> Option<&JsonRpcError> {
        self.inner.json_rpc_error.as_ref()
    }

    //
    // Private Constructors
    //

    fn new<E: Into<BoxError>>(kind: Kind, source: Option<E>) -> Self {
        Self {
            inner: Box::new(Inner {
                kind,
                source: source.map(Into::into),
                json_rpc_error: None,
            }),
        }
    }

    fn with_json_rpc_error(mut self, json_rpc_error: JsonRpcError) -> Self {
        self.inner.json_rpc_error = Some(json_rpc_error);
        self
    }

    pub(crate) fn status(status: u16) -> Self {
        Self::new(Kind::HttpStatus(status), None::<Error>)
    }

    pub(crate) fn timeout<E: Into<BoxError>>(e: E) -> Self {
        Self::new(Kind::Timeout, Some(e))
    }

    pub(crate) fn json_rpc(json_rpc_error: JsonRpcError) -> Self {
        Self::new(Kind::JsonRpcError, None::<Error>).with_json_rpc_error(json_rpc_error)
    }

    pub(crate) fn rpc_response<E: Into<BoxError>>(e: E) -> Self {
        Self::new(Kind::RpcResponse, Some(e))
    }

    pub(crate) fn batch<E: Into<BoxError>>(e: E) -> Self {
        Self::new(Kind::Batch, Some(e))
    }

    pub(crate) fn decode<E: Into<BoxError>>(e: E) -> Self {
        Self::new(Kind::Decode, Some(e))
    }

    pub(crate) fn unknown<E: Into<BoxError>>(e: E) -> Self {
        Self::new(Kind::Unknown, Some(e))
    }

    pub(crate) fn request<E: Into<BoxError>>(e: E) -> Self {
        Self::new(Kind::Request, Some(e))
    }

    pub(crate) fn chain_id(expected: u8, recieved: u8) -> Self {
        Self::new(
            Kind::ChainId,
            Some(format!("expected: {} recieved: {}", expected, recieved)),
        )
    }

    pub(crate) fn stale(expected: &super::State, recieved: &super::State) -> Self {
        Self::new(
            Kind::StaleResponse,
            Some(format!("expected: {:?} recieved: {:?}", expected, recieved)),
        )
    }

    #[cfg(feature = "async")]
    pub(crate) fn from_reqwest_error(e: reqwest::Error) -> Self {
        if e.is_timeout() {
            Self::timeout(e)
        } else if e.is_request() {
            Self::request(e)
        } else if e.is_decode() {
            Self::decode(e)
        } else {
            Self::unknown(e)
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.inner.source.as_ref().map(|e| &**e as _)
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Self::decode(e)
    }
}
