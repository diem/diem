// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use log::LevelFilter;
use std::path::PathBuf;

#[cfg(test)]
mod tests;
pub mod utils;

pub mod natives;

pub const MOVE_EXTENSION: &str = "move";
pub const COMPILED_EXTENSION: &str = "mv";
pub const ERROR_DESC_EXTENSION: &str = "errmap";

const MODULES_DIR: &str = "modules";
const NURSERY_DIR: &str = "nursery";
const DOCS_DIR: &str = "docs";
const NURSERY_DOCS_DIR: &str = "nursery/docs";

const REFERENCES_TEMPLATE: &str = "templates/references.md";
const OVERVIEW_TEMPLATE: &str = "templates/overview.md";

pub fn filter_move_files(dir_iter: impl Iterator<Item = PathBuf>) -> impl Iterator<Item = PathBuf> {
    dir_iter.flat_map(|path| {
        if path.extension()?.to_str()? == MOVE_EXTENSION {
            Some(path)
        } else {
            None
        }
    })
}

pub fn filter_move_bytecode_files(
    dir_iter: impl Iterator<Item = PathBuf>,
) -> impl Iterator<Item = PathBuf> {
    dir_iter.flat_map(|path| {
        if path.extension()?.to_str()? == COMPILED_EXTENSION {
            Some(path)
        } else {
            None
        }
    })
}

pub fn unit_testing_module_file() -> String {
    path_in_crate("nursery/UnitTest.move")
        .into_os_string()
        .into_string()
        .unwrap()
}

pub fn path_in_crate<S>(relative: S) -> PathBuf
where
    S: Into<String>,
{
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push(relative.into());
    path
}

pub fn move_stdlib_modules_full_path() -> String {
    format!("{}/{}", env!("CARGO_MANIFEST_DIR"), MODULES_DIR)
}

pub fn move_stdlib_docs_full_path() -> String {
    format!("{}/{}", env!("CARGO_MANIFEST_DIR"), DOCS_DIR)
}

pub fn move_nursery_docs_full_path() -> String {
    format!("{}/{}", env!("CARGO_MANIFEST_DIR"), NURSERY_DOCS_DIR)
}

pub fn move_stdlib_files() -> Vec<String> {
    let path = path_in_crate(MODULES_DIR);
    let dirfiles = utils::iterate_directory(&path);
    filter_move_files(dirfiles)
        .flat_map(|path| path.into_os_string().into_string())
        .collect()
}

pub fn move_nursery_files() -> Vec<String> {
    let path = path_in_crate(NURSERY_DIR);
    let dirfiles = utils::iterate_directory(&path);
    filter_move_files(dirfiles)
        .flat_map(|path| path.into_os_string().into_string())
        .collect()
}

pub fn build_doc(
    output_path: &str,
    doc_path: &str,
    templates: Vec<String>,
    references_file: Option<String>,
    sources: &[String],
    dep_paths: Vec<String>,
    with_diagram: bool,
) {
    let options = move_prover::cli::Options {
        move_sources: sources.to_vec(),
        move_deps: dep_paths,
        verbosity_level: LevelFilter::Warn,
        run_docgen: true,
        docgen: docgen::DocgenOptions {
            root_doc_templates: templates,
            references_file,
            doc_path: vec![doc_path.to_string()],
            output_directory: output_path.to_string(),
            include_dep_diagrams: with_diagram,
            include_call_diagrams: with_diagram,
            ..Default::default()
        },
        ..Default::default()
    };
    options.setup_logging_for_test();
    move_prover::run_move_prover_errors_to_stderr(options).unwrap();
}

pub fn build_stdlib_doc(output_path: &str) {
    build_doc(
        output_path,
        "",
        vec![path_in_crate(OVERVIEW_TEMPLATE)
            .to_string_lossy()
            .to_string()],
        Some(
            path_in_crate(REFERENCES_TEMPLATE)
                .to_string_lossy()
                .to_string(),
        ),
        move_stdlib_files().as_slice(),
        vec![],
        false,
    )
}

pub fn build_nursery_doc(output_path: &str) {
    build_doc(
        output_path,
        "",
        vec![],
        None,
        move_nursery_files().as_slice(),
        vec![move_stdlib_modules_full_path()],
        false,
    )
}
