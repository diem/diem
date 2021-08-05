// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{diem_framework_named_addresses, path_in_crate, save_binary};
use log::LevelFilter;
use move_binary_format::{compatibility::Compatibility, normalized::Module, CompiledModule};
use move_command_line_common::files::{
    extension_equals, find_filenames, MOVE_COMPILED_EXTENSION, MOVE_ERROR_DESC_EXTENSION,
};
use move_core_types::language_storage::ModuleId;
use std::{
    collections::BTreeMap,
    fs::{create_dir_all, remove_dir_all, File},
    io::Read,
    path::Path,
};

fn recreate_dir(dir_path: impl AsRef<Path>) {
    let dir_path = dir_path.as_ref();
    remove_dir_all(&dir_path).unwrap_or(());
    create_dir_all(&dir_path).unwrap();
}

fn extract_old_apis(modules_path: impl AsRef<Path>) -> Option<BTreeMap<ModuleId, Module>> {
    let modules_path = modules_path.as_ref();

    if !modules_path.is_dir() {
        println!(
            "Warning: failed to extract old module APIs -- path \"{}\" is not a directory",
            modules_path.to_string_lossy()
        );
        return None;
    }

    let mut old_module_apis = BTreeMap::new();
    let files = find_filenames(&[modules_path], |p| {
        extension_equals(p, MOVE_COMPILED_EXTENSION)
    })
    .unwrap();
    for f in files {
        let mut bytes = Vec::new();
        File::open(f)
            .expect("Failed to open module bytecode file")
            .read_to_end(&mut bytes)
            .expect("Failed to read module bytecode file");
        let m = CompiledModule::deserialize(&bytes).expect("Failed to deserialize module bytecode");
        old_module_apis.insert(m.self_id(), Module::new(&m));
    }
    Some(old_module_apis)
}

fn build_modules(output_path: impl AsRef<Path>) -> BTreeMap<String, CompiledModule> {
    let output_path = output_path.as_ref();
    recreate_dir(output_path);

    let compiled_modules = crate::build_stdlib();

    for (name, module) in &compiled_modules {
        let mut bytes = Vec::new();
        module.serialize(&mut bytes).unwrap();
        let mut module_path = Path::join(output_path, name);
        module_path.set_extension(MOVE_COMPILED_EXTENSION);
        save_binary(&module_path, &bytes);
    }

    compiled_modules
}

fn check_api_compatibility<'a, I>(old: &BTreeMap<ModuleId, Module>, new: I)
where
    I: IntoIterator<Item = &'a CompiledModule>,
{
    let mut is_linking_layout_compatible = true;
    for module in new.into_iter() {
        // extract new linking/layout API and check compatibility with old
        let new_module_id = module.self_id();
        if let Some(old_api) = old.get(&new_module_id) {
            let new_api = Module::new(module);
            let compatibility = Compatibility::check(old_api, &new_api);
            if is_linking_layout_compatible && !compatibility.is_fully_compatible() {
                println!("Found linking/layout-incompatible change:");
                is_linking_layout_compatible = false
            }
            if !compatibility.struct_and_function_linking {
                println!("Linking API for structs/functions of module {} has changed. Need to redeploy all dependent modules.", new_module_id.name())
            }
            if !compatibility.struct_layout {
                println!("Layout API for structs of module {} has changed. Need to do a data migration of published structs", new_module_id.name())
            }
        }
    }
}

/// The documentation root template for the Diem Framework modules.
const MODULE_DOC_TEMPLATE: &str = "modules/overview_template.md";

/// Path to the references template.
const REFERENCES_DOC_TEMPLATE: &str = "modules/references_template.md";

fn generate_module_docs(output_path: impl AsRef<Path>, with_diagram: bool) {
    let output_path = output_path.as_ref();
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
        diem_framework_named_addresses(),
    )
}

/// The documentation root template for scripts.
const SCRIPT_DOC_TEMPLATE: &str = "script_documentation/script_documentation_template.md";

/// The specification root template for scripts and stdlib.
const SPEC_DOC_TEMPLATE: &str = "script_documentation/spec_documentation_template.md";

fn generate_script_docs(
    output_path: impl AsRef<Path>,
    modules_doc_path: impl AsRef<Path>,
    with_diagram: bool,
) {
    let output_path = output_path.as_ref();
    recreate_dir(output_path);

    move_stdlib::build_doc(
        // FIXME: make move_stdlib::build_doc use Path.
        &output_path.to_string_lossy(),
        // FIXME: links to move stdlib modules are broken since the tool does not currently
        // support multiple paths.
        // FIXME: use absolute path.
        &modules_doc_path.as_ref().to_string_lossy(),
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
        diem_framework_named_addresses(),
    )
}

fn generate_script_abis(
    output_path: impl AsRef<Path>,
    legacy_compiled_scripts_path: impl AsRef<Path>,
) {
    let output_path = output_path.as_ref();
    recreate_dir(output_path);

    let options = move_prover::cli::Options {
        move_sources: crate::diem_stdlib_files(),
        move_deps: vec![
            move_stdlib::move_stdlib_modules_full_path(),
            crate::diem_stdlib_modules_full_path(),
        ],
        verbosity_level: LevelFilter::Warn,
        move_named_address_values: move_prover::cli::named_addresses_for_options(
            &diem_framework_named_addresses(),
        ),
        run_abigen: true,
        abigen: abigen::AbigenOptions {
            output_directory: output_path.to_string_lossy().to_string(),
            compiled_script_directory: legacy_compiled_scripts_path
                .as_ref()
                .to_string_lossy()
                .to_string(),
        },
        ..Default::default()
    };
    options.setup_logging_for_test();
    move_prover::run_move_prover_errors_to_stderr(options).unwrap();
}

fn generate_script_builder(output_path: impl AsRef<Path>, abi_paths: &[impl AsRef<Path>]) {
    let output_path = output_path.as_ref();

    let abis: Vec<_> = abi_paths
        .iter()
        .flat_map(|path| {
            transaction_builder_generator::read_abis(&[path.as_ref()]).unwrap_or_else(|_| {
                panic!("Failed to read ABIs at {}", path.as_ref().to_string_lossy())
            })
        })
        .collect();

    {
        let mut file = std::fs::File::create(output_path)
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

fn build_error_code_map(output_path: impl AsRef<Path>) {
    let output_path = output_path.as_ref();
    //assert!(output_path.is_file());
    recreate_dir(&output_path.parent().unwrap());

    let options = move_prover::cli::Options {
        move_sources: crate::diem_stdlib_files(),
        move_deps: vec![],
        move_named_address_values: move_prover::cli::named_addresses_for_options(
            &diem_framework_named_addresses(),
        ),
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

/// Options to configure the generation of a release.
pub struct ReleaseOptions {
    pub check_layout_compatibility: bool,
    pub build_modules: bool,
    pub module_docs: bool,
    pub script_docs: bool,
    pub with_diagram: bool,
    pub script_abis: bool,
    pub script_builder: bool,
    pub errmap: bool,
    pub time_it: bool,
}

impl Default for ReleaseOptions {
    fn default() -> Self {
        Self {
            check_layout_compatibility: false,
            build_modules: true,
            module_docs: true,
            script_docs: true,
            with_diagram: false,
            script_abis: true,
            script_builder: true,
            errmap: true,
            time_it: false,
        }
    }
}

fn run_step<F, R>(step_msg: Option<&str>, f: F) -> R
where
    F: FnOnce() -> R,
{
    match step_msg {
        Some(msg) => move_stdlib::utils::time_it(msg, f),
        None => f(),
    }
}

/// Create a Diem Framework release in the specified directory.
///
/// Unless being specifically disabled, the release will contain
///   - Compiled Modules
///   - Module Docs
///   - Script Docs
///   - Script ABIs
///   - Script Builder
///   - Error Descriptions
pub fn create_release(output_path: impl AsRef<Path>, options: &ReleaseOptions) {
    let output_path = output_path.as_ref();

    let msg = |s: &'static str| if options.time_it { Some(s) } else { None };

    if options.build_modules {
        let modules_path = output_path.join("modules");
        let mut old_module_apis = None;
        if options.check_layout_compatibility {
            old_module_apis = run_step(
                msg("Extracting linking/layout APIs from old module bytecodes"),
                || extract_old_apis(&modules_path),
            );
        }

        let modules = run_step(msg("Compiling modules"), || build_modules(&modules_path));

        if let Some(old_module_apis) = old_module_apis {
            run_step(msg("Checking linking/layout compatibility"), || {
                check_api_compatibility(&old_module_apis, modules.values())
            })
        }
    }

    let docs_path = output_path.join("docs");
    let module_docs_path = docs_path.join("modules");
    if options.module_docs {
        run_step(msg("Generating module docs"), || {
            generate_module_docs(&module_docs_path, options.with_diagram)
        });
    }
    if options.script_docs {
        run_step(msg("Generating script docs"), || {
            generate_script_docs(
                &docs_path.join("scripts"),
                &module_docs_path,
                options.with_diagram,
            )
        });
    }

    if options.script_abis {
        let script_abis_path = output_path.join("script_abis");
        run_step(msg("Generating script ABIs"), || {
            generate_script_abis(&script_abis_path, &Path::new("releases/legacy/scripts"))
        });
        if options.script_builder {
            run_step(msg("Generating Rust script builder"), || {
                generate_script_builder(
                    &output_path.join("transaction_script_builder.rs"),
                    &[
                        script_abis_path,
                        Path::new("releases/legacy/script_abis").into(),
                    ],
                )
            });
        }
    }

    if options.errmap {
        let mut err_exp_path = output_path
            .join("error_description")
            .join("error_description");
        err_exp_path.set_extension(MOVE_ERROR_DESC_EXTENSION);
        run_step(msg("Generating error explanations"), || {
            build_error_code_map(&err_exp_path)
        });
    }
}

/// Sync generated documentation from the current release to the previous locations of script and
/// module docs.
pub fn sync_doc_files(output_path: &str) {
    let sync = |from: &Path, to: &Path| {
        for s in find_filenames(&[&from], |p| extension_equals(p, "md")).unwrap() {
            let path = Path::new(&s);
            std::fs::copy(&path, to.join(path.strip_prefix(from).unwrap())).unwrap();
        }
    };

    sync(
        &Path::new(output_path).join("docs").join("modules"),
        &Path::new("modules").join("doc"),
    );

    sync(
        &Path::new(output_path).join("docs").join("scripts"),
        Path::new("script_documentation"),
    );
}
