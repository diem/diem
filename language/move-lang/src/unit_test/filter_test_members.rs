// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    diag,
    diagnostics::Diagnostics,
    parser::ast as P,
    shared::{known_attributes, CompilationEnv},
};
use move_ir_types::location::{sp, Loc};

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

//***************************************************************************
// Filtering of test-annotated module members
//***************************************************************************

const UNIT_TEST_MODULE_NAME: &str = "UnitTest";
const STDLIB_ADDRESS_NAME: &str = "Std";

// This filters out all test, and test-only annotated module member from `prog` if the `test` flag
// in `compilation_env` is not set. If the test flag is set, no filtering is performed, and instead
// a test plan is created for use by the testing framework.
pub fn program(compilation_env: &mut CompilationEnv, prog: P::Program) -> P::Program {
    let mut context = Context::new(compilation_env);

    if !check_has_unit_test_module(&mut context, &prog) {
        return prog;
    }

    let P::Program {
        source_definitions,
        lib_definitions,
    } = prog;

    let lib_definitions: Vec<_> = lib_definitions
        .into_iter()
        .filter_map(|def| filter_tests_from_definition(&mut context, def, false))
        .collect();

    let source_definitions: Vec<_> = source_definitions
        .into_iter()
        .filter_map(|def| filter_tests_from_definition(&mut context, def, true))
        .collect();

    P::Program {
        source_definitions,
        lib_definitions,
    }
}

fn check_has_unit_test_module(context: &mut Context, prog: &P::Program) -> bool {
    let has_unit_test_module = prog
        .lib_definitions
        .iter()
        .chain(prog.source_definitions.iter())
        .any(|def| match def {
            P::Definition::Module(mdef) => {
                mdef.name.0.value == UNIT_TEST_MODULE_NAME
                    && mdef.address.is_some()
                    && match &mdef.address.as_ref().unwrap().value {
                        // TODO: remove once named addresses have landed in the stdlib
                        P::LeadingNameAccess_::Name(name) => name.value == STDLIB_ADDRESS_NAME,
                        P::LeadingNameAccess_::AnonymousAddress(_) => false,
                    }
            }
            _ => false,
        });

    if !has_unit_test_module && context.env.flags().is_testing() {
        if let Some(def) = prog
            .source_definitions
            .iter()
            .chain(prog.lib_definitions.iter())
            .next()
        {
            let loc = match def {
                P::Definition::Module(P::ModuleDefinition { name, .. }) => name.0.loc,
                P::Definition::Address(P::AddressDefinition { loc, .. })
                | P::Definition::Script(P::Script { loc, .. }) => *loc,
            };
            context.env.add_diag(diag!(
                Attributes::InvalidTest,
                (
                    loc,
                    "Compilation in test mode requires passing the UnitTest module in the Move \
                     stdlib as a dependency",
                )
            ));
            return false;
        }
    }

    true
}

fn filter_tests_from_definition(
    context: &mut Context,
    def: P::Definition,
    is_source_def: bool,
) -> Option<P::Definition> {
    match def {
        P::Definition::Module(m) => {
            filter_tests_from_module(context, m, is_source_def).map(P::Definition::Module)
        }
        P::Definition::Address(a) => {
            let P::AddressDefinition {
                addr,
                attributes,
                loc,
                modules,
            } = a;
            if !should_remove_node(context.env, &attributes, is_source_def) {
                let modules = modules
                    .into_iter()
                    .filter_map(|m| filter_tests_from_module(context, m, is_source_def))
                    .collect();
                Some(P::Definition::Address(P::AddressDefinition {
                    attributes,
                    loc,
                    addr,
                    modules,
                }))
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

            let diags: Diagnostics = script_attributes
                .flat_map(|attr| {
                    test_attributes(attr).into_iter().map(|(loc, _)| {
                        let msg = "Testing attributes are not allowed in scripts.";
                        diag!(Attributes::InvalidTest, (loc, msg))
                    })
                })
                .collect();

            if diags.is_empty() {
                Some(P::Definition::Script(P::Script {
                    attributes,
                    loc,
                    uses,
                    constants,
                    function,
                    specs,
                }))
            } else {
                context.env.add_diags(diags);
                None
            }
        }
    }
}

fn filter_tests_from_module(
    context: &mut Context,
    module_def: P::ModuleDefinition,
    is_source_def: bool,
) -> Option<P::ModuleDefinition> {
    if should_remove_node(context.env, &module_def.attributes, is_source_def) {
        return None;
    }

    let P::ModuleDefinition {
        attributes,
        loc,
        address,
        name,
        is_spec_module,
        members,
    } = module_def;

    let mut new_members: Vec<_> = members
        .into_iter()
        .filter_map(|member| filter_tests_from_module_member(context, member, is_source_def))
        .collect();

    insert_test_poison(context, loc, &mut new_members);

    Some(P::ModuleDefinition {
        attributes,
        loc,
        address,
        name,
        is_spec_module,
        members: new_members,
    })
}

/// If a module is being compiled in test mode, this inserts a function that calls a native
/// function `0x1::UnitTest::create_signers_for_testing` that only exists if the VM is being run
/// with the "unit_test" feature flag set. This will then cause the module to fail to link if
/// an attempt is made to publish a module that has been compiled in test mode on a VM that is not
/// running in test mode.
fn insert_test_poison(context: &mut Context, mloc: Loc, members: &mut Vec<P::ModuleMember>) {
    if !context.env.flags().is_testing() {
        return;
    }

    let signature = P::FunctionSignature {
        type_parameters: vec![],
        parameters: vec![],
        return_type: sp(mloc, P::Type_::Unit),
    };

    let leading_name_access = sp(
        mloc,
        P::LeadingNameAccess_::Name(sp(mloc, STDLIB_ADDRESS_NAME.to_owned())),
    );

    let mod_name = sp(mloc, UNIT_TEST_MODULE_NAME.to_string());
    let mod_addr_name = sp(mloc, (leading_name_access, mod_name));
    let fn_name = sp(mloc, "create_signers_for_testing".to_string());
    let args_ = vec![sp(
        mloc,
        P::Exp_::Value(sp(mloc, P::Value_::Num("0".to_string()))),
    )];
    let nop_call = P::Exp_::Call(
        sp(mloc, P::NameAccessChain_::Three(mod_addr_name, fn_name)),
        None,
        sp(mloc, args_),
    );

    // fun unit_test_poison() { 0x1::UnitTest::create_signers_for_testing(0); () }
    let function = P::Function {
        attributes: vec![],
        loc: mloc,
        visibility: P::Visibility::Internal,
        acquires: vec![],
        signature,
        name: P::FunctionName(sp(mloc, "unit_test_poison".to_string())),
        body: sp(
            mloc,
            P::FunctionBody_::Defined((
                vec![],
                vec![sp(
                    mloc,
                    P::SequenceItem_::Seq(Box::new(sp(mloc, nop_call))),
                )],
                None,
                Box::new(Some(sp(mloc, P::Exp_::Unit))),
            )),
        ),
    };

    members.push(P::ModuleMember::Function(function));
}

fn filter_tests_from_module_member(
    context: &mut Context,
    module_member: P::ModuleMember,
    is_source_def: bool,
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

    if should_remove_node(context.env, attrs, is_source_def) {
        None
    } else {
        Some(module_member)
    }
}

// A module member should be removed if:
// * It is annotated as a test function (test_only, test, abort) and test mode is not set; or
// * If it is a library and is annotated as #[test]
fn should_remove_node(env: &CompilationEnv, attrs: &[P::Attributes], is_source_def: bool) -> bool {
    let flattened_attrs: Vec<_> = attrs.iter().flat_map(test_attributes).collect();
    !flattened_attrs.is_empty() && !env.flags().is_testing()
        || (!is_source_def
            && flattened_attrs
                .iter()
                .any(|attr| attr.1 == known_attributes::TestingAttributes::Test))
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
