// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_types::{contract_event, transaction};
use proptest::{
    prelude::*,
    test_runner::{Config, FileFailurePersistence, TestRunner},
};
use serde_reflection::Tracer;
use serde_yaml;
use std::sync::{Arc, Mutex};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "Libra format generator",
    about = "Trace serde (de)serialization to generate format descriptions for Libra types"
)]
struct Options {
    #[structopt(short, long)]
    with_deserialize: bool,
}

fn main() {
    let options = Options::from_args();

    let mut tracer = Tracer::new(lcs::is_human_readable());
    tracer = add_proptest_serialization_tracing(tracer);
    if options.with_deserialize {
        tracer = add_deserialization_tracing(tracer);
    }

    let registry = tracer.registry().unwrap();
    let output = serde_yaml::to_string(&registry).unwrap();
    println!("{}", output);
}

// Below constitutes the tool's knowledge of the interesting Libra types to analyze.

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
