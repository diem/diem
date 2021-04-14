// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    compiled_unit::CompiledUnit,
    errors::FilesSourceText,
    shared::{Address, CompilationEnv},
};
use bytecode_source_map::source_map::SourceMap;
use move_binary_format::file_format::CompiledModule;
use move_core_types::{
    account_address::AccountAddress, identifier::Identifier, language_storage::ModuleId,
    value::MoveValue,
};
use move_ir_types::location::*;
use std::collections::BTreeMap;

pub mod filter_test_members;
pub mod plan_builder;

pub type TestName = String;
pub type MappedCompiledModule = (CompiledModule, SourceMap<Loc>);

#[derive(Debug, Clone)]
pub struct TestPlan {
    pub files: FilesSourceText,
    pub module_tests: BTreeMap<ModuleId, ModuleTestPlan>,
    pub module_info: BTreeMap<ModuleId, MappedCompiledModule>,
}

#[derive(Debug, Clone)]
pub struct ModuleTestPlan {
    pub module_id: ModuleId,
    pub tests: BTreeMap<TestName, TestCase>,
}

#[derive(Debug, Clone)]
pub struct TestCase {
    pub test_name: TestName,
    pub arguments: Vec<MoveValue>,
    pub expected_failure: Option<ExpectedFailure>,
}

#[derive(Debug, Clone)]
pub enum ExpectedFailure {
    // expected failure, but abort code not checked
    Expected,
    // expected failure, abort code checked
    ExpectedWithCode(u64),
}

struct Context<'env> {
    env: &'env mut CompilationEnv,
}

impl<'env> Context<'env> {
    fn new(compilation_env: &'env mut CompilationEnv) -> Self {
        Self {
            env: compilation_env,
        }
    }
}

impl ModuleTestPlan {
    pub fn new(
        (addr, module_name): &(Address, String),
        tests: BTreeMap<TestName, TestCase>,
    ) -> Self {
        let addr = AccountAddress::new(addr.to_u8());
        let name = Identifier::new(module_name.to_string()).unwrap();
        let module_id = ModuleId::new(addr, name);
        ModuleTestPlan { module_id, tests }
    }
}

impl TestPlan {
    pub fn new(
        tests: Vec<ModuleTestPlan>,
        files: &FilesSourceText,
        units: &[CompiledUnit],
    ) -> Self {
        let module_tests: BTreeMap<_, _> = tests
            .into_iter()
            .map(|module_test| (module_test.module_id.clone(), module_test))
            .collect();

        let module_info = units
            .iter()
            .filter_map(|unit| {
                if let CompiledUnit::Module {
                    module, source_map, ..
                } = unit
                {
                    Some((module.self_id(), (module.clone(), source_map.clone())))
                } else {
                    None
                }
            })
            .collect();

        Self {
            files: files.clone(),
            module_info,
            module_tests,
        }
    }
}
