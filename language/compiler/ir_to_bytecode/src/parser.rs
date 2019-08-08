// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use codespan::{ByteIndex, CodeMap, Span};
use codespan_reporting::{emit, termcolor::Buffer, Diagnostic, Label, Severity};
use failure::*;
use ir_to_bytecode_syntax::syntax;
use lalrpop_util::ParseError;
use regex::Regex;
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};
use types::account_address::AccountAddress;

// Re-export this to make it convenient for other crates.
pub use ir_to_bytecode_syntax::ast;

// Since lalrpop can't handle comments without a custom lexer, we somewhat hackily remove all the
// comments from the input string before passing it off to lalrpop. We only support single line
// comments for now. Will later on add in other comment types.
fn strip_comments(string: &str) -> String {
    // Remove line comments
    let line_comments = Regex::new(r"(?m)//.*$").unwrap();
    line_comments.replace_all(string, "$1").into_owned()
}

/// Given the raw input of a file, creates a `Program` struct
/// Fails with `Err(_)` if the text cannot be parsed
pub fn parse_program(program_str: &str) -> Result<ast::Program> {
    let stripped_string = &strip_comments(program_str);
    let parser = syntax::ProgramParser::new();
    match parser.parse(stripped_string) {
        Ok(program) => Ok(program),
        Err(e) => handle_error(e, program_str),
    }
}

/// Given the raw input of a file, creates a `Script` struct
/// Fails with `Err(_)` if the text cannot be parsed
pub fn parse_script(script_str: &str) -> Result<ast::Script> {
    let stripped_string = &strip_comments(script_str);
    let parser = syntax::ScriptParser::new();
    match parser.parse(stripped_string) {
        Ok(script) => Ok(script),
        Err(e) => handle_error(e, script_str),
    }
}

/// Given the raw input of a file, creates a single `ModuleDefinition` struct
/// Fails with `Err(_)` if the text cannot be parsed
pub fn parse_module(modules_str: &str) -> Result<ast::ModuleDefinition> {
    let stripped_string = &strip_comments(modules_str);
    let parser = syntax::ModuleParser::new();
    match parser.parse(stripped_string) {
        Ok(module) => Ok(module),
        Err(e) => handle_error(e, modules_str),
    }
}

/// Given the raw input of a file, creates a single `Cmd` struct
/// Fails with `Err(_)` if the text cannot be parsed
pub fn parse_cmd(cmd_str: &str, _sender_address: AccountAddress) -> Result<ast::Cmd> {
    let stripped_string = &strip_comments(cmd_str);
    let parser = syntax::CmdParser::new();
    match parser.parse(stripped_string) {
        Ok(cmd) => Ok(cmd),
        Err(e) => handle_error(e, cmd_str),
    }
}

fn handle_error<'input, T, Token>(
    e: lalrpop_util::ParseError<usize, Token, &'static str>,
    code_str: &'input str,
) -> Result<T>
where
    Token: std::fmt::Display,
{
    let mut s = DefaultHasher::new();
    code_str.hash(&mut s);
    let mut code = CodeMap::new();
    code.add_filemap(s.finish().to_string().into(), code_str.to_string());
    let msg = match &e {
        ParseError::InvalidToken { location } => {
            let error =
                Diagnostic::new(Severity::Error, "Invalid Token").with_label(Label::new_primary(
                    Span::new(ByteIndex(*location as u32), ByteIndex(*location as u32)),
                ));
            let mut buffer = Buffer::no_color();
            emit(&mut buffer, &code, &error).unwrap();
            std::str::from_utf8(buffer.as_slice()).unwrap().to_string()
        }
        ParseError::UnrecognizedToken {
            token: Some((l, tok, r)),
            expected,
        } => {
            let error = Diagnostic::new(Severity::Error, format!("Unrecognized Token: {}", tok))
                .with_label(
                    Label::new_primary(Span::new(ByteIndex(*l as u32), ByteIndex(*r as u32)))
                        .with_message(format!(
                            "Expected: {}",
                            expected
                                .iter()
                                .fold(String::new(), |acc, token| format!("{} {},", acc, token))
                        )),
                );
            let mut buffer = Buffer::no_color();
            emit(&mut buffer, &code, &error).unwrap();
            std::str::from_utf8(buffer.as_slice()).unwrap().to_string()
        }
        _ => format!("{}", e),
    };
    println!("{}", msg);
    bail!("ParserError: {}", e)
}
