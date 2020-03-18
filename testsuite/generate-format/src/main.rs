// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use clap::{App, Arg};
use libra_types::{contract_event, transaction};
use proptest::{
    prelude::*,
    test_runner::{Config, FileFailurePersistence, TestRunner},
};
use serde_reflection::Tracer;
use serde_yaml;
use std::sync::{Arc, Mutex};

fn add_proptest_serialization_tracing(tracer: Tracer) -> Tracer {
    let mut runner = TestRunner::new(Config {
        failure_persistence: Some(Box::new(FileFailurePersistence::Off)),
        ..Config::default()
    });

    // Wrap the tracer for (hypothetical) concurrent accesses.
    let tracer = Arc::new(Mutex::new(tracer));

    runner
        .run(&any::<transaction::Transaction>(), |v| {
            tracer.lock().unwrap().trace_value(&v)?;
            Ok(())
        })
        .unwrap();

    runner
        .run(&any::<contract_event::ContractEvent>(), |v| {
            tracer.lock().unwrap().trace_value(&v)?;
            Ok(())
        })
        .unwrap();

    // Recover the Arc-mutex-ed tracer.
    Arc::try_unwrap(tracer).unwrap().into_inner().unwrap()
}

fn add_deserialization_tracing(mut tracer: Tracer) -> Tracer {
    tracer.trace_type::<transaction::Transaction>().unwrap();
    tracer
        .trace_type::<contract_event::ContractEvent>()
        .unwrap();
    tracer
}

fn main() {
    let matches = App::new("Libra format generator")
        .about("Trace serde (de)serialization to generate format descriptions for Libra types")
        .arg(
            Arg::with_name("with_deserialize")
                .long("with_deserialize")
                .help("Trace deserialization instead of serialization"),
        )
        .get_matches();

    let mut tracer = Tracer::new(lcs::is_human_readable());

    tracer = add_proptest_serialization_tracing(tracer);
    if matches.is_present("with_deserialize") {
        tracer = add_deserialization_tracing(tracer);
    }

    let registry = tracer.registry().unwrap();
    let output = serde_yaml::to_string(&registry).unwrap();
    println!("{}", output);
}
