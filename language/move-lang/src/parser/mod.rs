// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

mod lexer;
pub(crate) mod syntax;

pub mod ast;
pub mod comments;
pub(crate) mod merge_spec_modules;
pub(crate) mod sources_shadow_deps;

use crate::{
    diagnostics::{Diagnostics, FilesSourceText},
    parser,
    parser::syntax::parse_file_string,
    shared::CompilationEnv,
};
use anyhow::anyhow;
use comments::*;
use move_command_line_common::files::find_move_filenames;
use std::{
    collections::{BTreeSet, HashMap},
    fs::File,
    io::Read,
};

pub(crate) fn parse_program(
    compilation_env: &CompilationEnv,
    targets: &[String],
    deps: &[String],
) -> anyhow::Result<(
    FilesSourceText,
    Result<(parser::ast::Program, CommentMap), Diagnostics>,
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
    let mut diags: Diagnostics = Diagnostics::new();

    for fname in targets {
        let (defs, comments, ds) = parse_file(&mut files, fname)?;
        source_definitions.extend(defs);
        source_comments.insert(fname, comments);
        diags.extend(ds);
    }

    for fname in deps {
        let (defs, _, ds) = parse_file(&mut files, fname)?;
        lib_definitions.extend(defs);
        diags.extend(ds);
    }

    // TODO fix this to allow warnings
    let res = if diags.is_empty() {
        let pprog = parser::ast::Program {
            source_definitions,
            lib_definitions,
        };
        Ok((pprog, source_comments))
    } else {
        Err(diags)
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

// TODO replace with some sort of intern table
fn leak_str(s: &str) -> &'static str {
    Box::leak(Box::new(s.to_owned()))
}

fn parse_file(
    files: &mut FilesSourceText,
    fname: &'static str,
) -> anyhow::Result<(
    Vec<parser::ast::Definition>,
    MatchedFileCommentMap,
    Diagnostics,
)> {
    let mut diags = Diagnostics::new();
    let mut f = File::open(fname)
        .map_err(|err| std::io::Error::new(err.kind(), format!("{}: {}", err, fname)))?;
    let mut source_buffer = String::new();
    f.read_to_string(&mut source_buffer)?;
    let (no_comments_buffer, comment_map) = match strip_comments_and_verify(fname, &source_buffer) {
        Err(ds) => {
            diags.extend(ds);
            files.insert(fname, source_buffer);
            return Ok((vec![], MatchedFileCommentMap::new(), diags));
        }
        Ok(result) => result,
    };
    let (defs, comments) = match parse_file_string(fname, &no_comments_buffer, comment_map) {
        Ok(defs_and_comments) => defs_and_comments,
        Err(ds) => {
            diags.extend(ds);
            (vec![], MatchedFileCommentMap::new())
        }
    };
    files.insert(fname, source_buffer);
    Ok((defs, comments, diags))
}
