// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#[allow(unused_imports)]
use log::{debug, info, warn};

use anyhow::bail;
use bytecode_verifier::script_signature;
use heck::SnakeCase;
use move_command_line_common::files::MOVE_COMPILED_EXTENSION;
use move_core_types::{
    abi::{ArgumentABI, ScriptABI, ScriptFunctionABI, TransactionScriptABI, TypeArgumentABI},
    identifier::IdentStr,
    language_storage::TypeTag,
};
use move_model::{
    model::{FunctionEnv, FunctionVisibility, GlobalEnv, ModuleEnv},
    ty,
};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, io::Read, path::PathBuf};

/// Options passed into the ABI generator.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct AbigenOptions {
    /// Where to find the .mv files of scripts.
    pub compiled_script_directory: String,
    /// In which directory to store output.
    pub output_directory: String,
}

impl Default for AbigenOptions {
    fn default() -> Self {
        Self {
            compiled_script_directory: ".".to_string(),
            output_directory: "abi".to_string(),
        }
    }
}

/// The ABI generator.
pub struct Abigen<'env> {
    /// Options.
    options: &'env AbigenOptions,
    /// Input definitions.
    env: &'env GlobalEnv,
    /// Map from file name to generated script ABI (if any).
    output: BTreeMap<String, ScriptABI>,
}

impl<'env> Abigen<'env> {
    /// Creates a new ABI generator.
    pub fn new(env: &'env GlobalEnv, options: &'env AbigenOptions) -> Self {
        Self {
            options,
            env,
            output: Default::default(),
        }
    }

    /// Returns the result of ABI generation, a vector of pairs of filenames
    /// and JSON content.
    pub fn into_result(mut self) -> Vec<(String, Vec<u8>)> {
        std::mem::take(&mut self.output)
            .into_iter()
            .map(|(path, abi)| {
                let content = bcs::to_bytes(&abi).expect("ABI serialization should not fail");
                (path, content)
            })
            .collect()
    }

    /// Generates ABIs for all script modules in the environment (excluding the dependency set).
    pub fn gen(&mut self) {
        for module in self.env.get_modules() {
            if module.is_target() {
                let mut path = PathBuf::from(&self.options.output_directory);
                // We make a directory for all of the script function ABIs in a module. But, if
                // it's a script, we don't create a directory.
                if !module.is_script_module() {
                    path.push(
                        PathBuf::from(module.get_source_path())
                            .file_stem()
                            .expect("file extension"),
                    )
                }

                for abi in self
                    .compute_abi(&module)
                    .map_err(|err| {
                        format!(
                            "Error while processing file {:?}: {}",
                            module.get_source_path(),
                            err
                        )
                    })
                    .unwrap()
                {
                    // If the module is a script module, then the generated ABI is a transaction
                    // script ABI. If the module is not a script module, then all generated ABIs
                    // are script function ABIs.
                    let mut path = path.clone();
                    path.push(
                        PathBuf::from(abi.name())
                            .with_extension("abi")
                            .file_name()
                            .expect("file name"),
                    );
                    self.output.insert(path.to_str().unwrap().to_string(), abi);
                }
            }
        }
    }

    /// Compute the ABIs of all script functions in a module.
    fn compute_abi(&self, module_env: &ModuleEnv<'env>) -> anyhow::Result<Vec<ScriptABI>> {
        // Get all the script functions in this module
        let script_iter: Vec<_> = if module_env.is_script_module() {
            module_env.get_functions().collect()
        } else {
            module_env
                .get_functions()
                .filter(|func| {
                    let module = module_env.get_verified_module();
                    let func_name = module_env.symbol_pool().string(func.get_name());
                    let func_ident = IdentStr::new(&func_name).unwrap();
                    // only pick up script functions that also have a script-callable signature.
                    func.visibility() == FunctionVisibility::Script
                        && script_signature::verify_module_script_function(module, func_ident)
                            .is_ok()
                })
                .collect()
        };

        let mut abis = Vec::new();
        for func in script_iter.iter() {
            abis.push(self.generate_abi_for_function(func, module_env)?);
        }

        Ok(abis)
    }

    fn generate_abi_for_function(
        &self,
        func: &FunctionEnv<'env>,
        module_env: &ModuleEnv<'env>,
    ) -> anyhow::Result<ScriptABI> {
        let symbol_pool = module_env.symbol_pool();
        let name = symbol_pool.string(func.get_name()).to_string();
        let doc = func.get_doc().to_string();
        let ty_args = func
            .get_named_type_parameters()
            .iter()
            .map(|ty_param| {
                TypeArgumentABI::new(symbol_pool.string(ty_param.0).to_string().to_snake_case())
            })
            .collect();
        let args = func
            .get_parameters()
            .iter()
            .filter(|param| !matches!(&param.1, ty::Type::Primitive(ty::PrimitiveType::Signer)))
            .map(|param| {
                let tag = self.get_type_tag(&param.1)?;
                Ok(ArgumentABI::new(
                    symbol_pool.string(param.0).to_string(),
                    tag,
                ))
            })
            .collect::<anyhow::Result<_>>()?;

        // This is a transaction script, so include the code, but no module ID
        if module_env.is_script_module() {
            let code = self.load_compiled_bytes(module_env)?.to_vec();
            Ok(ScriptABI::TransactionScript(TransactionScriptABI::new(
                name, doc, code, ty_args, args,
            )))
        } else {
            // This is a script function, so no code. But we need to include the module ID
            Ok(ScriptABI::ScriptFunction(ScriptFunctionABI::new(
                name,
                module_env.get_verified_module().self_id(),
                doc,
                ty_args,
                args,
            )))
        }
    }

    fn load_compiled_bytes(&self, module_env: &ModuleEnv<'env>) -> anyhow::Result<Vec<u8>> {
        let mut path = PathBuf::from(&self.options.compiled_script_directory);
        path.push(
            PathBuf::from(module_env.get_source_path())
                .with_extension(MOVE_COMPILED_EXTENSION)
                .file_name()
                .expect("file name"),
        );
        let mut f = match std::fs::File::open(path.clone()) {
            Ok(f) => f,
            Err(error) => bail!("Failed to open compiled file {:?}: {}", path, error),
        };
        let mut bytes = Vec::new();
        f.read_to_end(&mut bytes)?;
        Ok(bytes)
    }

    fn get_type_tag(&self, ty0: &ty::Type) -> anyhow::Result<TypeTag> {
        use ty::Type::*;
        let tag = match ty0 {
            Primitive(prim) => {
                use ty::PrimitiveType::*;
                match prim {
                    Bool => TypeTag::Bool,
                    U8 => TypeTag::U8,
                    U64 => TypeTag::U64,
                    U128 => TypeTag::U128,
                    Address => TypeTag::Address,
                    Signer => TypeTag::Signer,
                    Num | Range | EventStore => {
                        bail!("Type {:?} is not allowed in scripts.", ty0)
                    }
                }
            }
            Vector(ty) => {
                let tag = self.get_type_tag(ty)?;
                TypeTag::Vector(Box::new(tag))
            }
            Tuple(_)
            | Struct(_, _, _)
            | TypeParameter(_)
            | Fun(_, _)
            | TypeDomain(_)
            | ResourceDomain(..)
            | Error
            | Var(_)
            | Reference(_, _) => bail!("Type {:?} is not allowed in scripts.", ty0),
        };
        Ok(tag)
    }
}
