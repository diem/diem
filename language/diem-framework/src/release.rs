// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{path_in_crate, save_binary};
use log::LevelFilter;
use std::{
    collections::BTreeMap,
    fs::{create_dir_all, remove_dir_all},
    path::Path,
};
use vm::CompiledModule;

fn recreate_dir(dir_path: &Path) {
    remove_dir_all(&dir_path).unwrap_or(());
    create_dir_all(&dir_path).unwrap();
}

pub fn build_modules(output_path: &Path) -> BTreeMap<String, CompiledModule> {
    recreate_dir(output_path);

    let compiled_modules = crate::build_stdlib();

    for (name, module) in &compiled_modules {
        let mut bytes = Vec::new();
        module.serialize(&mut bytes).unwrap();
        let mut module_path = Path::join(&output_path, name);
        module_path.set_extension(move_stdlib::COMPILED_EXTENSION);
        save_binary(&module_path, &bytes);
    }

    compiled_modules
}

/// The documentation root template for the Diem Framework modules.
const MODULE_DOC_TEMPLATE: &str = "modules/overview_template.md";

/// Path to the references template.
const REFERENCES_DOC_TEMPLATE: &str = "modules/references_template.md";

pub fn generate_module_docs(output_path: &Path, with_diagram: bool) {
    recreate_dir(output_path);

    move_stdlib::build_doc(
        // FIXME: make move_stdlib::build_doc use Path.
        &output_path.to_string_lossy(),
        // FIXME: use absolute path when the bug in docgen is fixed.
        //        &move_stdlib::move_stdlib_docs_full_path(),
        "../move-stdlib/docs",
        vec![path_in_crate(MODULE_DOC_TEMPLATE)
            .to_string_lossy()
            .to_string()],
        Some(
            path_in_crate(REFERENCES_DOC_TEMPLATE)
                .to_string_lossy()
                .to_string(),
        ),
        crate::diem_stdlib_files_no_dependencies().as_slice(),
        vec![move_stdlib::move_stdlib_modules_full_path()],
        with_diagram,
    )
}

/// The documentation root template for scripts.
const SCRIPT_DOC_TEMPLATE: &str = "script_documentation/script_documentation_template.md";

/// The specification root template for scripts and stdlib.
const SPEC_DOC_TEMPLATE: &str = "script_documentation/spec_documentation_template.md";

pub fn generate_script_docs(output_path: &Path, modules_doc_path: &Path, with_diagram: bool) {
    // recreate_dir(output_path);

    move_stdlib::build_doc(
        // FIXME: make move_stdlib::build_doc use Path.
        &output_path.to_string_lossy(),
        // FIXME: links to move stdlib modules are broken since the tool does not currently
        // support multiple paths.
        // FIXME: use absolute path.
        &modules_doc_path.to_string_lossy(),
        vec![
            path_in_crate(SCRIPT_DOC_TEMPLATE)
                .to_string_lossy()
                .to_string(),
            path_in_crate(SPEC_DOC_TEMPLATE)
                .to_string_lossy()
                .to_string(),
        ],
        Some(
            path_in_crate(REFERENCES_DOC_TEMPLATE)
                .to_string_lossy()
                .to_string(),
        ),
        &[
            path_in_crate("modules/AccountAdministrationScripts.move")
                .to_str()
                .unwrap()
                .to_string(),
            path_in_crate("modules/AccountCreationScripts.move")
                .to_str()
                .unwrap()
                .to_string(),
            path_in_crate("modules/PaymentScripts.move")
                .to_str()
                .unwrap()
                .to_string(),
            path_in_crate("modules/SystemAdministrationScripts.move")
                .to_str()
                .unwrap()
                .to_string(),
            path_in_crate("modules/TreasuryComplianceScripts.move")
                .to_str()
                .unwrap()
                .to_string(),
            path_in_crate("modules/ValidatorAdministrationScripts.move")
                .to_str()
                .unwrap()
                .to_string(),
        ],
        vec![
            move_stdlib::move_stdlib_modules_full_path(),
            crate::diem_stdlib_modules_full_path(),
        ],
        with_diagram,
    )
}

pub fn generate_script_abis(output_path: &Path, compiled_scripts_path: &Path) {
    recreate_dir(output_path);

    let options = move_prover::cli::Options {
        move_sources: crate::diem_stdlib_files(),
        move_deps: vec![
            move_stdlib::move_stdlib_modules_full_path(),
            crate::diem_stdlib_modules_full_path(),
        ],
        verbosity_level: LevelFilter::Warn,
        run_abigen: true,
        abigen: abigen::AbigenOptions {
            output_directory: output_path.to_string_lossy().to_string(),
            compiled_script_directory: compiled_scripts_path.to_string_lossy().to_string(),
        },
        ..Default::default()
    };
    options.setup_logging_for_test();
    move_prover::run_move_prover_errors_to_stderr(options).unwrap();
}

pub fn generate_script_builder(
    output_path: &Path,
    abi_paths: impl IntoIterator<Item = impl AsRef<Path>>,
) {
    let abis: Vec<_> = abi_paths
        .into_iter()
        .flat_map(|path| {
            transaction_builder_generator::read_abis(path.as_ref()).unwrap_or_else(|_| {
                panic!("Failed to read ABIs at {}", path.as_ref().to_string_lossy())
            })
        })
        .collect();

    {
        let mut file = std::fs::File::create(&output_path)
            .expect("Failed to open file for Rust script build generation");
        transaction_builder_generator::rust::output(&mut file, &abis, /* local types */ true)
            .expect("Failed to generate Rust builders for Diem");
    }

    std::process::Command::new("rustfmt")
        .arg("--config")
        .arg("imports_granularity=crate")
        .arg(output_path)
        .status()
        .expect("Failed to run rustfmt on generated code");
}

pub fn build_error_code_map(output_path: &Path) {
    assert!(output_path.is_file());
    recreate_dir(&output_path.parent().unwrap());

    let options = move_prover::cli::Options {
        move_sources: crate::diem_stdlib_files(),
        move_deps: vec![],
        verbosity_level: LevelFilter::Warn,
        run_errmapgen: true,
        errmapgen: errmapgen::ErrmapOptions {
            output_file: output_path.to_string_lossy().to_string(),
            ..Default::default()
        },
        ..Default::default()
    };
    options.setup_logging_for_test();
    move_prover::run_move_prover_errors_to_stderr(options).unwrap();
}
