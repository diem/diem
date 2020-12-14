// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use diem_types::transaction;
use move_core_types::language_storage;
use serde_reflection::{Registry, Result, Samples, Tracer, TracerConfig};

/// Default output file.
pub fn output_file() -> Option<&'static str> {
    Some("tests/staged/move_abi.yaml")
}

pub fn get_registry() -> Result<Registry> {
    let mut tracer =
        Tracer::new(TracerConfig::default().is_human_readable(bcs::is_human_readable()));
    let samples = Samples::new();
    // 1. Record samples for types with custom deserializers.

    // 2. Trace the main entry point(s) + every enum separately.
    tracer.trace_type::<transaction::ScriptABI>(&samples)?;
    tracer.trace_type::<language_storage::TypeTag>(&samples)?;
    tracer.registry()
}
