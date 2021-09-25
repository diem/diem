// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use regex::Regex;
use std::{
    collections::BTreeMap,
    fs,
    fs::read_to_string,
    io::Write,
    iter::FromIterator,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    str::FromStr,
};

// Gather test files from the test directory
fn gather_tests(directory_path: PathBuf) -> Vec<String> {
    let mut test_files = Vec::new();
    let error_msg = format!("Failed to read test directory {:?}", directory_path);
    for entry in fs::read_dir(directory_path).unwrap_or_else(|_| panic!("{}", error_msg)) {
        let entry = entry.unwrap();
        if !entry.file_type().unwrap().is_file() {
            continue;
        };
        let file_path = entry.path();
        test_files.push(file_path.into_os_string().into_string().unwrap());
    }
    test_files
}

// Parse expected or actual output into a map
// from trimmed non-empty lines to counts.
fn build_output_counter(output: &str) -> BTreeMap<&str, u32> {
    let items: Vec<&str> = Vec::from_iter(
        output
            .split('\n')
            .collect::<Vec<&str>>()
            .iter()
            .filter(|x| !x.is_empty())
            .map(|x| x.trim()),
    );
    let mut counter = BTreeMap::<&str, u32>::new();
    for item in items.iter() {
        *counter.entry(item).or_insert(0) += 1;
    }
    counter
}

// Two outputs are considered equivalent if they
// have the same lines (and counts of each line),
// order-independent.
fn compare_lines(actual: &str, expected: &str) -> bool {
    let actual_counter = build_output_counter(actual);
    let expected_counter = build_output_counter(expected);
    actual_counter == expected_counter
}

// Run the Datalog analysis on the test file and
// capture output
fn run_analysis(file_name: &str) -> Result<String, ()> {
    let file_data = read_to_string(file_name).expect("Test case should be readable");
    let mut child = Command::new("analyses/ddlog_ddlog/target/release/ddlog_cli")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to execute process");
    child
        .stdin
        .take()
        .unwrap()
        .write_all(file_data.as_bytes())
        .unwrap();
    match child.wait_with_output() {
        Ok(output) => Ok((String::from_utf8_lossy(&output.stdout)
            + String::from_utf8_lossy(&output.stderr))
        .to_string()),
        Err(_) => Err(()),
    }
}

// Check the call graph output files against
// the expected output from the test case file.
fn execute_test(file_name: &str) -> usize {
    let test_case_data =
        fs::read_to_string(Path::new(&file_name)).expect("Failed to read test case");
    // Check that the expected and actual output files match
    let mut expected = String::new();
    let expected_regex = Regex::new(r"(# expect) (.*)").unwrap();
    for capture in expected_regex.captures_iter(&test_case_data) {
        if let Some(c) = capture.get(2) {
            expected.push_str(c.as_str());
            expected.push('\n')
        }
    }
    let actual = run_analysis(file_name);
    if let Ok(actual) = actual {
        if compare_lines(&expected, &actual) {
            0
        } else {
            println!("{} failed", file_name);
            println!("Expected:\n{}", expected);
            println!("Actual:\n{}", actual);
            1
        }
    } else {
        println!("{} failed", file_name);
        1
    }
}

// Run the tests in the ddlog_tests directory
// This test is ignored by default because it requires manual setup
// of Differential Datalog
#[test]
#[ignore]
fn run_tests() {
    let test_path = PathBuf::from_str("./tests/ddlog_tests/").unwrap();
    println!("{:?}", std::fs::canonicalize(test_path.to_owned()));
    let files = gather_tests(test_path);
    let result = files.iter().fold(0, |acc, file| acc + execute_test(file));
    assert_eq!(result, 0);
}
