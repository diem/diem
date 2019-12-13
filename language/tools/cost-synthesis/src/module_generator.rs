// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Logic for random valid module and module universe generation.
//!
//! This module contains the logic for generating random valid modules and valid (rooted) module
//! universes. Note that we do not generate valid function bodies for the functions that are
//! generated -- any function bodies that are generated are simply non-semantic sequences of
//! instructions to check BrTrue, BrFalse, and Branch instructions.
use bytecode_verifier::VerifiedModule;
use utils::module_generation::{generate_verified_modules, ModuleGeneratorOptions};

pub fn generate_padded_modules(
    num: usize,
    table_size: usize,
) -> (VerifiedModule, Vec<VerifiedModule>) {
    let mut generation_options = ModuleGeneratorOptions::default();
    // We don't handle type parameter generation
    generation_options.max_ty_params = 1;
    generation_options.min_fields = 1;
    generation_options.min_table_size = table_size;
    generate_verified_modules(num, generation_options)
}
