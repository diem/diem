// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

#[macro_use(sp)]
extern crate move_ir_types;

pub mod cfgir;
pub mod command_line;
pub mod compiled_unit;
pub mod errors;
pub mod expansion;
pub mod hlir;
pub mod ir_translation;
pub mod naming;
pub mod parser;
pub mod shared;
pub mod test_utils;
mod to_bytecode;
pub mod typing;

use codespan::{ByteIndex, Span};
use compiled_unit::CompiledUnit;
use errors::*;
use move_ir_types::location::*;
use parser::syntax::parse_file_string;
use shared::Address;
use std::{
    collections::{BTreeMap, HashMap},
    fs::File,
    io::{self, ErrorKind, Read, Write},
    iter::Peekable,
    path::{Path, PathBuf},
    str::Chars,
};

pub const MOVE_EXTENSION: &str = "move";
pub const MOVE_COMPILED_EXTENSION: &str = "mv";
pub const SOURCE_MAP_EXTENSION: &str = "mvsm";

//**************************************************************************************************
// Entry
//**************************************************************************************************

/// Given a set of targets and a set of dependencies
/// - Checks the targets with the dependencies (targets can be dependencies of other targets)
/// Does not run compile to Move bytecode
/// Very large programs might fail on compilation even though they have been checked due to size
///   limitations of the Move bytecode
pub fn move_check(
    targets: &[String],
    deps: &[String],
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
    targets: &[String],
    deps: &[String],
    sender_opt: Option<Address>,
) -> io::Result<(FilesSourceText, Errors)> {
    let (files, pprog_and_comments_res) = parse_program(targets, deps)?;
    let pprog_res = pprog_and_comments_res.map(|(pprog, _)| pprog);
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
    targets: &[String],
    deps: &[String],
    sender_opt: Option<Address>,
) -> io::Result<(FilesSourceText, Vec<CompiledUnit>)> {
    let (files, pprog_and_comments_res) = parse_program(targets, deps)?;
    let pprog_res = pprog_and_comments_res.map(|(pprog, _)| pprog);
    match compile_program(pprog_res, sender_opt) {
        Err(errors) => errors::report_errors(files, errors),
        Ok(compiled_units) => Ok((files, compiled_units)),
    }
}

/// Move compile but it returns the errors instead of reporting them to stderr
pub fn move_compile_no_report(
    targets: &[String],
    deps: &[String],
    sender_opt: Option<Address>,
) -> io::Result<(FilesSourceText, Result<Vec<CompiledUnit>, Errors>)> {
    let (files, pprog_and_comments_res) = parse_program(targets, deps)?;
    let pprog_res = pprog_and_comments_res.map(|(pprog, _)| pprog);
    Ok(match compile_program(pprog_res, sender_opt) {
        Err(errors) => (files, Err(errors)),
        Ok(units) => (files, Ok(units)),
    })
}

/// Move compile up to expansion phase, returning errors instead of reporting them to stderr.
///
/// This also returns a map containing documentation comments for each source in `targets`.
pub fn move_compile_to_expansion_no_report(
    targets: &[String],
    deps: &[String],
    sender_opt: Option<Address>,
) -> io::Result<(
    FilesSourceText,
    Result<(expansion::ast::Program, CommentMap), Errors>,
)> {
    let (files, pprog_and_comments_res) = parse_program(targets, deps)?;
    let res = pprog_and_comments_res.and_then(|(pprog, comment_map)| {
        let (eprog, errors) = expansion::translate::program(pprog, sender_opt);
        check_errors(errors)?;
        Ok((eprog, comment_map))
    });
    Ok((files, res))
}

//**************************************************************************************************
// Utils
//**************************************************************************************************

/// Runs the bytecode verifier on the compiled units
/// Fails if the bytecode verifier errors
pub fn sanity_check_compiled_units(files: FilesSourceText, compiled_units: Vec<CompiledUnit>) {
    let (_, ice_errors) = compiled_unit::verify_units(compiled_units);
    if !ice_errors.is_empty() {
        errors::report_errors(files, ice_errors)
    }
}

/// Given a file map and a set of compiled programs, saves the compiled programs to disk
pub fn output_compiled_units(
    emit_source_maps: bool,
    files: FilesSourceText,
    compiled_units: Vec<CompiledUnit>,
    out_dir: &str,
) -> io::Result<()> {
    const SCRIPT_SUB_DIR: &str = "scripts";
    const MODULE_SUB_DIR: &str = "modules";

    macro_rules! emit_unit {
        ($path:ident, $unit:ident) => {{
            if emit_source_maps {
                $path.set_extension(SOURCE_MAP_EXTENSION);
                File::create($path.as_path())?.write_all(&$unit.serialize_source_map())?;
            }

            $path.set_extension(MOVE_COMPILED_EXTENSION);
            File::create($path.as_path())?.write_all(&$unit.serialize())?
        }};
    }

    let (compiled_units, ice_errors) = compiled_unit::verify_units(compiled_units);
    let (modules, scripts): (Vec<_>, Vec<_>) = compiled_units
        .into_iter()
        .partition(|u| matches!(u, CompiledUnit::Module { .. }));

    // modules
    if !modules.is_empty() {
        std::fs::create_dir_all(format!("{}/{}", out_dir, MODULE_SUB_DIR))?;
    }
    for (idx, unit) in modules.into_iter().enumerate() {
        let mut path = PathBuf::from(format!(
            "{}/{}/{}_{}",
            out_dir,
            MODULE_SUB_DIR,
            idx,
            unit.name(),
        ));
        emit_unit!(path, unit);
    }

    // scripts
    if !scripts.is_empty() {
        std::fs::create_dir_all(format!("{}/{}", out_dir, SCRIPT_SUB_DIR))?;
    }
    for unit in scripts {
        let mut path = PathBuf::from(format!("{}/{}/{}", out_dir, SCRIPT_SUB_DIR, unit.name()));
        emit_unit!(path, unit);
    }
    // let script_map = {
    //     let mut m: BTreeMap<String, Vec<CompiledUnit>> = BTreeMap::new();
    //     for u in scripts {
    //         m.entry(u.name()).or_insert_with(Vec::new).push(u)
    //     }
    //     m
    // };
    // for (n, units) in script_map {
    //     let num_units = units.len();
    //     for (idx, unit) in units.into_iter().enumerate() {
    //         let file_name = if num_units == 1 {
    //             format!("{}", n)
    //         } else {
    //             format!("{}_{}", n, idx)
    //         };
    //         let mut path = PathBuf::from(format!("{}/{}/{}", out_dir, SCRIPT_SUB_DIR, file_name));
    //         emit_unit!(path, unit);
    //     }
    // }
    if !ice_errors.is_empty() {
        errors::report_errors(files, ice_errors)
    }
    Ok(())
}

//**************************************************************************************************
// Translations
//**************************************************************************************************

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
) -> Result<Vec<CompiledUnit>, Errors> {
    let cprog = check_program(prog, sender_opt)?;
    to_bytecode::translate::program(cprog)
}

//**************************************************************************************************
// Parsing
//**************************************************************************************************

fn parse_program(
    targets: &[String],
    deps: &[String],
) -> io::Result<(
    FilesSourceText,
    Result<(parser::ast::Program, CommentMap), Errors>,
)> {
    let targets = find_and_intern_move_filenames(targets)?;
    let deps = find_and_intern_move_filenames(deps)?;
    let mut files: FilesSourceText = HashMap::new();
    let mut source_definitions = Vec::new();
    let mut source_comments = CommentMap::new();
    let mut lib_definitions = Vec::new();
    let mut errors: Errors = Vec::new();

    for fname in targets {
        let (defs, comments, mut es) = parse_file(&mut files, fname)?;
        source_definitions.extend(defs);
        source_comments.insert(fname, comments);
        errors.append(&mut es);
    }

    for fname in deps {
        let (defs, _, mut es) = parse_file(&mut files, fname)?;
        lib_definitions.extend(defs);
        errors.append(&mut es);
    }

    let res = if errors.is_empty() {
        Ok((
            parser::ast::Program {
                source_definitions,
                lib_definitions,
            },
            source_comments,
        ))
    } else {
        Err(errors)
    };
    Ok((files, res))
}

fn find_and_intern_move_filenames(files: &[String]) -> io::Result<Vec<&'static str>> {
    let mut result = vec![];
    for file in files {
        let path = Path::new(file);
        if !path.exists() {
            return Err(std::io::Error::new(
                ErrorKind::NotFound,
                format!("No such file or directory '{}'", file),
            ));
        }
        if !path.is_dir() {
            // If the filename is specified directly, add it to the list, regardless
            // of whether it has a ".move" extension.
            result.push(leak_str(file));
            continue;
        }
        for entry in walkdir::WalkDir::new(path)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let entry_path = entry.path();
            if !entry.file_type().is_file() || !has_move_extension(&entry_path) {
                continue;
            }
            match entry_path.to_str() {
                Some(p) => result.push(leak_str(p)),
                None => {
                    return Err(std::io::Error::new(
                        ErrorKind::Other,
                        "non-Unicode file name",
                    ))
                }
            }
        }
    }
    Ok(result)
}

fn has_move_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|s| s.to_str())
        .map_or(false, |extension| extension == MOVE_EXTENSION)
}

// TODO replace with some sort of intern table
fn leak_str(s: &str) -> &'static str {
    Box::leak(Box::new(s.to_owned()))
}

fn parse_file(
    files: &mut FilesSourceText,
    fname: &'static str,
) -> io::Result<(Vec<parser::ast::Definition>, MatchedFileCommentMap, Errors)> {
    let mut errors: Errors = Vec::new();
    let mut f = File::open(fname)
        .map_err(|err| std::io::Error::new(err.kind(), format!("{}: {}", err, fname)))?;
    let mut source_buffer = String::new();
    f.read_to_string(&mut source_buffer)?;
    let (no_comments_buffer, comment_map) = match strip_comments_and_verify(fname, &source_buffer) {
        Err(errs) => {
            errors.extend(errs.into_iter());
            files.insert(fname, source_buffer);
            return Ok((vec![], MatchedFileCommentMap::new(), errors));
        }
        Ok(result) => result,
    };
    let (defs, comments) = match parse_file_string(fname, &no_comments_buffer, comment_map) {
        Ok(defs_and_comments) => defs_and_comments,
        Err(errs) => {
            errors.extend(errs);
            (vec![], MatchedFileCommentMap::new())
        }
    };
    files.insert(fname, source_buffer);
    Ok((defs, comments, errors))
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

fn verify_string(fname: &'static str, string: &str) -> Result<(), Errors> {
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
                "Invalid character '{}' found when reading file. \
                 Only ASCII printable characters, tabs (\\t), and line endings (\\n) \
                 are permitted.",
                chr
            );
            Err(vec![vec![(loc, msg)]])
        }
    }
}

/// Types to represent comments.
pub type CommentMap = BTreeMap<&'static str, MatchedFileCommentMap>;
pub type MatchedFileCommentMap = BTreeMap<ByteIndex, String>;
pub type FileCommentMap = BTreeMap<Span, String>;

/// Strips line and block comments from input source, and collects documentation comments,
/// putting them into a map indexed by the span of the comment region. Comments in the original
/// source will be replaced by spaces, such that positions of source items stay unchanged.
/// Block comments can be nested.
///
/// Documentation comments are comments which start with
/// `///` or `/**`, but not `////` or `/***`. The actually comment delimiters
/// (`/// .. <newline>` and `/** .. */`) will be not included in extracted comment string. The
/// span in the returned map, however, covers the whole region of the comment, including the
/// delimiters.
fn strip_comments(fname: &'static str, input: &str) -> Result<(String, FileCommentMap), Errors> {
    const SLASH: char = '/';
    const SPACE: char = ' ';
    const STAR: char = '*';

    enum State {
        Source,
        LineComment,
        BlockComment,
    }

    let mut source = String::with_capacity(input.len());
    let mut comment_map = FileCommentMap::new();

    let mut state = State::Source;
    let mut pos = 0;
    let mut comment_start_pos = 0;
    let mut comment = String::new();
    let mut block_nest = 0;

    let next_is =
        |peekable: &mut Peekable<Chars>, chr| peekable.peek().map(|c| *c == chr).unwrap_or(false);

    let mut commit_comment = |state, start_pos, end_pos, content: String| match state {
        State::BlockComment if !content.starts_with('*') || content.starts_with("**") => {}
        State::LineComment if !content.starts_with('/') || content.starts_with("//") => {}
        _ => {
            comment_map.insert(Span::new(start_pos, end_pos), content[1..].to_string());
        }
    };

    let mut char_iter = input.chars().peekable();
    while let Some(chr) = char_iter.next() {
        match state {
            // Line comments
            State::Source if chr == SLASH && next_is(&mut char_iter, SLASH) => {
                // Starting line comment. We do not capture the `//` in the comment.
                char_iter.next();
                source.push(SPACE);
                source.push(SPACE);
                comment_start_pos = pos;
                pos += 2;
                state = State::LineComment;
            }
            State::LineComment if is_permitted_newline_char(chr) => {
                // Ending line comment. The newline will be added to the source.
                commit_comment(state, comment_start_pos, pos, std::mem::take(&mut comment));
                source.push(chr);
                pos += 1;
                state = State::Source;
            }
            State::LineComment => {
                // Continuing line comment.
                source.push(SPACE);
                comment.push(chr);
                pos += 1;
            }

            // Block comments.
            State::Source if chr == SLASH && next_is(&mut char_iter, STAR) => {
                // Starting block comment. We do not capture the `/*` in the comment.
                char_iter.next();
                source.push(SPACE);
                source.push(SPACE);
                comment_start_pos = pos;
                pos += 2;
                state = State::BlockComment;
            }
            State::BlockComment if chr == SLASH && next_is(&mut char_iter, STAR) => {
                // Starting nested block comment.
                char_iter.next();
                source.push(SPACE);
                comment.push(chr);
                pos += 1;
                block_nest += 1;
            }
            State::BlockComment
                if block_nest > 0 && chr == STAR && next_is(&mut char_iter, SLASH) =>
            {
                // Ending nested block comment.
                char_iter.next();
                source.push(SPACE);
                comment.push(chr);
                pos -= 1;
                block_nest -= 1;
            }
            State::BlockComment
                if block_nest == 0 && chr == STAR && next_is(&mut char_iter, SLASH) =>
            {
                // Ending block comment. The `*/` will not be captured and also not part of the
                // source.
                char_iter.next();
                source.push(SPACE);
                source.push(SPACE);
                pos += 2;
                commit_comment(state, comment_start_pos, pos, std::mem::take(&mut comment));
                state = State::Source;
            }
            State::BlockComment => {
                // Continuing block comment.
                source.push(SPACE);
                comment.push(chr);
                pos += 1;
            }
            State::Source => {
                // Continuing regular source.
                source.push(chr);
                pos += 1;
            }
        }
    }
    match state {
        State::LineComment => {
            // We allow the last line to have no line terminator
            commit_comment(state, comment_start_pos, pos, std::mem::take(&mut comment));
        }
        State::BlockComment => {
            if pos > 0 {
                // try to point to last real character
                pos -= 1;
            }
            return Err(vec![vec![
                (
                    Loc::new(fname, Span::new(pos, pos)),
                    "unclosed block comment".to_string(),
                ),
                (
                    Loc::new(fname, Span::new(comment_start_pos, comment_start_pos + 2)),
                    "begin of unclosed block comment".to_string(),
                ),
            ]]);
        }
        State::Source => {}
    }

    Ok((source, comment_map))
}

// We restrict strings to only ascii visual characters (0x20 <= c <= 0x7E) or a permitted newline
// character--\n--or a tab--\t.
fn strip_comments_and_verify(
    fname: &'static str,
    string: &str,
) -> Result<(String, FileCommentMap), Errors> {
    verify_string(fname, string)?;
    strip_comments(fname, string)
}
