// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use clap::{App, Arg};
use diem_framework::*;
use move_stdlib::utils::time_it;
use std::{
    collections::BTreeMap,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};
use vm::{compatibility::Compatibility, normalized::Module, CompiledModule};

/// The file name for the compiled stdlib
const COMPILED_STDLIB_DIR: &str = "stdlib";

/// The output path for stdlib documentation.
const MODULES_DOC_DIR: &str = "modules/doc";

/// The output path for transaction script documentation.
const SCRIPTS_DOC_DIR: &str = "script_documentation";

const ERROR_DESC_DIR: &str = "error_descriptions";
const ERROR_DESC_FILENAME: &str = "error_descriptions";

/// The output path under which compiled script files can be found
const LEGACY_COMPILED_TRANSACTION_SCRIPTS_DIR: &str = "compiled/legacy/transaction_scripts";
/// The output path for transaction script ABIs.
const COMPILED_SCRIPTS_ABI_DIR: &str = "compiled/script_abis";
/// Location of legacy transaction script ABIs
const LEGACY_COMPILED_TRANSACTION_SCRIPTS_ABI_DIR: &str = "compiled/legacy/transaction_scripts/abi";
/// Where to write generated transaction builders.
const TRANSACTION_BUILDERS_GENERATED_SOURCE_PATH: &str =
    "../../client/transaction-builder/src/stdlib.rs";

// Generates the compiled stdlib and transaction scripts. Until this is run changes to the source
// modules/scripts, and changes in the Move compiler will not be reflected in the stdlib used for
// genesis, and everywhere else across the code-base unless otherwise specified.
fn main() {
    let cli = App::new("stdlib")
        .name("Move standard library")
        .author("The Diem Core Contributors")
        .arg(
            Arg::with_name("no-doc")
                .long("no-doc")
                .help("do not generate documentation"),
        )
        .arg(
            Arg::with_name("no-script-abi")
                .long("no-script-abi")
                .requires("no-compiler")
                .help("do not generate script ABIs"),
        )
        .arg(
            Arg::with_name("no-script-builder")
                .long("no-script-builder")
                .help("do not generate script builders"),
        )
        .arg(
            Arg::with_name("no-compiler")
                .long("no-compiler")
                .help("do not compile modules and scripts"),
        )
        .arg(
            Arg::with_name("no-check-linking-layout-compatibility")
                .long("no-check-linking-layout-compatibility")
                .help("do not print information about linking and layout compatibility between the old and new standard library"),
        )
        .arg(
            Arg::with_name("with-diagram")
                .long("with-diagram")
                .help("include diagrams in the stdlib documentation")
        );
    let matches = cli.get_matches();
    let no_doc = matches.is_present("no-doc");
    let no_script_abi = matches.is_present("no-script-abi");
    let no_script_builder = matches.is_present("no-script-builder");
    let no_compiler = matches.is_present("no-compiler");
    let no_check_linking_layout_compatibility =
        matches.is_present("no-check-liking-layout-compatibility");
    let with_diagram = matches.is_present("with-diagram");

    // Make sure that the current directory is `language/diem-framework` from now on.
    let exec_path = std::env::args().next().expect("path of the executable");
    let base_path = std::path::Path::new(&exec_path)
        .parent()
        .unwrap()
        .join("../../language/diem-framework");
    std::env::set_current_dir(&base_path).expect("failed to change directory");

    #[cfg(debug_assertions)]
    {
        println!("NOTE: run this program in --release mode for better speed");
    }

    let mut old_module_apis = BTreeMap::new();
    if !no_check_linking_layout_compatibility {
        time_it(
            "Extracting linking/layout ABIs from old module bytecodes",
            || {
                let mut module_path = PathBuf::from(COMPILED_OUTPUT_PATH);
                module_path.push(COMPILED_STDLIB_DIR);
                for f in move_stdlib::utils::iterate_directory(&module_path) {
                    let mut bytes = Vec::new();
                    File::open(f)
                        .expect("Failed to open module bytecode file")
                        .read_to_end(&mut bytes)
                        .expect("Failed to read module bytecode file");
                    let m = CompiledModule::deserialize(&bytes)
                        .expect("Failed to deserialize module bytecode");
                    old_module_apis.insert(m.self_id(), Module::new(&m));
                }
            },
        );
    }

    if !no_compiler {
        time_it(
            "Compiling modules and checking linking/layout compatibility",
            || {
                let mut module_path = PathBuf::from(COMPILED_OUTPUT_PATH);
                module_path.push(COMPILED_STDLIB_DIR);
                let new_modules = release::build_modules(&module_path);

                let mut is_linking_layout_compatible = true;
                if !no_check_linking_layout_compatibility {
                    for module in new_modules.values() {
                        // extract new linking/layout API and check compatibility with old
                        let new_module_id = module.self_id();
                        if let Some(old_api) = old_module_apis.get(&new_module_id) {
                            let new_api = Module::new(module);
                            let compatibility = Compatibility::check(old_api, &new_api);
                            if is_linking_layout_compatible && !compatibility.is_fully_compatible()
                            {
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
            },
        );
    }

    // Generate documentation
    if !no_doc {
        time_it("Generating stdlib documentation", || {
            release::generate_module_docs(&Path::new(MODULES_DOC_DIR), with_diagram);
        });
        time_it("Generating script documentation", || {
            release::generate_script_docs(
                &Path::new(SCRIPTS_DOC_DIR),
                &Path::new(MODULES_DOC_DIR),
                with_diagram,
            );
        });
    }

    // Generate script ABIs
    if !no_script_abi {
        time_it("Generating script ABIs", || {
            release::generate_script_abis(
                &Path::new(COMPILED_SCRIPTS_ABI_DIR),
                &Path::new(LEGACY_COMPILED_TRANSACTION_SCRIPTS_DIR),
            )
        });
    }

    // Generate script builders in Rust
    if !no_script_builder {
        time_it("Generating Rust script builders", || {
            release::generate_script_builder(
                &Path::new(TRANSACTION_BUILDERS_GENERATED_SOURCE_PATH),
                &[
                    Path::new(COMPILED_SCRIPTS_ABI_DIR),
                    Path::new(LEGACY_COMPILED_TRANSACTION_SCRIPTS_ABI_DIR),
                ],
            );
        });
    }

    time_it("Generating error explanations", || {
        let mut path = PathBuf::from(COMPILED_OUTPUT_PATH);
        path.push(ERROR_DESC_DIR);
        path.push(ERROR_DESC_FILENAME);
        path.set_extension(ERROR_DESC_EXTENSION);
        release::build_error_code_map(&path)
    });
}
