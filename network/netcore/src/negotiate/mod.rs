// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Protocol negotiation on AsyncRead/AsyncWrite streams
//!
//! Upgrading a stream to a particular protocol can be done either using 'protocol-interactive' or
//! 'protocol-select', both of which use u16 length prefix framing.

mod inbound;
mod outbound;
#[cfg(test)]
mod test;

pub use self::{
    inbound::negotiate_inbound,
    outbound::{negotiate_outbound_interactive, negotiate_outbound_select},
};

static PROTOCOL_INTERACTIVE: &[u8] = b"/libra/protocol-interactive/1.0.0";
static PROTOCOL_SELECT: &[u8] = b"/libra/protocol-select/1.0.0";
static PROTOCOL_NOT_SUPPORTED: &[u8] = b"not supported";
