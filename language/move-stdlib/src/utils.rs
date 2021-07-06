// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::{io::Write, time::Instant};

pub fn time_it<F, R>(msg: &str, f: F) -> R
where
    F: FnOnce() -> R,
{
    let now = Instant::now();
    print!("{} ... ", msg);
    let _ = std::io::stdout().flush();
    let res = f();
    println!("(took {:.3}s)", now.elapsed().as_secs_f64());
    res
}
