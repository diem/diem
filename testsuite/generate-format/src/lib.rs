// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

/// How to record the format of interesting Libra types.
/// See API documentation with `cargo doc -p serde-reflection --open`
use libra_types::{contract_event, transaction};
use proptest::{
    prelude::*,
    test_runner::{Config, FileFailurePersistence, TestRunner},
};
use serde_reflection::{SerializationRecords, Tracer};
use std::sync::{Arc, Mutex};

/// Default output file.
pub static FILE_PATH: &str = "tests/staged/libra.yaml";

/// Which Libra values to record with the serialization tracing API.
///
/// This step is useful to inject well-formed values that must pass
/// custom-validation checks (e.g. keys).
pub fn add_proptest_serialization_tracing(
    tracer: Tracer,
    records: SerializationRecords,
) -> (Tracer, SerializationRecords) {
    let mut runner = TestRunner::new(Config {
        failure_persistence: Some(Box::new(FileFailurePersistence::Off)),
        ..Config::default()
    });

    // Wrap the tracer for (hypothetical) concurrent accesses.
    let tracer = Arc::new(Mutex::new(tracer));
    let records = Arc::new(Mutex::new(records));

    runner
        .run(&any::<transaction::Transaction>(), |v| {
            tracer
                .lock()
                .unwrap()
                .trace_value(&mut records.lock().unwrap(), &v)?;
            Ok(())
        })
        .unwrap();

    runner
        .run(&any::<contract_event::ContractEvent>(), |v| {
            tracer
                .lock()
                .unwrap()
                .trace_value(&mut records.lock().unwrap(), &v)?;
            Ok(())
        })
        .unwrap();

    // Recover the Arc-mutex-ed tracer.
    (
        Arc::try_unwrap(tracer).unwrap().into_inner().unwrap(),
        Arc::try_unwrap(records).unwrap().into_inner().unwrap(),
    )
}

/// Which Libra types to record with the deserialization tracing API.
///
/// This step is useful to guarantee coverage of the analysis but it may
/// fail if the previous step missed some custom types.
pub fn add_deserialization_tracing(mut tracer: Tracer, records: &SerializationRecords) -> Tracer {
    tracer
        .trace_type::<transaction::Transaction>(&records)
        .unwrap();
    tracer
        .trace_type::<contract_event::ContractEvent>(&records)
        .unwrap();
    tracer
}
