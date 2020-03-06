// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Names of modules and functions that are somehow special to the Move VM.

use move_core_types::identifier::Identifier;
use once_cell::sync::Lazy;

pub(crate) static EMIT_EVENT_NAME: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("write_to_event_store").unwrap());

pub(crate) static SAVE_ACCOUNT_NAME: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("save_account").unwrap());
