// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![feature(panic_info_message)]

use backtrace::Backtrace;
use logger::prelude::*;
use serde::Serialize;
use std::{
    panic::{self, PanicInfo},
    process, thread, time,
};

#[derive(Debug, Serialize)]
pub struct CrashInfo {
    reason: String,
    details: String,
    backtrace: String,
}

pub fn setup_panic_handler() {
    // If RUST_BACKTRACE variable isn't present or RUST_BACKTRACE=0, we setup panic handler
    let is_backtrace_set = std::env::var_os("RUST_BACKTRACE")
        .map(|x| &x != "0")
        .unwrap_or(false);

    if is_backtrace_set {
        info!("Skip panic handler setup because RUST_BACKTRACE is set");
    } else {
        panic::set_hook(Box::new(move |pi: &PanicInfo<'_>| {
            handle_panic(pi);
        }));
    }
}

// formats and logs panic information
fn handle_panic(panic_info: &PanicInfo<'_>) {
    let reason = match panic_info.message() {
        Some(m) => format!("{}", m),
        None => "Unknown Reason".into(),
    };

    let mut details = String::new();

    let payload = match panic_info.payload().downcast_ref::<&str>() {
        Some(pld) => format!("Details: {}. ", pld),
        None => "[no extra details]. ".into(),
    };
    details.push_str(&payload);

    let location = match panic_info.location() {
        Some(loc) => format!(
            "Thread panicked at file '{}' at line {}",
            loc.file(),
            loc.line()
        ),
        None => "[no location details].".into(),
    };
    details.push_str(&location);

    let backtrace = format!("{:#?}", Backtrace::new());

    let info = CrashInfo {
        reason,
        details,
        backtrace,
    };
    crit!("{}", toml::to_string_pretty(&info).unwrap());

    // allow to save on disk
    thread::sleep(time::Duration::from_millis(100));

    // kill the process
    process::exit(12);
}
