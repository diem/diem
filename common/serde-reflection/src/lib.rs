// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

//! # Serde Reflection (experimental)
//!
//! This crate provides a way to extract IDL-like format descriptions for values
//! that implement the Serialize trait of Serde.

mod de;
mod error;
mod format;
mod trace;

pub use de::Tracer as DTracer;
pub use error::{Error, Result};
pub use format::{Format, Named, VariantFormat};
pub use trace::Tracer;
