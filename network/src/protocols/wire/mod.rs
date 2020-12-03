// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module defines the structs transported during the DiemNet handshake protocol and
//! the DiemNet messaging protocol.
//! The handshake protocol is executed prior to executing the messaging protocol, and is used to
//! determine the version of messaging protocol to use. Each node only supports one version of the
//! handshake protocol on an end-point, and that is advertised as part of its discovery
//! NetworkAddress.

pub mod handshake;
pub mod messaging;
