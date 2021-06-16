// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#[cfg(debug_assertions)]
use crate::debug::DebugContext;

#[cfg(debug_assertions)]
use ::{
    move_binary_format::file_format::Bytecode,
    move_vm_types::values::Locals,
    once_cell::sync::Lazy,
    std::{
        env,
        fs::{File, OpenOptions},
        io::Write,
        process,
        sync::Mutex,
        thread,
    },
};

#[cfg(debug_assertions)]
use crate::{
    interpreter::Interpreter,
    loader::{Function, Loader},
};

#[cfg(debug_assertions)]
const MOVE_VM_TRACING_ENV_VAR_NAME: &str = "MOVE_VM_TRACE";

#[cfg(debug_assertions)]
const MOVE_VM_STEPPING_ENV_VAR_NAME: &str = "MOVE_VM_STEP";

#[cfg(debug_assertions)]
static FILE_PATH: Lazy<String> = Lazy::new(|| {
    env::var(MOVE_VM_TRACING_ENV_VAR_NAME).unwrap_or_else(|_| "move_vm_trace.trace".to_string())
});

#[cfg(debug_assertions)]
static TRACING_ENABLED: Lazy<bool> = Lazy::new(|| env::var(MOVE_VM_TRACING_ENV_VAR_NAME).is_ok());

#[cfg(debug_assertions)]
static DEBUGGING_ENABLED: Lazy<bool> =
    Lazy::new(|| env::var(MOVE_VM_STEPPING_ENV_VAR_NAME).is_ok());

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

#[cfg(debug_assertions)]
static DEBUG_CONTEXT: Lazy<Mutex<DebugContext>> = Lazy::new(|| Mutex::new(DebugContext::new()));

// Only include in debug builds
#[cfg(debug_assertions)]
pub(crate) fn trace(
    function_desc: &Function,
    locals: &Locals,
    pc: u16,
    instr: &Bytecode,
    loader: &Loader,
    interp: &Interpreter,
) {
    if *TRACING_ENABLED {
        let f = &mut *LOGGING_FILE.lock().unwrap();
        writeln!(
            f,
            "{}-{:?},{},{},{:?}",
            process::id(),
            thread::current().id(),
            function_desc.pretty_string(),
            pc,
            instr,
        )
        .unwrap();
    }
    if *DEBUGGING_ENABLED {
        DEBUG_CONTEXT
            .lock()
            .unwrap()
            .debug_loop(function_desc, locals, pc, instr, loader, interp);
    }
}

#[macro_export]
macro_rules! trace {
    ($function_desc:expr, $locals:expr, $pc:expr, $instr:tt, $resolver:expr, $interp:expr) => {
        // Only include this code in debug releases
        #[cfg(debug_assertions)]
        crate::tracing::trace(
            &$function_desc,
            $locals,
            $pc,
            &$instr,
            $resolver.loader(),
            $interp,
        )
    };
}
