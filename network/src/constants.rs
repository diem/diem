// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

/// A collection of constants and default values for configuring various network components.

// NB: Almost all of these values are educated guesses, and not determined using any empirical
// data. If you run into a limit and believe that it is unreasonably tight, please submit a PR
// with your use-case. If you do change a value, please add a comment linking to the PR which
// advocated the change.
/// The timeout for any inbound RPC call before it's cut off
pub const INBOUND_RPC_TIMEOUT_MS: u64 = 10_000;
/// Limit on concurrent Outbound RPC requests before backpressure is applied
pub const MAX_CONCURRENT_OUTBOUND_RPCS: u32 = 100;
/// Limit on concurrent Inbound RPC requests before backpressure is applied
pub const MAX_CONCURRENT_INBOUND_RPCS: u32 = 100;

// These are only used in tests
// TODO: Fix this so the tests and the defaults in config are the same
pub const NETWORK_CHANNEL_SIZE: usize = 1024;
pub const MAX_FRAME_SIZE: usize = 8 * 1024 * 1024; /* 8 MiB */
pub const MAX_CONCURRENT_NETWORK_REQS: usize = 100;
pub const MAX_CONCURRENT_NETWORK_NOTIFS: usize = 100;
