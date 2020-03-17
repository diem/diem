// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use lcs;
use libra_types::{contract_event, transaction};
use serde_reflection::Tracer;

use proptest::{
    prelude::*,
    test_runner::{Config, FileFailurePersistence, TestRunner},
};
use serde_yaml;
use std::sync::{Arc, Mutex};

fn main() {
    let mut runner = TestRunner::new(Config {
        failure_persistence: Some(Box::new(FileFailurePersistence::Off)),
        ..Config::default()
    });

    let tracer = Arc::new(Mutex::new(Tracer::new(lcs::is_human_readable())));

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

    // Pull a finalized registry out of the Arc-mutex-ed tracer.
    let registry = Arc::try_unwrap(tracer)
        .unwrap()
        .into_inner()
        .unwrap()
        .registry()
        .unwrap();
    let output = serde_yaml::to_string(&registry).unwrap();
    println!("{}", output);
}
