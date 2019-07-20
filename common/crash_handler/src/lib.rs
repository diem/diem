// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use backtrace::Backtrace;
use logger::prelude::*;
use serde::Serialize;
use std::{
    panic::{self, PanicInfo},
    process, thread, time,
};

#[derive(Debug, Serialize)]
pub struct CrashInfo {
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
    // The Display formatter for a PanicInfo contains the message, payload and location.
    let details = format!("{}", panic_info);
    let backtrace = format!("{:#?}", Backtrace::new());

    let info = CrashInfo { details, backtrace };
    crit!("{}", toml::to_string_pretty(&info).unwrap());

    // allow to save on disk
    thread::sleep(time::Duration::from_millis(100));

    // kill the process
    process::exit(12);
}
