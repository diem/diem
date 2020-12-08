// Copyright (c) The Diem Core Contributors
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
pub mod interface_generator;
pub mod ir_translation;
pub mod naming;
pub mod parser;
pub mod shared;
mod to_bytecode;
pub mod typing;

use anyhow::anyhow;
use codespan::{ByteIndex, Span};
use compiled_unit::CompiledUnit;
use errors::*;
use move_ir_types::location::*;
use parser::syntax::parse_file_string;
use shared::Address;
use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    fs,
    fs::File,
    io::{Read, Write},
    iter::Peekable,
    path::{Path, PathBuf},
    str::Chars,
};
use tempfile::NamedTempFile;

pub const MOVE_EXTENSION: &str = "move";
pub const MOVE_COMPILED_EXTENSION: &str = "mv";
pub const MOVE_COMPILED_INTERFACES_DIR: &str = "mv_interfaces";
pub const SOURCE_MAP_EXTENSION: &str = "mvsm";

#[macro_export]
macro_rules! unwrap_or_report_errors {
    ($files:ident, $res:ident) => {{
        match $res {
            Ok(t) => t,
            Err(errors) => {
                assert!(!errors.is_empty());
                $crate::errors::report_errors($files, errors)
            }
        }
    }};
}

//**************************************************************************************************
// Entry
//**************************************************************************************************

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Pass {
    Parser,
    Expansion,
    Naming,
    Typing,
    HLIR,
    CFGIR,
    Compilation,
}

pub enum PassResult {
    Parser(Option<Address>, parser::ast::Program),
    Expansion(expansion::ast::Program, Errors),
    Naming(naming::ast::Program, Errors),
    Typing(typing::ast::Program),
    HLIR(hlir::ast::Program, Errors),
    CFGIR(cfgir::ast::Program),
    Compilation(Vec<CompiledUnit>),
}

/// Given a set of targets and a set of dependencies
/// - Checks the targets with the dependencies (targets can be dependencies of other targets)
/// Does not run compile to Move bytecode
/// Very large programs might fail on compilation even though they have been checked due to size
///   limitations of the Move bytecode
pub fn move_check(
    targets: &[String],
    deps: &[String],
    sender_opt: Option<Address>,
    interface_files_dir_opt: Option<String>,
) -> anyhow::Result<(FilesSourceText, Result<(), Errors>)> {
    let (files, pprog_and_comments_res) =
        move_parse(targets, deps, sender_opt, interface_files_dir_opt)?;
    let (_comments, sender_opt, pprog) = match pprog_and_comments_res {
        Err(errors) => return Ok((files, Err(errors))),
        Ok(res) => res,
    };
    let result = match move_continue_up_to(PassResult::Parser(sender_opt, pprog), Pass::CFGIR) {
        Ok(PassResult::CFGIR(_)) => Ok(()),
        Ok(_) => unreachable!(),
        Err(errors) => Err(errors),
    };
    Ok((files, result))
}

/// Similar to move_check but it reports it's errors to stderr
pub fn move_check_and_report(
    targets: &[String],
    deps: &[String],
    sender_opt: Option<Address>,
    interface_files_dir_opt: Option<String>,
) -> anyhow::Result<FilesSourceText> {
    let (files, errors_result) = move_check(targets, deps, sender_opt, interface_files_dir_opt)?;
    unwrap_or_report_errors!(files, errors_result);
    Ok(files)
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
    interface_files_dir_opt: Option<String>,
) -> anyhow::Result<(FilesSourceText, Result<Vec<CompiledUnit>, Errors>)> {
    let (files, pprog_and_comments_res) =
        move_parse(targets, deps, sender_opt, interface_files_dir_opt)?;
    let (_comments, sender_opt, pprog) = match pprog_and_comments_res {
        Err(errors) => return Ok((files, Err(errors))),
        Ok(res) => res,
    };
    let result = match move_continue_up_to(PassResult::Parser(sender_opt, pprog), Pass::Compilation)
    {
        Ok(PassResult::Compilation(units)) => Ok(units),
        Ok(_) => unreachable!(),
        Err(errors) => Err(errors),
    };
    Ok((files, result))
}

/// Similar to move_compile but it reports it's errors to stderr
pub fn move_compile_and_report(
    targets: &[String],
    deps: &[String],
    sender_opt: Option<Address>,
    interface_files_dir_opt: Option<String>,
) -> anyhow::Result<(FilesSourceText, Vec<CompiledUnit>)> {
    let (files, units_res) = move_compile(targets, deps, sender_opt, interface_files_dir_opt)?;
    let units = unwrap_or_report_errors!(files, units_res);
    Ok((files, units))
}

/// Given a set of targets and a set of dependencies, produces a `parser::ast::Program` along side a
/// `CommentMap`. The `sender_opt` is returned for ease of use with `PassResult::Parser`
pub fn move_parse(
    targets: &[String],
    deps: &[String],
    sender_opt: Option<Address>,
    interface_files_dir_opt: Option<String>,
) -> anyhow::Result<(
    FilesSourceText,
    Result<(CommentMap, Option<Address>, parser::ast::Program), Errors>,
)> {
    let mut deps = deps.to_vec();
    generate_interface_files_for_deps(&mut deps, interface_files_dir_opt)?;
    let (files, pprog_and_comments_res) = parse_program(targets, &deps)?;
    let result = pprog_and_comments_res.map(|(pprog, comments)| (comments, sender_opt, pprog));
    Ok((files, result))
}

/// Runs the compiler from a previous result until a stopping point.
/// The stopping point is inclusive, meaning the pass specified by `until: Pass` will be run
pub fn move_continue_up_to(pass: PassResult, until: Pass) -> Result<PassResult, Errors> {
    run(pass, until)
}

//**************************************************************************************************
// Utils
//**************************************************************************************************

macro_rules! dir_path {
    ($($dir:expr),+) => {{
        let mut p = PathBuf::new();
        $(p.push($dir);)+
        p
    }};
}

macro_rules! file_path {
    ($dir:expr, $name:expr, $ext:expr) => {{
        let mut p = PathBuf::from($dir);
        p.push($name);
        p.set_extension($ext);
        p
    }};
}

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
) -> anyhow::Result<()> {
    const SCRIPT_SUB_DIR: &str = "scripts";
    const MODULE_SUB_DIR: &str = "modules";
    fn num_digits(n: usize) -> usize {
        format!("{}", n).len()
    }
    fn format_idx(idx: usize, width: usize) -> String {
        format!("{:0width$}", idx, width = width)
    }

    macro_rules! emit_unit {
        ($path:ident, $unit:ident) => {{
            if emit_source_maps {
                $path.set_extension(SOURCE_MAP_EXTENSION);
                fs::write($path.as_path(), &$unit.serialize_source_map())?;
            }

            $path.set_extension(MOVE_COMPILED_EXTENSION);
            fs::write($path.as_path(), &$unit.serialize())?
        }};
    }

    let (compiled_units, ice_errors) = compiled_unit::verify_units(compiled_units);
    let (modules, scripts): (Vec<_>, Vec<_>) = compiled_units
        .into_iter()
        .partition(|u| matches!(u, CompiledUnit::Module { .. }));

    // modules
    if !modules.is_empty() {
        std::fs::create_dir_all(dir_path!(out_dir, MODULE_SUB_DIR))?;
    }
    let digit_width = num_digits(modules.len());
    for (idx, unit) in modules.into_iter().enumerate() {
        let mut path = dir_path!(
            out_dir,
            MODULE_SUB_DIR,
            format!("{}_{}", format_idx(idx, digit_width), unit.name())
        );
        emit_unit!(path, unit);
    }

    // scripts
    if !scripts.is_empty() {
        std::fs::create_dir_all(dir_path!(out_dir, SCRIPT_SUB_DIR))?;
    }
    for unit in scripts {
        let mut path = dir_path!(out_dir, SCRIPT_SUB_DIR, unit.name());
        emit_unit!(path, unit);
    }

    if !ice_errors.is_empty() {
        errors::report_errors(files, ice_errors)
    }
    Ok(())
}

fn generate_interface_files_for_deps(
    deps: &mut Vec<String>,
    interface_files_dir_opt: Option<String>,
) -> anyhow::Result<()> {
    if let Some(dir) = generate_interface_files(deps, interface_files_dir_opt, true)? {
        deps.push(dir)
    }
    Ok(())
}

pub fn generate_interface_files(
    mv_file_locations: &[String],
    interface_files_dir_opt: Option<String>,
    separate_by_hash: bool,
) -> anyhow::Result<Option<String>> {
    let mv_files = {
        let mut v = vec![];
        let (mv_magic_files, other_file_locations): (Vec<_>, Vec<_>) = mv_file_locations
            .iter()
            .cloned()
            .partition(|s| Path::new(s).is_file() && has_compiled_module_magic_number(s));
        v.extend(mv_magic_files);
        let mv_ext_files = find_filenames(&other_file_locations, |path| {
            extension_equals(path, MOVE_COMPILED_EXTENSION)
        })?;
        v.extend(mv_ext_files);
        v
    };
    if mv_files.is_empty() {
        return Ok(None);
    }

    let interface_files_dir =
        interface_files_dir_opt.unwrap_or_else(|| command_line::DEFAULT_OUTPUT_DIR.to_string());
    let interface_sub_dir = dir_path!(interface_files_dir, MOVE_COMPILED_INTERFACES_DIR);
    let all_addr_dir = if separate_by_hash {
        use std::{
            collections::hash_map::DefaultHasher,
            hash::{Hash, Hasher},
        };
        const HASH_DELIM: &str = "%|%";

        let mut hasher = DefaultHasher::new();
        mv_files.len().hash(&mut hasher);
        HASH_DELIM.hash(&mut hasher);
        for mv_file in &mv_files {
            std::fs::read(mv_file)?.hash(&mut hasher);
            HASH_DELIM.hash(&mut hasher);
        }

        let mut dir = interface_sub_dir;
        dir.push(format!("{:020}", hasher.finish()));
        dir
    } else {
        interface_sub_dir
    };

    for mv_file in mv_files {
        let (id, interface_contents) = interface_generator::write_to_string(&mv_file)?;
        let addr_dir = dir_path!(all_addr_dir.clone(), format!("{}", id.address()));
        let file_path = file_path!(addr_dir.clone(), format!("{}", id.name()), MOVE_EXTENSION);
        // it's possible some files exist but not others due to multithreaded environments
        if separate_by_hash && Path::new(&file_path).is_file() {
            continue;
        }

        let mut tmp = NamedTempFile::new()?;
        tmp.write_all(interface_contents.as_bytes())?;

        std::fs::create_dir_all(addr_dir)?;
        // it's possible some files exist but not others due to multithreaded environments
        // Check for the file existing and then safely move the tmp file there if
        // it does not
        if separate_by_hash && Path::new(&file_path).is_file() {
            continue;
        }
        std::fs::rename(tmp.path(), file_path)?;
    }

    Ok(Some(all_addr_dir.into_os_string().into_string().unwrap()))
}

fn has_compiled_module_magic_number(path: &str) -> bool {
    use move_vm::file_format_common::BinaryConstants;
    let mut file = match File::open(path) {
        Err(_) => return false,
        Ok(f) => f,
    };
    let mut magic = [0u8; BinaryConstants::DIEM_MAGIC_SIZE];
    let num_bytes_read = match file.read(&mut magic) {
        Err(_) => return false,
        Ok(n) => n,
    };
    num_bytes_read == BinaryConstants::DIEM_MAGIC_SIZE && magic == BinaryConstants::DIEM_MAGIC
}

pub fn path_to_string(path: &Path) -> anyhow::Result<String> {
    match path.to_str() {
        Some(p) => Ok(p.to_string()),
        None => Err(anyhow!("non-Unicode file name")),
    }
}

pub fn extension_equals(path: &Path, target_ext: &str) -> bool {
    match path.extension().and_then(|s| s.to_str()) {
        Some(extension) => extension == target_ext,
        None => false,
    }
}

//**************************************************************************************************
// Translations
//**************************************************************************************************

impl PassResult {
    pub fn equivalent_pass(&self) -> Pass {
        match self {
            PassResult::Parser(_, _) => Pass::Parser,
            PassResult::Expansion(_, _) => Pass::Expansion,
            PassResult::Naming(_, _) => Pass::Naming,
            PassResult::Typing(_) => Pass::Typing,
            PassResult::HLIR(_, _) => Pass::HLIR,
            PassResult::CFGIR(_) => Pass::CFGIR,
            PassResult::Compilation(_) => Pass::Compilation,
        }
    }

    pub fn check_for_errors(self) -> Result<Self, Errors> {
        Ok(match self {
            result @ PassResult::Parser(_, _)
            | result @ PassResult::Typing(_)
            | result @ PassResult::CFGIR(_)
            | result @ PassResult::Compilation(_) => result,
            PassResult::Expansion(eprog, errors) => {
                check_errors(errors)?;
                PassResult::Expansion(eprog, Errors::new())
            }
            PassResult::Naming(nprog, errors) => {
                check_errors(errors)?;
                PassResult::Naming(nprog, Errors::new())
            }
            PassResult::HLIR(hprog, errors) => {
                check_errors(errors)?;
                PassResult::HLIR(hprog, Errors::new())
            }
        })
    }
}

fn run(cur: PassResult, until: Pass) -> Result<PassResult, Errors> {
    if cur.equivalent_pass() >= until {
        return Ok(cur);
    }

    match cur {
        PassResult::Parser(sender_opt, prog) => {
            let (eprog, errors) = expansion::translate::program(prog, sender_opt);
            run(PassResult::Expansion(eprog, errors), until)
        }
        PassResult::Expansion(eprog, errors) => {
            let (nprog, errors) = naming::translate::program(eprog, errors);
            run(PassResult::Naming(nprog, errors), until)
        }
        PassResult::Naming(nprog, errors) => {
            let (tprog, errors) = typing::translate::program(nprog, errors);
            check_errors(errors)?;
            run(PassResult::Typing(tprog), until)
        }
        PassResult::Typing(tprog) => {
            let (hprog, errors) = hlir::translate::program(tprog);
            run(PassResult::HLIR(hprog, errors), until)
        }
        PassResult::HLIR(hprog, errors) => {
            let (cprog, errors) = cfgir::translate::program(errors, hprog);
            check_errors(errors)?;
            run(PassResult::CFGIR(cprog), until)
        }
        PassResult::CFGIR(cprog) => {
            let compiled_units = to_bytecode::translate::program(cprog)?;
            assert!(until == Pass::Compilation);
            run(PassResult::Compilation(compiled_units), Pass::Compilation)
        }
        PassResult::Compilation(_) => unreachable!("ICE Pass::Compilation is >= all passes"),
    }
}

//**************************************************************************************************
// Parsing
//**************************************************************************************************

fn parse_program(
    targets: &[String],
    deps: &[String],
) -> anyhow::Result<(
    FilesSourceText,
    Result<(parser::ast::Program, CommentMap), Errors>,
)> {
    let targets = find_move_filenames(targets, true)?
        .iter()
        .map(|s| leak_str(s))
        .collect::<Vec<&'static str>>();
    let deps = find_move_filenames(deps, true)?
        .iter()
        .map(|s| leak_str(s))
        .collect::<Vec<&'static str>>();
    check_targets_deps_dont_intersect(&targets, &deps)?;
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

fn check_targets_deps_dont_intersect(
    targets: &[&'static str],
    deps: &[&'static str],
) -> anyhow::Result<()> {
    let target_set = targets.iter().collect::<BTreeSet<_>>();
    let dep_set = deps.iter().collect::<BTreeSet<_>>();
    let intersection = target_set.intersection(&dep_set).collect::<Vec<_>>();
    if intersection.is_empty() {
        return Ok(());
    }

    let all_files = intersection
        .into_iter()
        .map(|s| format!("    {}", s))
        .collect::<Vec<_>>()
        .join("\n");
    Err(anyhow!(
        "The following files were marked as both targets and dependencies:\n{}",
        all_files
    ))
}

/// - For each directory in `paths`, it will return all files with the `MOVE_EXTENSION` found
///   recursively in that directory
/// - If `keep_specified_files` any file explicitly passed in `paths`, will be added to the result
///   Otherwise, they will be discarded
pub fn find_move_filenames(
    paths: &[String],
    keep_specified_files: bool,
) -> anyhow::Result<Vec<String>> {
    if keep_specified_files {
        let (mut files, other_paths): (Vec<String>, Vec<String>) =
            paths.iter().cloned().partition(|s| Path::new(s).is_file());
        files.extend(find_filenames(&other_paths, |path| {
            extension_equals(path, MOVE_EXTENSION)
        })?);
        Ok(files)
    } else {
        find_filenames(paths, |path| extension_equals(path, MOVE_EXTENSION))
    }
}

/// - For each directory in `paths`, it will return all files that satisfy the predicate
/// - Any file explicitly passed in `paths`, it will include that file in the result, regardless
///   of the file extension
pub fn find_filenames<Predicate: FnMut(&Path) -> bool>(
    paths: &[String],
    mut is_file_desired: Predicate,
) -> anyhow::Result<Vec<String>> {
    let mut result = vec![];

    for s in paths {
        let path = Path::new(s);
        if !path.exists() {
            return Err(anyhow!(format!("No such file or directory '{}'", s)));
        }
        if path.is_file() && is_file_desired(path) {
            result.push(path_to_string(path)?);
            continue;
        }
        if !path.is_dir() {
            continue;
        }
        for entry in walkdir::WalkDir::new(path)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let entry_path = entry.path();
            if !entry.file_type().is_file() || !is_file_desired(&entry_path) {
                continue;
            }

            result.push(path_to_string(entry_path)?);
        }
    }
    Ok(result)
}

// TODO replace with some sort of intern table
fn leak_str(s: &str) -> &'static str {
    Box::leak(Box::new(s.to_owned()))
}

fn parse_file(
    files: &mut FilesSourceText,
    fname: &'static str,
) -> anyhow::Result<(Vec<parser::ast::Definition>, MatchedFileCommentMap, Errors)> {
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
                "Invalid character '{}' found when reading file. Only ASCII printable characters, \
                 tabs (\\t), and line endings (\\n) are permitted.",
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
    const QUOTE: char = '"';
    const BACKSLASH: char = '\\';

    enum State {
        Source,
        String,
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
            // Strings
            State::Source if chr == QUOTE => {
                source.push(chr);
                pos += 1;
                state = State::String;
            }
            State::String => {
                source.push(chr);
                pos += 1;
                if chr == BACKSLASH {
                    // Skip over the escaped character (e.g., a quote or another backslash)
                    if let Some(next) = char_iter.next() {
                        source.push(next);
                        pos += 1;
                    }
                } else if chr == QUOTE {
                    state = State::Source;
                }
            }
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
        State::Source | State::String => {}
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
