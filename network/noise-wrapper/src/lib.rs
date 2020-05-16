// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! [Noise protocol framework][noise] support for use in Libra.
//!
//! This crate implements wrappers around our implementation of Noise IK.
//!
//! For the handshake, we already know in advance what length the messages are,
//! but post-handshake noise messages can be of variable length.
//! For this reason, post-handshake noise messages need to be prefixed with a
//! 2-byte length field.
//! The [`NoiseSocket`](crate::socket::NoiseSocket) module handles this logic
//! when reading and writing post-handshake Noise messages
//! to a socket.
//!
//! [noise]: http://noiseprotocol.org/

pub mod handshake;
pub mod socket;