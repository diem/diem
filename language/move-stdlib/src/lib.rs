// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use log::LevelFilter;
use move_command_line_common::files::{extension_equals, find_filenames, MOVE_EXTENSION};
use move_lang::shared::AddressBytes;
use std::{collections::BTreeMap, path::PathBuf};

#[cfg(test)]
mod tests;
pub mod utils;

pub mod natives;

const MODULES_DIR: &str = "modules";
const NURSERY_DIR: &str = "nursery";
const DOCS_DIR: &str = "docs";
const NURSERY_DOCS_DIR: &str = "nursery/docs";
const ERRMAP_FILE: &str = "error_description.errmap";

const REFERENCES_TEMPLATE: &str = "templates/references.md";
const OVERVIEW_TEMPLATE: &str = "templates/overview.md";

pub fn unit_testing_files() -> Vec<String> {
    vec![path_in_crate("nursery/UnitTest.move")]
        .into_iter()
        .map(|p| p.into_os_string().into_string().unwrap())
        .collect()
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

pub fn move_stdlib_errmap_full_path() -> String {
    format!("{}/{}", env!("CARGO_MANIFEST_DIR"), ERRMAP_FILE)
}

pub fn move_stdlib_files() -> Vec<String> {
    let path = path_in_crate(MODULES_DIR);
    find_filenames(&[path], |p| extension_equals(p, MOVE_EXTENSION)).unwrap()
}

pub fn move_nursery_files() -> Vec<String> {
    let path = path_in_crate(NURSERY_DIR);
    find_filenames(&[path], |p| extension_equals(p, MOVE_EXTENSION)).unwrap()
}

pub fn move_stdlib_named_addresses() -> BTreeMap<String, AddressBytes> {
    let mapping = [("Std", "0x1")];
    mapping
        .iter()
        .map(|(name, addr)| (name.to_string(), AddressBytes::parse_str(addr).unwrap()))
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
    named_addresses: BTreeMap<String, AddressBytes>,
) {
    let options = move_prover::cli::Options {
        move_sources: sources.to_vec(),
        move_deps: dep_paths,
        move_named_address_values: move_prover::cli::named_addresses_for_options(&named_addresses),
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
        move_stdlib_named_addresses(),
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
        move_stdlib_named_addresses(),
    )
}

pub fn build_error_code_map(output_path: &str) {
    let options = move_prover::cli::Options {
        move_sources: crate::move_stdlib_files(),
        move_deps: vec![],
        verbosity_level: LevelFilter::Warn,
        run_errmapgen: true,
        move_named_address_values: move_prover::cli::named_addresses_for_options(
            &move_stdlib_named_addresses(),
        ),
        errmapgen: errmapgen::ErrmapOptions {
            output_file: output_path.to_string(),
            ..Default::default()
        },
        ..Default::default()
    };
    options.setup_logging_for_test();
    move_prover::run_move_prover_errors_to_stderr(options).unwrap();
}

const ERROR_DESCRIPTIONS: &[u8] = include_bytes!("../error_description.errmap");

pub fn error_descriptions() -> &'static [u8] {
    ERROR_DESCRIPTIONS
}
