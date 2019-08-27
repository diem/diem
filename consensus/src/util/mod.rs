// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#[cfg(any(test, feature = "fuzzing"))]
pub mod mock_time_service;
pub mod time_service;
#[cfg(test)]
mod time_service_test;
