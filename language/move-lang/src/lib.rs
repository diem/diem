// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

#[macro_use]
pub mod shared;

pub mod errors;

pub mod cfgir;
pub mod expansion;
pub mod hlir;
pub mod naming;
pub mod parser;
pub mod to_bytecode;
pub mod typing;

pub mod command_line;

pub mod test_utils;

use codespan::{ByteIndex, Span};
use errors::*;
use parser::syntax::parse_file_string;
use shared::{Address, Loc};
use std::{
    collections::HashMap,
    fs::File,
    io::{self, Read, Write},
};

// fn run(_targets: &[&str], _deps: &[&str]) -> io::Result<()> {
//     panic!()
// }

//**************************************************************************************************
// Entry
//**************************************************************************************************

/// Given a set of targets and a set of dependencies
/// - Checks the targets with the dependencies (targets can be dependencies of other targets)
/// Does not run compile to Move bytecode
/// Very large programs might fail on compilation even though they have been checked due to size
///   limitations of the Move bytecode
pub fn move_check(
    targets: &[&'static str],
    deps: &[&'static str],
    sender_opt: Option<Address>,
) -> io::Result<()> {
    let (files, errors) = move_check_no_report(targets, deps, sender_opt)?;
    if !errors.is_empty() {
        errors::report_errors(files, errors)
    }
    Ok(())
}

/// Move check but it returns the errors instead of reporting them to stderr
pub fn move_check_no_report(
    targets: &[&'static str],
    deps: &[&'static str],
    sender_opt: Option<Address>,
) -> io::Result<(FilesSourceText, Errors)> {
    let (files, pprog_res) = parse_program(targets, deps)?;
    match check_program(pprog_res, sender_opt) {
        Err(errors) => Ok((files, errors)),
        Ok(_) => Ok((files, vec![])),
    }
}

/// Given a set of targets and a set of dependencies
/// - Checks the targets with the dependencies (targets can be dependencies of other targets)
/// - Compiles the targets to Move bytecode
/// Does not run the Move bytecode verifier on the compiled targets, as the Move front end should
///   be more restrictive
pub fn move_compile(
    targets: &[&'static str],
    deps: &[&'static str],
    sender_opt: Option<Address>,
) -> io::Result<(FilesSourceText, Vec<to_bytecode::translate::CompiledUnit>)> {
    let (files, pprog_res) = parse_program(targets, deps)?;
    match compile_program(pprog_res, sender_opt) {
        Err(errors) => errors::report_errors(files, errors),
        Ok(compiled_units) => Ok((files, compiled_units)),
    }
}

/// Runs the bytecode verifier on the compiled units
/// Fails if the bytecode verifier errors
pub fn sanity_check_compiled_units(
    files: FilesSourceText,
    compiled_units: Vec<to_bytecode::translate::CompiledUnit>,
) {
    let (_, ice_errors) = to_bytecode::translate::verify_units(compiled_units);
    if !ice_errors.is_empty() {
        errors::report_errors(files, ice_errors)
    }
}

/// Given a file map and a set of compiled programs, saves the compiled programs to disk
pub fn output_compiled_units(
    files: FilesSourceText,
    compiled_units: Vec<to_bytecode::translate::CompiledUnit>,
    out_dir: &str,
) -> io::Result<()> {
    std::fs::create_dir_all(out_dir)?;
    let (compiled_units, ice_errors) = to_bytecode::translate::verify_units(compiled_units);
    let files_and_units = compiled_units
        .into_iter()
        .enumerate()
        .map(|(idx, compiled_unit)| {
            let path = format!(
                "{}/transaction_{}_{}.mv",
                out_dir,
                idx,
                compiled_unit.name()
            );
            let file = File::create(path)?;
            Ok((file, compiled_unit))
        })
        .collect::<io::Result<Vec<_>>>()?;
    for (mut file, compiled_unit) in files_and_units {
        file.write_all(&compiled_unit.serialize())?;
    }
    if !ice_errors.is_empty() {
        errors::report_errors(files, ice_errors)
    }
    Ok(())
}

fn check_program(
    prog: Result<parser::ast::Program, Errors>,
    sender_opt: Option<Address>,
) -> Result<cfgir::ast::Program, Errors> {
    let (eprog, errors) = expansion::translate::program(prog?, sender_opt);
    let (nprog, errors) = naming::translate::program(eprog, errors);
    let (tprog, errors) = typing::translate::program(nprog, errors);
    check_errors(errors)?;
    let (hprog, errors) = hlir::translate::program(tprog);
    let (cprog, errors) = cfgir::translate::program(errors, hprog);
    check_errors(errors)?;
    Ok(cprog)
}

fn compile_program(
    prog: Result<parser::ast::Program, Errors>,
    sender_opt: Option<Address>,
) -> Result<Vec<to_bytecode::translate::CompiledUnit>, Errors> {
    let cprog = check_program(prog, sender_opt)?;
    to_bytecode::translate::program(cprog)
}

fn check_errors(errors: Errors) -> Result<(), Errors> {
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

//**************************************************************************************************
// Parsing
//**************************************************************************************************

fn parse_program(
    targets: &[&'static str],
    deps: &[&'static str],
) -> io::Result<(FilesSourceText, Result<parser::ast::Program, Errors>)> {
    let mut files: FilesSourceText = HashMap::new();
    let mut source_definitions = Vec::new();
    let mut lib_definitions = Vec::new();
    let mut errors: Errors = Vec::new();

    for fname in targets {
        let (def_opt, mut es) = parse_file(&mut files, fname)?;
        if let Some(def) = def_opt {
            source_definitions.push(def);
        }
        errors.append(&mut es);
    }

    for fname in deps {
        let (def_opt, mut es) = parse_file(&mut files, fname)?;
        if let Some(def) = def_opt {
            lib_definitions.push(def);
        }
        errors.append(&mut es);
    }

    let res = if errors.is_empty() {
        Ok(parser::ast::Program {
            source_definitions,
            lib_definitions,
        })
    } else {
        Err(errors)
    };
    Ok((files, res))
}

fn parse_file(
    files: &mut FilesSourceText,
    fname: &'static str,
) -> io::Result<(Option<parser::ast::FileDefinition>, Errors)> {
    let mut errors: Errors = Vec::new();
    let mut f = File::open(fname)?;
    let mut source_buffer = String::new();
    f.read_to_string(&mut source_buffer)?;
    let no_comments_buffer = match strip_comments_and_verify(fname, &source_buffer) {
        Err(err) => {
            errors.push(err);
            return Ok((None, errors));
        }
        Ok(no_comments_buffer) => no_comments_buffer,
    };
    let def_opt = match parse_file_string(fname, &no_comments_buffer) {
        Ok(def) => Some(def),
        Err(err) => {
            errors.push(err);
            None
        }
    };
    files.insert(fname, no_comments_buffer);
    Ok((def_opt, errors))
}

//**************************************************************************************************
// Comments
//**************************************************************************************************

/// Determine if a character is an allowed eye-visible (printable) character.
///
/// The only allowed printable characters are the printable ascii characters (SPACE through ~) and
/// tabs. All other characters are invalid and we return false.
pub fn is_permitted_printable_char(c: char) -> bool {
    let x = c as u32;
    let is_above_space = x >= 0x20; // Don't allow meta characters
    let is_below_tilde = x <= 0x7E; // Don't allow DEL meta character
    let is_tab = x == 0x09; // Allow tabs
    (is_above_space && is_below_tilde) || is_tab
}

/// Determine if a character is a permitted newline character.
///
/// The only permitted newline character is \n. All others are invalid.
pub fn is_permitted_newline_char(c: char) -> bool {
    let x = c as u32;
    x == 0x0A
}

/// Determine if a character is permitted character.
///
/// A permitted character is either a permitted printable character, or a permitted
/// newline. Any other characters are disallowed from appearing in the file.
pub fn is_permitted_char(c: char) -> bool {
    is_permitted_printable_char(c) || is_permitted_newline_char(c)
}

fn verify_string(fname: &'static str, string: &str) -> Result<(), Error> {
    match string
        .chars()
        .enumerate()
        .find(|(_, c)| !is_permitted_char(*c))
    {
        None => Ok(()),
        Some((idx, chr)) => {
            let span = Span::new(ByteIndex(idx as u32), ByteIndex(idx as u32));
            let loc = Loc::new(fname, span);
            let msg = format!(
                "Parser Error: invalid character {} found when reading file.\
                 Only ascii printable, tabs (\\t), and \\n line ending characters are permitted.",
                chr
            );
            Err(vec![(loc, msg)])
        }
    }
}

fn strip_comments(source: &str) -> String {
    const SLASH: char = '/';
    const SPACE: char = ' ';

    let mut in_comment = false;
    let mut acc = String::with_capacity(source.len());
    let mut char_iter = source.chars().peekable();

    while let Some(chr) = char_iter.next() {
        let at_newline = is_permitted_newline_char(chr);
        let at_or_after_slash_slash =
            in_comment || (chr == SLASH && char_iter.peek().map(|c| *c == SLASH).unwrap_or(false));
        in_comment = !at_newline && at_or_after_slash_slash;
        acc.push(if in_comment { SPACE } else { chr });
    }

    acc
}

// We restrict strings to only ascii visual characters (0x20 <= c <= 0x7E) or a permitted newline
// character--\n--or a tab--\t.
fn strip_comments_and_verify(fname: &'static str, string: &str) -> Result<String, Error> {
    verify_string(fname, string)?;
    Ok(strip_comments(string))
}
