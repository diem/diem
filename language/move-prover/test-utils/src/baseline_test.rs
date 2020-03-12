// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! A module supporting baseline (golden) tests.

use crate::read_bool_env_var;
use anyhow::{anyhow, Context, Result};
use prettydiff::{basic::DiffOp, diff_lines};
use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
};

/// Verifies or updates baseline file for the given generated text.
pub fn verify_or_update_baseline(baseline_file_name: &Path, text: &str) -> Result<()> {
    let update_baseline = read_bool_env_var("UPBL");

    if update_baseline {
        // Just update the baseline file.
        let mut file = File::create(baseline_file_name)?;
        write!(file, "{}", clean_for_baseline(text))?;
        Ok(())
    } else {
        // Read the baseline and diff it.
        let mut contents = String::new();
        let mut file = File::open(baseline_file_name)
            .with_context(||
                format!("Cannot read baseline file at `{}`. To create a new baseline, call this test with env var UPBL=1.", baseline_file_name.to_string_lossy()))?;
        file.read_to_string(&mut contents)?;
        diff(text, &contents)
    }
}

/// Clean a content to be usable as a baseline file. Currently, we ensure there are no
/// trailing whitespaces and no empty last line, because this is required by git-checks.sh.
fn clean_for_baseline(content: &str) -> String {
    let mut res = String::new();
    for line in content.lines() {
        res.push_str(line.trim_end());
        res.push_str("\n");
    }
    res = res.trim_end().to_string(); // removes empty lines at end
    res.push_str("\n"); // adds back a single newline
    res
}

/// Diffs old and new content.
fn diff(old_content: &str, new_content: &str) -> Result<()> {
    if old_content.trim() == new_content.trim() {
        return Ok(());
    }

    let print_lines = |result: &mut Vec<String>, lines: &[&str], prefix: &str| {
        for line in lines {
            result.push(format!("{}{}", prefix, line));
        }
    };

    let print_context = |result: &mut Vec<String>, lines: &[&str]| {
        if lines.len() <= 3 {
            print_lines(result, lines, "= ");
        } else {
            print_lines(result, &lines[..1], "= ");
            result.push(format!("= ... ({} lines)", lines.len() - 2));
            print_lines(result, &lines[lines.len() - 1..], "= ");
        }
    };

    let diff = diff_lines(&new_content, &old_content);
    let mut result = vec![];
    result.push(
        "
New output differs from baseline!
Call this test with env variable UPBL=1 to regenerate baseline files.
Then use your favorite changelist diff tool to verify you are good with the changes.

Or check the rudimentary diff below:
"
        .to_string(),
    );
    for d in diff.diff() {
        match d {
            DiffOp::Equal(lines) => print_context(&mut result, lines),
            DiffOp::Insert(lines) => print_lines(&mut result, lines, "+ "),
            DiffOp::Remove(lines) => print_lines(&mut result, lines, "- "),
            DiffOp::Replace(old, new) => {
                print_lines(&mut result, old, "- ");
                print_lines(&mut result, new, "+ ");
            }
        }
    }
    Err(anyhow!(result.join("\n")))
}
