// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    cfgir::ast as G,
    expansion::ast as E,
    shared::{known_attributes, Address, CompilationEnv, Identifier},
    unit_test::{Context, ExpectedFailure, ModuleTestPlan, TestCase},
};
use move_core_types::{account_address::AccountAddress as MoveAddress, value::MoveValue};
use move_ir_types::location::Loc;
use std::collections::BTreeMap;

//***************************************************************************
// Test Plan Building
//***************************************************************************

// Constructs a test plan for each module in `prog`. This also validates the structure of the
// attributes as the test plan is constructed.
pub fn construct_test_plan(
    compilation_env: &mut CompilationEnv,
    prog: &G::Program,
) -> Option<Vec<ModuleTestPlan>> {
    if !compilation_env.flags().is_testing() {
        return None;
    }
    let mut context = Context::new(compilation_env);
    Some(
        prog.modules
            .iter()
            .flat_map(|(_, module_ident, module_def)| {
                construct_module_test_plan(&mut context, module_ident, module_def)
            })
            .collect(),
    )
}

fn construct_module_test_plan(
    context: &mut Context,
    module_ident: &(Address, String),
    module: &G::ModuleDefinition,
) -> Option<ModuleTestPlan> {
    let tests: BTreeMap<_, _> = module
        .functions
        .iter()
        .filter_map(|(loc, fn_name, func)| {
            build_test_info(context, loc, fn_name, func)
                .map(|test_case| (fn_name.to_string(), test_case))
        })
        .collect();

    if tests.is_empty() {
        None
    } else {
        Some(ModuleTestPlan::new(module_ident, tests))
    }
}

fn build_test_info(
    context: &mut Context,
    fn_loc: Loc,
    fn_name: &str,
    function: &G::Function,
) -> Option<TestCase> {
    let get_attrs = |attr_name: &'static str| {
        function
            .attributes
            .iter()
            .filter(|attr| attr.value.attribute_name().value == attr_name)
            .collect::<Vec<_>>()
    };

    let previously_annotated_msg = "Previously annotated here";
    let in_this_test_msg = "In this test";
    let make_duplicate_msg = |attr_name: &str| {
        format!(
            "Duplicate '#[{0}]' attribute. Only one #[{0}] attribute is allowed",
            attr_name
        )
    };

    let test_attributes = get_attrs(known_attributes::TestingAttributes::TEST);
    let test_only_attributes = get_attrs(known_attributes::TestingAttributes::TEST_ONLY);
    let abort_attributes = get_attrs(known_attributes::TestingAttributes::EXPECTED_FAILURE);

    // expected failures cannot be annotated on non-#[test] functions
    if !abort_attributes.is_empty() && test_attributes.is_empty() {
        let fn_msg =  "Only functions defined as a test with #[test] can also have an #[expected_failure] attribute";
        let abort_msg = "Attributed as #[expected_failure] here";
        context.env.add_error(vec![
            (fn_loc, fn_msg),
            (abort_attributes.last().unwrap().loc, abort_msg),
        ])
    }

    if test_attributes.is_empty() {
        return None;
    }

    // Check for duplicate #[test(...)] attributes
    if test_attributes.len() > 1 {
        let len = test_attributes.len();
        context.env.add_error(vec![
            (
                test_attributes[len - 1].loc,
                make_duplicate_msg(known_attributes::TestingAttributes::TEST),
            ),
            (
                test_attributes[len - 2].loc,
                previously_annotated_msg.into(),
            ),
            (fn_loc, in_this_test_msg.into()),
        ])
    }

    // A #[test] function cannot also be annotated #[test_only]
    if !test_only_attributes.is_empty() {
        let msg = "Function annotated as both #[test(...)] and #[test_only]. You need to declare it as either one or the other";
        context.env.add_error(vec![
            (test_only_attributes.last().unwrap().loc, msg),
            (
                test_attributes.last().unwrap().loc,
                previously_annotated_msg,
            ),
            (fn_loc, in_this_test_msg),
        ])
    }

    // Only one abort can be specified
    if abort_attributes.len() > 1 {
        let len = abort_attributes.len();
        context.env.add_error(vec![
            (
                abort_attributes[len - 1].loc,
                make_duplicate_msg(known_attributes::TestingAttributes::EXPECTED_FAILURE),
            ),
            (
                abort_attributes[len - 2].loc,
                previously_annotated_msg.into(),
            ),
        ])
    }

    let test_attribute = test_attributes.last().unwrap();
    let test_annotation_params = parse_test_attribute(context, test_attribute);
    let mut arguments = Vec::new();

    for (var, _) in &function.signature.parameters {
        match test_annotation_params.get(var.value()) {
            Some(value) => arguments.push(value.clone()),
            None => {
                let missing_param_msg = "Missing test parameter assignment in test. Expected a parameter to be assigned in this attribute";
                context.env.add_error(vec![
                    (test_attribute.loc, missing_param_msg),
                    (var.loc(), "Corresponding to this parameter"),
                    (fn_loc, in_this_test_msg),
                ])
            }
        }
    }

    let expected_failure = if abort_attributes.is_empty() {
        None
    } else {
        parse_failure_attribute(context, abort_attributes.last().unwrap())
    };

    Some(TestCase {
        test_name: fn_name.to_string(),
        arguments,
        expected_failure,
    })
}

//***************************************************************************
// Attribute parsers
//***************************************************************************

fn parse_test_attribute(
    context: &mut Context,
    sp!(aloc, test_attribute): &E::Attribute,
) -> BTreeMap<String, MoveValue> {
    use E::Attribute_ as EA;

    match test_attribute {
        EA::Name(nm) => {
            assert!(
                nm.value == known_attributes::TestingAttributes::TEST,
                "ICE: We should only be parsing a raw test attribute"
            );
            BTreeMap::new()
        }
        EA::Assigned(nm, sp!(assign_loc, attr_value)) => {
            let value = match convert_attribute_value_to_move_value(attr_value) {
                Some(move_value) => move_value,
                None => {
                    context.env.add_error(vec![
                        (*assign_loc, "Unsupported attribute value"),
                        (*aloc, "Assigned in this attribute"),
                    ]);
                    return BTreeMap::new();
                }
            };

            let mut args = BTreeMap::new();
            args.insert(nm.value.to_string(), value);
            args
        }
        EA::Parameterized(nm, attributes) => {
            assert!(
                nm.value == known_attributes::TestingAttributes::TEST,
                "ICE: We should only be parsing a raw test attribute"
            );
            attributes
                .iter()
                .flat_map(|attr| parse_test_attribute(context, attr))
                .collect()
        }
    }
}

fn parse_failure_attribute(
    context: &mut Context,
    sp!(aloc, expected_attr): &E::Attribute,
) -> Option<ExpectedFailure> {
    use E::{AttributeValue_ as EAV, Attribute_ as EA, Value_ as EV};
    match expected_attr {
        EA::Name(nm) => {
            assert!(
                nm.value == known_attributes::TestingAttributes::EXPECTED_FAILURE,
                "ICE: We should only be parsing a raw expected failure attribute"
            );
            Some(ExpectedFailure::Expected)
        }
        EA::Assigned(_, sp!(assign_loc, _)) => {
            let invalid_assignment_msg = "Invalid expected failure code assignment";
            let expected_msg = format!(
                "Expect an #[expected_failure({}=...)] attribute for abort code assignment",
                known_attributes::TestingAttributes::CODE_ASSIGNMENT_NAME
            );
            context.env.add_error(vec![
                (*assign_loc, invalid_assignment_msg.into()),
                (*aloc, expected_msg),
            ]);
            None
        }
        EA::Parameterized(sp!(_, nm), attrs) => {
            if attrs.len() != 1 {
                let invalid_attr_msg = format!(
                    "Invalid #[expected_failure(...)] attribute, expected 1 argument but found {}",
                    attrs.len()
                );
                context.env.add_error(vec![(*aloc, invalid_attr_msg)]);
                return None;
            }
            assert!(
                nm == known_attributes::TestingAttributes::EXPECTED_FAILURE,
                "ICE: expected failure attribute must have the right name"
            );
            match attrs.last().unwrap() {
                sp!(assign_loc, EA::Assigned(sp!(_, nm), value))
                    if nm == known_attributes::TestingAttributes::CODE_ASSIGNMENT_NAME =>
                {
                    match value {
                        sp!(_, EAV::Value(true, sp!(_, EV::U128(u))))
                            if *u <= std::u64::MAX as u128 =>
                        {
                            Some(ExpectedFailure::ExpectedWithCode(*u as u64))
                        }
                        sp!(_, EAV::Value(false, sp!(_, EV::U64(u)))) => {
                            Some(ExpectedFailure::ExpectedWithCode(*u))
                        }
                        sp!(vloc, EAV::Value(false, sp!(_, EV::U8(_))))
                        | sp!(vloc, EAV::Value(false, sp!(_, EV::U128(_)))) => {
                            context.env.add_error(vec![
                                (
                                    *assign_loc,
                                    "Invalid value in expected failure code assignment",
                                ),
                                (*vloc, "Annotated non-u64 literals are not permitted"),
                            ]);
                            None
                        }
                        sp!(vloc, _) => {
                            context.env.add_error(vec![
                                (*vloc, "Invalid value in expected failure code assignment"),
                                (*assign_loc, "In this assignment"),
                            ]);
                            None
                        }
                    }
                }
                sp!(assign_loc, EA::Assigned(sp!(nmloc, _), _)) => {
                    let invalid_name_msg = format!(
                                "Invalid name in expected failure code assignment. Did you mean to use '{}'?",
                                known_attributes::TestingAttributes::CODE_ASSIGNMENT_NAME
                            );
                    context.env.add_error(vec![
                        (*nmloc, invalid_name_msg),
                        (*assign_loc, "In this assignment".into()),
                    ]);
                    None
                }
                sp!(loc, _) => {
                    let msg = "Unsupported attribute value for expected failure attribute";
                    context
                        .env
                        .add_error(vec![(*aloc, msg), (*loc, "In this assignment")]);
                    None
                }
            }
        }
    }
}

fn convert_attribute_value_to_move_value(value: &E::AttributeValue_) -> Option<MoveValue> {
    use E::{AttributeValue_ as EAV, Value_ as EV};
    match value {
        // Only addresses are allowed
        EAV::Value(_, sp!(_, EV::Address(a))) => {
            Some(MoveValue::Address(MoveAddress::new(a.to_u8())))
        }
        _ => None,
    }
}
