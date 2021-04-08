// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use move_binary_format::CompiledModule;

#[test]
fn can_deserialize_and_verify_modules_from_all_versions() {
    for ver in crate::list_all_releases().unwrap() {
        // Load modules from files
        let blobs = crate::load_modules_from_release(&ver).unwrap();
        assert!(!blobs.is_empty(), "no modules found under version {}", ver);

        // Deserialize modules
        let modules = blobs
            .iter()
            .map(|blob| {
                CompiledModule::deserialize(blob)
                    .unwrap_or_else(|_| panic!("unable to deserialize module from version {}", ver))
            })
            .collect::<Vec<_>>();

        // Verifiy modules
        for (i, module) in modules.iter().enumerate() {
            bytecode_verifier::verify_module(module).unwrap_or_else(|_| {
                panic!(
                    "failed to verify module {} from version {}",
                    module.self_id(),
                    ver
                )
            });

            bytecode_verifier::dependencies::verify_module(
                module,
                modules
                    .iter()
                    .enumerate()
                    .flat_map(|(j, module)| if i == j { None } else { Some(module) }),
            )
            .unwrap();
        }
    }
}

// TODO: tests to ensure script abis and error_descriptions can be correctly read
