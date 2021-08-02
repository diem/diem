// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{bail, Result};
use codespan_reporting::{
    diagnostic::{Diagnostic, Label},
    files::SimpleFiles,
    term::{
        emit,
        termcolor::{ColorChoice, StandardStream},
        Config,
    },
};
use ir_to_bytecode_syntax::syntax::{self, ParseError};
use move_command_line_common::character_sets::is_permitted_char;
use move_core_types::account_address::AccountAddress;
use move_ir_types::{ast, location::*};
use move_symbol_pool::Symbol;

// We restrict strings to only ascii visual characters (0x20 <= c <= 0x7E) or a permitted newline
// character--\n--or a tab--\t. Checking each character in the input string is more fool-proof
// than checking each character later during lexing & tokenization, since that would require special
// handling of characters inside of comments (usually not included as tokens) and within byte
// array literals.
fn verify_string(string: &str) -> Result<()> {
    string
        .chars()
        .find(|c| !is_permitted_char(*c))
        .map_or(Ok(()), |chr| {
            bail!(
                "Parser Error: invalid character {} found when reading file.\
                 Only ascii printable, tabs (\\t), and \\n line ending characters are permitted.",
                chr
            )
        })
}

/// Given the raw input of a file, creates a `ScriptOrModule` enum
/// Fails with `Err(_)` if the text cannot be parsed`
pub fn parse_script_or_module(file_name: Symbol, s: &str) -> Result<ast::ScriptOrModule> {
    verify_string(s)?;
    syntax::parse_script_or_module_string(file_name, s).or_else(|e| handle_error(e, s))
}

/// Given the raw input of a file, creates a `Script` struct
/// Fails with `Err(_)` if the text cannot be parsed
pub fn parse_script(file_name: Symbol, script_str: &str) -> Result<ast::Script> {
    verify_string(script_str)?;
    syntax::parse_script_string(file_name, script_str).or_else(|e| handle_error(e, script_str))
}

/// Given the raw input of a file, creates a single `ModuleDefinition` struct
/// Fails with `Err(_)` if the text cannot be parsed
pub fn parse_module(file_name: Symbol, modules_str: &str) -> Result<ast::ModuleDefinition> {
    verify_string(modules_str)?;
    syntax::parse_module_string(file_name, modules_str).or_else(|e| handle_error(e, modules_str))
}

/// Given the raw input of a file, creates a single `Cmd_` struct
/// Fails with `Err(_)` if the text cannot be parsed
pub fn parse_cmd_(
    file_name: Symbol,
    cmd_str: &str,
    _sender_address: AccountAddress,
) -> Result<ast::Cmd_> {
    verify_string(cmd_str)?;
    syntax::parse_cmd_string(file_name, cmd_str).or_else(|e| handle_error(e, cmd_str))
}

fn handle_error<T>(e: syntax::ParseError<Loc, anyhow::Error>, code_str: &str) -> Result<T> {
    let msg = match &e {
        ParseError::InvalidToken { location } => {
            let mut files = SimpleFiles::new();
            let id = files.add(location.file(), code_str.to_string());
            let lbl = Label::primary(id, location.usize_range()).with_message("Invalid Token");
            let error = Diagnostic::error()
                .with_message("Parser Error")
                .with_labels(vec![lbl]);
            let writer = &mut StandardStream::stderr(ColorChoice::Auto);
            emit(writer, &Config::default(), &files, &error).unwrap();
            "Invalid Token".to_string()
        }
        ParseError::User { error } => {
            println!("{}", error);
            format!("{}", error)
        }
    };
    bail!("ParserError: {}", msg)
}

#[cfg(test)]
mod tests {
    #[test]
    fn verify_character_allowlist() {
        let mut good_chars = (0x20..=0x7E).collect::<Vec<u8>>();
        good_chars.push(0x0A);
        good_chars.push(0x09);

        let mut bad_chars = (0x0..0x09).collect::<Vec<_>>();
        bad_chars.append(&mut (0x0B..=0x1F).collect::<Vec<_>>());
        bad_chars.push(0x7F);

        // Test to make sure that all the characters that are in the allowlist pass.
        {
            let s = std::str::from_utf8(&good_chars)
                .expect("Failed to construct string containing an invalid character. This shouldn't happen.");
            assert!(super::verify_string(s).is_ok());
        }

        // Test to make sure that we fail for all characters not in the allowlist.
        for bad_char in bad_chars {
            good_chars.push(bad_char);
            let s = std::str::from_utf8(&good_chars)
                .expect("Failed to construct string containing an invalid character. This shouldn't happen.");
            assert!(super::verify_string(s).is_err());
            good_chars.pop();
        }
    }
}
