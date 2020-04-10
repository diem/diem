// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::path::Path;

pub struct StringError(String);

pub const SENDER: &str = "0x8675309";

pub const STD_LIB_DIR: &str = "../stdlib/modules";
pub const FUNCTIONAL_TEST_DIR: &str = "tests/functional";
pub const MOVE_CHECK_DIR: &str = "tests/move_check";
pub const STD_LIB_TRANSACTION_SCRIPTS_DIR: &str = "../stdlib/transaction_scripts";
pub const PATH_TO_IR_TESTS: &str = "../ir-testsuite/tests";

pub const MIGRATION_SUB_DIR: &str = "translated_ir_tests";
pub const TODO_EXTENSION: &str = "move_TODO";
pub const MOVE_EXTENSION: &str = "move";
pub const IR_EXTENSION: &str = "mvir";

pub const COMPLETED_DIRECTORIES: &[&str; 3] =
    &["borrow_tests", "commands", "generics/instantiation_loops"];

/// We need to replicate the specification of the (non-staged) stdlib files here since we can't
/// import the stdlib crate: it will create a circular dependency since the stdlib needs the
/// compiler to compile the stdlib and scripts.
pub fn stdlib_files() -> Vec<String> {
    let dirfiles = datatest_stable::utils::iterate_directory(Path::new(STD_LIB_DIR));
    dirfiles
        .flat_map(|path| {
            if path.extension()?.to_str()? == MOVE_EXTENSION {
                path.into_os_string().into_string().ok()
            } else {
                None
            }
        })
        .collect()
}

impl std::fmt::Display for StringError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.0)
    }
}

impl std::fmt::Debug for StringError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.0)
    }
}

impl std::error::Error for StringError {
    fn description(&self) -> &str {
        &self.0
    }
}

pub fn read_env_var(v: &str) -> String {
    std::env::var(v).unwrap_or_else(|_| "".into())
}

pub fn read_bool_var(v: &str) -> bool {
    let val = read_env_var(v);
    val == "1" || val == "true"
}

pub fn error(s: String) -> datatest_stable::Result<()> {
    Err(Box::new(StringError(s)))
}

//**************************************************************************************************
// IR Test Translation
//**************************************************************************************************

pub fn ir_tests() -> impl Iterator<Item = (String, String)> {
    macro_rules! comp_to_string {
        ($comp_opt:expr) => {{
            $comp_opt.as_os_str().to_str()?
        }};
    }
    let num_root_components = Path::new(PATH_TO_IR_TESTS)
        .canonicalize()
        .unwrap()
        .components()
        .map(|_| 1)
        .sum();
    datatest_stable::utils::iterate_directory(Path::new(PATH_TO_IR_TESTS)).flat_map(move |path| {
        if path.extension()?.to_str()? != IR_EXTENSION {
            return None;
        }
        let pathbuf = path.canonicalize().ok()?;
        let mut components = pathbuf.components();
        // skip over the components pointing to the IR test dir
        for _ in 0..num_root_components {
            components.next();
        }
        // iterate over the components starting with the file name
        let mut components = components.rev();
        let name = comp_to_string!(components.next().unwrap()).to_owned();
        // Combine the other components into one single string
        // These components represet the subdir path under the IR test directory. For migration
        // purposes, consider all of these as a single subdir
        let mut dir = String::new();
        for comp in components {
            let sep = if dir.is_empty() { "" } else { "/" };
            dir = format!("{}{}{}", comp_to_string!(comp), sep, dir)
        }
        Some((dir, name))
    })
}

pub fn translated_ir_test_name(has_main: bool, subdir: &str, name: &str) -> Option<String> {
    let fmt = |dir, subdir, basename, ext| {
        format!(
            "{}/{}/{}/{}.{}",
            dir, MIGRATION_SUB_DIR, subdir, basename, ext
        )
    };
    let check = |x| Path::new(x).is_file();
    let ft = fmt(FUNCTIONAL_TEST_DIR, subdir, name, MOVE_EXTENSION);
    let ft_todo = fmt(FUNCTIONAL_TEST_DIR, subdir, name, TODO_EXTENSION);
    let mc = fmt(MOVE_CHECK_DIR, subdir, name, MOVE_EXTENSION);
    let mc_todo = fmt(MOVE_CHECK_DIR, subdir, name, TODO_EXTENSION);
    if check(&ft) || check(&ft_todo) || check(&mc) || check(&mc_todo) {
        None
    } else if has_main {
        Some(ft)
    } else {
        Some(mc)
    }
}
