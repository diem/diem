// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use move_command_line_common::files::{MOVE_EXTENSION, MOVE_IR_EXTENSION};
use move_lang_test_utils::*;
use std::{collections::HashSet, path::Path};

//#[test]
#[allow(dead_code)]
fn test_ir_test_coverage() {
    for completed_directory in COMPLETED_DIRECTORIES {
        let dir = format!("{}/{}", PATH_TO_IR_TESTS, completed_directory);
        let p = Path::new(&dir);
        if !p.is_dir() {
            panic!("Invalid completed directory. '{}' does not exist", dir)
        }
    }
    let completed_directories: HashSet<String> = COMPLETED_DIRECTORIES
        .iter()
        .map(|s| (*s).to_owned())
        .collect();

    let not_migrated = ir_tests()
        .filter(|(subdir, name)| {
            completed_directories.contains(subdir) && !translated_test_exists(subdir, name)
        })
        .map(|(subdir, name)| format!("{}/{}/{}", PATH_TO_IR_TESTS, subdir, name))
        .collect::<Vec<_>>();
    if !not_migrated.is_empty() {
        let mut msg = "\n\nThe following tests have not been migrated:\n".to_owned();
        for path in not_migrated {
            use std::fmt::Write as _;
            let _ = writeln!(msg, "{}", path);
        }
        msg.push_str("\nA corresponding test needs to be added:\n");
        use std::fmt::Write as _;
        let _ = writeln!(
            msg,
            "    {}/<dir name>/<test name>.{}",
            MOVE_CHECK_DIR, MOVE_EXTENSION
        );
        msg.push_str("  or\n");
        let _ = writeln!(
            msg,
            "    {}/<dir name>/<test name>.{}",
            FUNCTIONAL_TEST_DIR, MOVE_EXTENSION
        );
        let _ =
            write!(msg,
            "Replace the extension '.{}' with '.{}' to mark the test as present, but it will not \
             be run.\n\n",
            MOVE_EXTENSION, TODO_EXTENSION
        );
        msg.push_str("Running the following tool may help with the migration:\n");
        msg.push_str(
            "  cargo run -p move-lang-ir-utils --bin ir-test-translation -- -d <dir_name>\n\n",
        );
        panic!("{}", msg)
    }
}

fn translated_test_exists(subdir: &str, name_str: &str) -> bool {
    let mut stem = name_str.to_owned();
    (0..=MOVE_IR_EXTENSION.len()).for_each(|_| {
        stem.pop().unwrap();
    });
    let stem_str = &stem;
    translated_ir_test_name(false, subdir, stem_str).is_none()
}
