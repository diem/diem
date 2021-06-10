// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

mod lexer;
pub(crate) mod syntax;

pub mod ast;
pub mod comments;
pub(crate) mod merge_spec_modules;
pub(crate) mod sources_shadow_deps;

use crate::{
    command_line::MOVE_EXTENSION, errors::*, parser, parser::syntax::parse_file_string,
    shared::CompilationEnv,
};
use anyhow::anyhow;
use comments::*;
use std::{
    collections::{BTreeSet, HashMap},
    fs::File,
    io::Read,
    path::Path,
};

pub(crate) fn parse_program(
    compilation_env: &CompilationEnv,
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
    let mut deps = find_move_filenames(deps, true)?
        .iter()
        .map(|s| leak_str(s))
        .collect::<Vec<&'static str>>();
    ensure_targets_deps_dont_intersect(compilation_env, &targets, &mut deps)?;
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
        let pprog = parser::ast::Program {
            source_definitions,
            lib_definitions,
        };
        Ok((pprog, source_comments))
    } else {
        Err(errors)
    };
    Ok((files, res))
}

fn ensure_targets_deps_dont_intersect(
    compilation_env: &CompilationEnv,
    targets: &[&'static str],
    deps: &mut Vec<&'static str>,
) -> anyhow::Result<()> {
    /// Canonicalize a file path.
    fn canonicalize(path: &str) -> String {
        match std::fs::canonicalize(path) {
            Ok(s) => s.to_string_lossy().to_string(),
            Err(_) => path.to_owned(),
        }
    }
    let target_set = targets
        .iter()
        .map(|s| canonicalize(s))
        .collect::<BTreeSet<_>>();
    let dep_set = deps
        .iter()
        .map(|s| canonicalize(s))
        .collect::<BTreeSet<_>>();
    let intersection = target_set.intersection(&dep_set).collect::<Vec<_>>();
    if intersection.is_empty() {
        return Ok(());
    }
    if compilation_env.flags().sources_shadow_deps() {
        deps.retain(|fname| !intersection.contains(&&canonicalize(fname)));
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
