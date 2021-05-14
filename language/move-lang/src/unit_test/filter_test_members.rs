// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    parser::ast as P,
    shared::{known_attributes, CompilationEnv},
    unit_test::Context,
};
use move_ir_types::location::Loc;

//***************************************************************************
// Filtering of test-annotated module members
//***************************************************************************

// This filters out all test, and test-only annotated module member from `prog` if the `test` flag
// in `compilation_env` is not set. If the test flag is set, no filtering is performed, and instead
// a test plan is created for use by the testing framework.
pub fn program(compilation_env: &mut CompilationEnv, prog: P::Program) -> P::Program {
    let mut context = Context::new(compilation_env);

    let P::Program {
        source_definitions,
        lib_definitions,
    } = prog;

    let lib_definitions: Vec<_> = lib_definitions
        .into_iter()
        .filter_map(|def| filter_tests_from_definition(&mut context, def))
        .collect();

    let source_definitions: Vec<_> = source_definitions
        .into_iter()
        .filter_map(|def| filter_tests_from_definition(&mut context, def))
        .collect();

    P::Program {
        source_definitions,
        lib_definitions,
    }
}

fn filter_tests_from_definition(
    context: &mut Context,
    def: P::Definition,
) -> Option<P::Definition> {
    match def {
        P::Definition::Module(m) => filter_tests_from_module(context, m).map(P::Definition::Module),
        P::Definition::Address(attributes, loc, addr, ms) => {
            let ms: Vec<_> = ms
                .into_iter()
                .filter_map(|m| filter_tests_from_module(context, m))
                .collect();
            if !ms.is_empty() && !should_remove_node(context.env, &attributes) {
                Some(P::Definition::Address(attributes, loc, addr, ms))
            } else {
                None
            }
        }
        P::Definition::Script(P::Script {
            attributes,
            uses,
            constants,
            function,
            specs,
            loc,
        }) => {
            let script_attributes = attributes
                .iter()
                .chain(uses.iter().flat_map(|use_decl| &use_decl.attributes))
                .chain(function.attributes.iter())
                .chain(specs.iter().flat_map(|spec| &spec.value.attributes));

            let errors: Vec<_> = script_attributes
                .map(|attr| {
                    test_attributes(attr).into_iter().map(|(loc, _)| {
                        let msg = "Testing attributes are not allowed in scripts.";
                        vec![(loc, msg.to_string())]
                    })
                })
                .flatten()
                .collect();

            if errors.is_empty() {
                Some(P::Definition::Script(P::Script {
                    attributes,
                    loc,
                    uses,
                    constants,
                    function,
                    specs,
                }))
            } else {
                context.env.add_errors(errors);
                None
            }
        }
    }
}

fn filter_tests_from_module(
    context: &mut Context,
    module_def: P::ModuleDefinition,
) -> Option<P::ModuleDefinition> {
    if should_remove_node(context.env, &module_def.attributes) {
        return None;
    }

    let P::ModuleDefinition {
        attributes,
        loc,
        address,
        name,
        members,
    } = module_def;

    let new_members: Vec<_> = members
        .into_iter()
        .filter_map(|member| filter_tests_from_module_member(context, member))
        .collect();

    Some(P::ModuleDefinition {
        attributes,
        loc,
        address,
        name,
        members: new_members,
    })
}

fn filter_tests_from_module_member(
    context: &mut Context,
    module_member: P::ModuleMember,
) -> Option<P::ModuleMember> {
    use P::ModuleMember as PM;
    let attrs = match &module_member {
        PM::Function(func) => &func.attributes,
        PM::Struct(strct) => &strct.attributes,
        PM::Spec(sp!(_, spec)) => &spec.attributes,
        PM::Use(use_decl) => &use_decl.attributes,
        PM::Friend(friend_decl) => &friend_decl.attributes,
        PM::Constant(constant) => &constant.attributes,
    };

    if should_remove_node(context.env, attrs) {
        None
    } else {
        Some(module_member)
    }
}

// A module member should be removed if:
// * It is annotated as a test function (test_only, test, abort); and
// * Test mode is not set
fn should_remove_node(env: &CompilationEnv, attrs: &[P::Attributes]) -> bool {
    attrs.iter().flat_map(test_attributes).count() > 0 && !env.flags().is_testing()
}

fn test_attributes(attrs: &P::Attributes) -> Vec<(Loc, known_attributes::TestingAttributes)> {
    attrs
        .value
        .iter()
        .filter_map(|attr| {
            known_attributes::TestingAttributes::resolve(&attr.value.attribute_name().value)
                .map(|test_attr| (attr.loc, test_attr))
        })
        .collect()
}
