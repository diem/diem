// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use boogie_backend::{
    options::{BoogieOptions, VectorTheory},
    prelude_template_helpers::StratificationHelper,
};
use handlebars::Handlebars;
#[allow(unused_imports)]
use log::{debug, info, warn};
use move_model::{code_writer::CodeWriter, emit, emitln};
use serde::{Deserialize, Serialize};

pub const PRELUDE_TEMPLATE: &[u8] = include_bytes!("prelude/prelude.bpl");
pub const VECTOR_ARRAY_THEORY: &[u8] = include_bytes!("prelude/vector-array-theory.bpl");
pub const VECTOR_ARRAY_INTERN_THEORY: &[u8] =
    include_bytes!("prelude/vector-array-intern-theory.bpl");
pub const VECTOR_SMT_SEQ_THEORY: &[u8] = include_bytes!("prelude/vector-smt-seq-theory.bpl");
pub const VECTOR_SMT_ARRAY_THEORY: &[u8] = include_bytes!("prelude/vector-smt-array-theory.bpl");
pub const VECTOR_SMT_ARRAY_EXT_THEORY: &[u8] =
    include_bytes!("prelude/vector-smt-array-ext-theory.bpl");
pub const MULTISET_ARRAY_THEORY: &[u8] = include_bytes!("prelude/multiset-array-theory.bpl");

mod boogie_helpers;
pub mod boogie_wrapper;
pub mod bytecode_translator;
mod spec_translator;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
struct TemplateOptions {
    options: BoogieOptions,
}

/// Adds the prelude to the generated output.
pub fn add_prelude(options: &BoogieOptions, writer: &CodeWriter) -> anyhow::Result<()> {
    emit!(writer, "\n// ** Expanded prelude\n\n");
    let content = String::from_utf8_lossy(PRELUDE_TEMPLATE).to_string();
    let mut handlebars = Handlebars::new();
    handlebars.set_strict_mode(true);
    handlebars.register_helper(
        "stratified",
        Box::new(StratificationHelper::new(options.stratification_depth)),
    );
    // Bind the basic array theory to make it available for inclusion in other theories.
    handlebars.register_template_string(
        "vector-array-theory",
        String::from_utf8_lossy(VECTOR_ARRAY_THEORY).to_string(),
    )?;
    // Bind the chosen vector and multiset theory
    let vector_theory = match options.vector_theory {
        VectorTheory::BoogieArray => VECTOR_ARRAY_THEORY,
        VectorTheory::BoogieArrayIntern => VECTOR_ARRAY_INTERN_THEORY,
        VectorTheory::SmtArray => VECTOR_SMT_ARRAY_THEORY,
        VectorTheory::SmtArrayExt => VECTOR_SMT_ARRAY_EXT_THEORY,
        VectorTheory::SmtSeq => VECTOR_SMT_SEQ_THEORY,
    };
    handlebars.register_template_string(
        "vector-theory",
        String::from_utf8_lossy(vector_theory).to_string(),
    )?;
    handlebars.register_template_string(
        "multiset-theory",
        String::from_utf8_lossy(MULTISET_ARRAY_THEORY).to_string(),
    )?;

    let template_options = TemplateOptions {
        options: options.clone(),
    };
    let expanded_content = handlebars.render_template(&content, &template_options)?;
    emitln!(writer, &expanded_content);
    Ok(())
}
