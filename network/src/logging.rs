// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    peer_manager::{ConnectionNotification, ConnectionRequest, PeerManagerRequest},
    ConnectivityRequest,
};
use libra_config::network_id::NetworkContext;
use libra_logger::LoggingField;
use libra_types::PeerId;

/// This file contains constants used for structured logging data types so that there is
/// consistency among structured logs.
pub const CONNECTIVITY_MANAGER_LOOP: &str = "connectivity_manager_loop";
pub const ONCHAIN_DISCOVERY_LOOP: &str = "onchain_discovery_loop";
pub const PEER_MANAGER_LOOP: &str = "peer_manager_loop";

/// Common terms
pub const TYPE: &str = "type";
pub const START: &str = "start";
pub const TERMINATION: &str = "termination";
pub const EVENT: &str = "event";

/// Specific fields for logging
pub const NETWORK_CONTEXT: LoggingField<&NetworkContext> = LoggingField::new("network_context");
pub const EVENT_ID: LoggingField<&u32> = LoggingField::new("event_id");
pub const REMOTE_PEER: LoggingField<&PeerId> = LoggingField::new("remote_peer");
pub const CONNECTION_NOTIFICATION: LoggingField<&ConnectionNotification> =
    LoggingField::new("conn_notification");
pub const CONNECTIVITY_REQUEST: LoggingField<&ConnectivityRequest> =
    LoggingField::new("connectivity_request");
pub const CONNECTION_REQUEST: LoggingField<&ConnectionRequest> =
    LoggingField::new("connection_request");
pub const PEER_MANAGER_REQUEST: LoggingField<&PeerManagerRequest> =
    LoggingField::new("peer_manager_request");
