// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::ProtocolId;
use bytes::Bytes;
use serde::Serialize;
use std::fmt::Debug;

// TODO(philiphayes): just use wire::DirectSendMsg directly

#[derive(Clone, Eq, PartialEq, Serialize)]
pub struct Message {
    /// Message type.
    pub protocol_id: ProtocolId,
    /// Serialized message data.
    #[serde(skip)]
    pub mdata: Bytes,
}

impl Debug for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mdata_str = if self.mdata.len() <= 10 {
            format!("{:?}", self.mdata)
        } else {
            format!("{:?}...", self.mdata.slice(..10))
        };
        write!(
            f,
            "Message {{ protocol: {:?}, mdata: {} }}",
            self.protocol_id, mdata_str
        )
    }
}
