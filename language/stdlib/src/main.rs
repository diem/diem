// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use clap::{App, Arg};
use rayon::prelude::*;
use std::{
    collections::BTreeMap,
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
    time::Instant,
};
use stdlib::*;
use vm::{compatibility::Compatibility, normalized::Module, CompiledModule};

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

    // Make sure that the current directory is `language/stdlib` from now on.
    let exec_path = std::env::args().next().expect("path of the executable");
    let base_path = std::path::Path::new(&exec_path)
        .parent()
        .unwrap()
        .join("../../language/stdlib");
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
                for f in stdlib::utils::iterate_directory(&module_path) {
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
                std::fs::create_dir_all(COMPILED_OUTPUT_PATH).unwrap();
                let mut module_path = PathBuf::from(COMPILED_OUTPUT_PATH);
                module_path.push(COMPILED_STDLIB_DIR);
                let new_modules = build_stdlib();

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

                // write module bytecodes to disk. start by clearing out all of the old ones
                std::fs::remove_dir_all(&module_path).unwrap();
                std::fs::create_dir_all(&module_path).unwrap();
                for (name, module) in new_modules {
                    let mut bytes = Vec::new();
                    module.serialize(&mut bytes).unwrap();
                    module_path.push(name);
                    module_path.set_extension(COMPILED_EXTENSION);
                    save_binary(&module_path, &bytes);
                    module_path.pop();
                }
            },
        );
    }

    let txn_source_files = stdlib::utils::iterate_directory(Path::new(TRANSACTION_SCRIPTS));
    let transaction_files = filter_move_files(txn_source_files)
        .flat_map(|path| path.into_os_string().into_string().ok())
        .collect::<Vec<_>>();
    if !no_compiler {
        time_it("Compiling transaction scripts", || {
            std::fs::remove_dir_all(&COMPILED_TRANSACTION_SCRIPTS_DIR).unwrap_or(());
            std::fs::create_dir_all(&COMPILED_TRANSACTION_SCRIPTS_DIR).unwrap();

            transaction_files.par_iter().for_each(|txn_file| {
                let compiled_script = compile_script(txn_file.clone());
                let mut txn_path = PathBuf::from(COMPILED_OUTPUT_PATH);
                txn_path.push(txn_file.clone());
                txn_path.set_extension(COMPILED_EXTENSION);
                save_binary(&txn_path, &compiled_script);
            })
        });
    }

    // Generate documentation
    if !no_doc {
        time_it("Generating stdlib documentation", || {
            std::fs::remove_dir_all(&STD_LIB_DOC_DIR).unwrap_or(());
            std::fs::create_dir_all(&STD_LIB_DOC_DIR).unwrap();
            build_stdlib_doc(with_diagram);
        });
        time_it("Generating script documentation", || {
            std::fs::remove_dir_all(&TRANSACTION_SCRIPTS_DOC_DIR).unwrap_or(());
            std::fs::create_dir_all(&TRANSACTION_SCRIPTS_DOC_DIR).unwrap();
            build_transaction_script_doc(&transaction_files, with_diagram);
        });
    }

    // Generate script ABIs
    if !no_script_abi {
        time_it("Generating script ABIs", || {
            std::fs::remove_dir_all(&COMPILED_TRANSACTION_SCRIPTS_ABI_DIR).unwrap_or(());
            std::fs::create_dir_all(&COMPILED_TRANSACTION_SCRIPTS_ABI_DIR).unwrap();

            transaction_files
                .par_iter()
                .for_each(|txn_file| build_transaction_script_abi(txn_file.clone()));
        });
    }

    // Generate script builders in Rust
    if !no_script_builder {
        time_it("Generating Rust script builders", || {
            generate_rust_transaction_builders();
        });
    }

    time_it("Generating error explanations", || {
        build_stdlib_error_code_map()
    });

    time_it("Generating packed types map", build_packed_types_map);
}

fn time_it<F>(msg: &str, mut f: F)
where
    F: FnMut(),
{
    let now = Instant::now();
    print!("{} ... ", msg);
    let _ = std::io::stdout().flush();
    f();
    println!("(took {:.3}s)", now.elapsed().as_secs_f64());
}
