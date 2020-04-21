// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_types::{contract_event, transaction};
use proptest::{
    prelude::*,
    test_runner::{Config, FileFailurePersistence, TestRunner},
};
use serde_reflection::{Registry, Samples, Tracer, TracerConfig};
use std::sync::{Arc, Mutex};

/// Default output file.
pub fn output_file() -> Option<&'static str> {
    Some("tests/staged/libra.yaml")
}

pub fn get_registry(_name: String, skip_deserialize: bool) -> Registry {
    let (mut tracer, samples) = proptest_serialization_tracing();
    if !skip_deserialize {
        deserialization_tracing(&mut tracer, &samples);
    }
    tracer.registry().unwrap()
}

/// Which Libra values to record with the serialization tracing API.
///
/// This step is useful to inject well-formed values that must pass
/// custom-validation checks (e.g. keys).
fn proptest_serialization_tracing() -> (Tracer, Samples) {
    let mut runner = TestRunner::new(Config {
        failure_persistence: Some(Box::new(FileFailurePersistence::Off)),
        ..Config::default()
    });

    let tracer = Arc::new(Mutex::new(Tracer::new(
        TracerConfig::default().is_human_readable(lcs::is_human_readable()),
    )));
    let samples = Arc::new(Mutex::new(Samples::new()));

    runner
        .run(&any::<transaction::Transaction>(), |v| {
            tracer
                .lock()
                .unwrap()
                .trace_value(&mut samples.lock().unwrap(), &v)?;
            Ok(())
        })
        .unwrap();

    runner
        .run(&any::<contract_event::ContractEvent>(), |v| {
            tracer
                .lock()
                .unwrap()
                .trace_value(&mut samples.lock().unwrap(), &v)?;
            Ok(())
        })
        .unwrap();

    // Recover the Arc-mutex-ed tracer.
    (
        Arc::try_unwrap(tracer).unwrap().into_inner().unwrap(),
        Arc::try_unwrap(samples).unwrap().into_inner().unwrap(),
    )
}

/// Which Libra types to record with the deserialization tracing API.
///
/// This step is useful to guarantee coverage of the analysis but it may
/// fail if the previous step missed some custom types.
fn deserialization_tracing(tracer: &mut Tracer, samples: &Samples) {
    tracer
        .trace_type::<transaction::Transaction>(&samples)
        .unwrap();
    tracer
        .trace_type::<contract_event::ContractEvent>(&samples)
        .unwrap();
}
