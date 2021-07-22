// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    compiled_unit::{AnnotatedCompiledUnit, NamedCompiledModule},
    diagnostics::FilesSourceText,
    shared::AddressBytes,
};
use move_core_types::{
    account_address::AccountAddress, identifier::Identifier, language_storage::ModuleId,
    value::MoveValue,
};
use std::collections::BTreeMap;

pub mod filter_test_members;
pub mod plan_builder;

pub type TestName = String;

#[derive(Debug, Clone)]
pub struct TestPlan {
    pub files: FilesSourceText,
    pub module_tests: BTreeMap<ModuleId, ModuleTestPlan>,
    pub module_info: BTreeMap<ModuleId, NamedCompiledModule>,
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

impl ModuleTestPlan {
    pub fn new(
        addr: &AddressBytes,
        module_name: &str,
        tests: BTreeMap<TestName, TestCase>,
    ) -> Self {
        let addr = AccountAddress::new((*addr).into_bytes());
        let name = Identifier::new(module_name.to_owned()).unwrap();
        let module_id = ModuleId::new(addr, name);
        ModuleTestPlan { module_id, tests }
    }
}

impl TestPlan {
    pub fn new(
        tests: Vec<ModuleTestPlan>,
        files: FilesSourceText,
        units: Vec<AnnotatedCompiledUnit>,
    ) -> Self {
        let module_tests: BTreeMap<_, _> = tests
            .into_iter()
            .map(|module_test| (module_test.module_id.clone(), module_test))
            .collect();

        let module_info = units
            .into_iter()
            .filter_map(|unit| {
                if let AnnotatedCompiledUnit::Module(annot_module) = unit {
                    Some((
                        annot_module.named_module.module.self_id(),
                        annot_module.named_module,
                    ))
                } else {
                    None
                }
            })
            .collect();

        Self {
            files,
            module_tests,
            module_info,
        }
    }
}
