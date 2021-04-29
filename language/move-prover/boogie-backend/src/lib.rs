// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use std::collections::BTreeSet;

use itertools::Itertools;
#[allow(unused_imports)]
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use tera::{Context, Tera};

use bytecode::mono_analysis;
use move_model::{
    code_writer::CodeWriter,
    emit, emitln,
    model::GlobalEnv,
    ty::{PrimitiveType, Type},
};

use crate::{
    boogie_helpers::{boogie_type, boogie_type_suffix},
    bytecode_translator::has_native_equality,
    options::{BoogieOptions, VectorTheory},
};

const PRELUDE_TEMPLATE: &[u8] = include_bytes!("prelude/prelude.bpl");
const NATIVE_TEMPLATE: &[u8] = include_bytes!("prelude/native.bpl");
const VECTOR_ARRAY_THEORY: &[u8] = include_bytes!("prelude/vector-array-theory.bpl");
const VECTOR_ARRAY_INTERN_THEORY: &[u8] = include_bytes!("prelude/vector-array-intern-theory.bpl");
const VECTOR_SMT_SEQ_THEORY: &[u8] = include_bytes!("prelude/vector-smt-seq-theory.bpl");
const VECTOR_SMT_ARRAY_THEORY: &[u8] = include_bytes!("prelude/vector-smt-array-theory.bpl");
const VECTOR_SMT_ARRAY_EXT_THEORY: &[u8] =
    include_bytes!("prelude/vector-smt-array-ext-theory.bpl");
const MULTISET_ARRAY_THEORY: &[u8] = include_bytes!("prelude/multiset-array-theory.bpl");

const BCS_MODULE: &str = "0x1::BCS";
const EVENT_MODULE: &str = "0x1::Event";

mod boogie_helpers;
pub mod boogie_wrapper;
pub mod bytecode_translator;
pub mod options;
mod prover_task_runner;
mod spec_translator;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
struct TypeInfo {
    name: String,
    suffix: String,
    has_native_equality: bool,
}

/// Adds the prelude to the generated output.
pub fn add_prelude(
    env: &GlobalEnv,
    options: &BoogieOptions,
    writer: &CodeWriter,
) -> anyhow::Result<()> {
    emit!(writer, "\n// ** Expanded prelude\n\n");
    let templ = |name: &'static str, cont: &[u8]| (name, String::from_utf8_lossy(cont).to_string());

    // Add the prelude template.
    let mut templates = vec![
        templ("native", NATIVE_TEMPLATE),
        templ("prelude", PRELUDE_TEMPLATE),
        // Add the basic array theory to make it available for inclusion in other theories.
        templ("vector-array-theory", VECTOR_ARRAY_THEORY),
    ];

    // Bind the chosen vector and multiset theory
    let vector_theory = match options.vector_theory {
        VectorTheory::BoogieArray => VECTOR_ARRAY_THEORY,
        VectorTheory::BoogieArrayIntern => VECTOR_ARRAY_INTERN_THEORY,
        VectorTheory::SmtArray => VECTOR_SMT_ARRAY_THEORY,
        VectorTheory::SmtArrayExt => VECTOR_SMT_ARRAY_EXT_THEORY,
        VectorTheory::SmtSeq => VECTOR_SMT_SEQ_THEORY,
    };
    templates.push(templ("vector-theory", vector_theory));
    templates.push(templ("multiset-theory", MULTISET_ARRAY_THEORY));

    let mut tera = Tera::default();
    tera.add_raw_templates(templates)?;

    let mut context = Context::new();
    context.insert("options", options);

    let mono_info = mono_analysis::get_info(env);
    // Add vector instances implicitly used by the prelude.
    let implicit_vec_inst = vec![TypeInfo::new(
        env,
        options,
        &Type::Primitive(PrimitiveType::U8),
    )];
    let vec_instances = mono_info
        .vec_inst
        .iter()
        .map(|ty| TypeInfo::new(env, options, ty))
        .chain(implicit_vec_inst.into_iter())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect_vec();
    context.insert("vec_instances", &vec_instances);
    let filter_native = |module: &str| {
        mono_info
            .native_inst
            .iter()
            .filter(|(id, _)| env.get_module(**id).get_full_name_str() == module)
            .map(|(_, insts)| {
                insts
                    .iter()
                    .map(|inst| TypeInfo::new(env, options, &inst[0]))
            })
            .flatten()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect_vec()
    };
    let bcs_instances = filter_native(BCS_MODULE);
    context.insert("bcs_instances", &bcs_instances);
    let event_instances = filter_native(EVENT_MODULE);
    context.insert("event_instances", &event_instances);

    let expanded_content = tera.render("prelude", &context)?;
    emitln!(writer, &expanded_content);
    Ok(())
}

impl TypeInfo {
    fn new(env: &GlobalEnv, options: &BoogieOptions, ty: &Type) -> Self {
        Self {
            name: boogie_type(env, ty),
            suffix: boogie_type_suffix(env, ty),
            has_native_equality: has_native_equality(env, options, ty),
        }
    }
}
