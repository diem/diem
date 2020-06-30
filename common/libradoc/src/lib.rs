// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use once_cell::sync::Lazy;
use regex::Regex;
use std::{collections::BTreeMap, io::BufRead};

/// Replace the markdown content in `reader` and return a new string, where some of the Rust quotes
/// have been updated to use the latest definitions.
#[allow(clippy::while_let_on_iterator)]
pub fn update_rust_quotes<R>(
    reader: R,
    definitions: &BTreeMap<String, String>,
) -> std::io::Result<String>
where
    R: BufRead,
{
    let mut result = String::new();
    let mut lines = reader.lines();

    while let Some(line) = lines.next() {
        let line = line?;
        result += &line;
        result += "\n";
        // Copying line until we find a command.
        if let Some(name) = match_begin_command(&line) {
            match definitions.get(&name) {
                Some(content) => {
                    eprintln!("[*] Replacing quote for {}", name);
                    result += "```rust\n";
                    result += content;
                    result += "```\n";

                    // Skipping the rest of the quote.
                    while let Some(line) = lines.next() {
                        let line = line?;
                        if match_end_command(&line) {
                            result += &line;
                            result += "\n";
                            break;
                        }
                    }
                }
                None => {
                    eprintln!(
                        "[-] No definition available for {}. Leaving quote untouched",
                        name
                    );
                }
            }
        }
    }

    Ok(result)
}

static BEGIN_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"<!-- @begin-libradoc name=([^ ]*) -->").unwrap());

fn match_begin_command(line: &str) -> Option<String> {
    match BEGIN_RE.captures(line) {
        Some(cap) => Some(cap[1].to_string()),
        None => None,
    }
}

fn match_end_command(line: &str) -> bool {
    line.contains("<!-- @end-libradoc -->")
}
