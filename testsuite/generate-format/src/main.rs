// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use clap::{App, Arg};
use libra_types::{contract_event, transaction};
use proptest::prelude::*;
use proptest::test_runner::{Config, FileFailurePersistence, TestRunner};
use serde_reflection::{DTracer, Format, Tracer};
use serde_yaml;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

fn registry_from_serialize() -> BTreeMap<&'static str, Format> {
    let mut runner = TestRunner::new(Config {
        failure_persistence: Some(Box::new(FileFailurePersistence::Off)),
        ..Config::default()
    });

    let tracer = Arc::new(Mutex::new(Tracer::new(lcs::is_human_readable())));

    runner
        .run(&any::<transaction::Transaction>(), |v| {
            tracer.lock().unwrap().trace(&v)?;
            Ok(())
        })
        .unwrap();

    runner
        .run(&any::<contract_event::ContractEvent>(), |v| {
            tracer.lock().unwrap().trace(&v)?;
            Ok(())
        })
        .unwrap();

    // Pull a finalized registry out of the Arc-mutex-ed tracer.
    Arc::try_unwrap(tracer)
        .unwrap()
        .into_inner()
        .unwrap()
        .registry()
        .unwrap()
}

fn registry_from_deserialize() -> BTreeMap<&'static str, Format> {
    let mut tracer = DTracer::new(lcs::is_human_readable());
    tracer.trace::<transaction::Transaction>().unwrap();
    tracer.trace::<contract_event::ContractEvent>().unwrap();

    tracer.registry().unwrap()
}

fn main() {
    let matches = App::new("Libra format generator")
        .about("Trace serde (de)serialization to generate format descriptions for Libra types")
        .arg(
            Arg::with_name("deserialize")
                .long("deserialize")
                .help("Trace deserialization instead of serialization"),
        )
        .get_matches();

    let registry = if matches.is_present("deserialize") {
        registry_from_deserialize()
    } else {
        registry_from_serialize()
    };
    let output = serde_yaml::to_string(&registry).unwrap();
    println!("{}", output);
}
