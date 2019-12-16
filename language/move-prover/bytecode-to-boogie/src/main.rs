// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use bytecode_to_boogie::cli::Options;
use bytecode_to_boogie::driver::Driver;
use std::env;

fn main() {
    let mut options = Options::default();
    let args: Vec<String> = env::args().collect();
    options.initialize_from_args(&args);
    options.setup_logging();
    let mut driver = Driver::new(&options);
    driver.run();
}
