// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use serde_reflection::{Registry, Samples, Tracer, TracerConfig};
use vm::file_format as ff;

pub fn output_file() -> Option<&'static str> {
    None
}

pub fn get_registry(_name: String, _skip_deserialize: bool) -> Registry {
    let mut tracer = Tracer::new(TracerConfig::default());
    let samples = Samples::new();
    tracer.trace_type::<ff::SignatureToken>(&samples).unwrap();
    tracer.trace_type::<ff::Bytecode>(&samples).unwrap();
    tracer.trace_type::<ff::Kind>(&samples).unwrap();
    tracer.trace_type::<ff::CompiledScript>(&samples).unwrap();
    tracer.registry().unwrap()
}
