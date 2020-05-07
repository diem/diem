// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#[cfg(debug_assertions)]
use once_cell::sync::Lazy;
#[cfg(debug_assertions)]
use std::{
    env,
    fs::{File, OpenOptions},
    io::Write,
    sync::Mutex,
};
#[cfg(debug_assertions)]
use vm::file_format::Bytecode;

#[cfg(debug_assertions)]
const MOVE_VM_TRACING_ENV_VAR_NAME: &str = "MOVE_VM_TRACE";

#[cfg(debug_assertions)]
static FILE_PATH: Lazy<String> = Lazy::new(|| {
    env::var(MOVE_VM_TRACING_ENV_VAR_NAME).unwrap_or_else(|_| "move_vm_trace.trace".to_string())
});

#[cfg(debug_assertions)]
static TRACING_ENABLED: Lazy<bool> = Lazy::new(|| env::var(MOVE_VM_TRACING_ENV_VAR_NAME).is_ok());

#[cfg(debug_assertions)]
static LOGGING_FILE: Lazy<Mutex<File>> = Lazy::new(|| {
    Mutex::new(
        OpenOptions::new()
            .write(true)
            .create(true)
            .append(true)
            .open(&*FILE_PATH)
            .unwrap(),
    )
});

// Only include in debug builds
#[cfg(debug_assertions)]
pub fn trace(function_desc: &str, pc: u16, instr: &Bytecode) {
    if *TRACING_ENABLED {
        let f = &mut *LOGGING_FILE.lock().unwrap();
        writeln!(f, "{},{},{:?}", function_desc, pc, instr).unwrap();
    }
}

#[macro_export]
macro_rules! trace {
    ($function_desc:expr, $pc:expr, $instr:tt) => {
        // Only include this code in debug releases
        #[cfg(debug_assertions)]
        crate::tracing::trace(&$function_desc, $pc, &$instr)
    };
}
