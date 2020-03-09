// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module translates the bytecode of a module to Boogie code.

use std::collections::{BTreeMap, BTreeSet};

use itertools::Itertools;
#[allow(unused_imports)]
use log::{debug, info};

use spec_lang::env::{FunctionEnv, GlobalEnv, Loc, ModuleEnv, Parameter, StructEnv, TypeParameter};
use spec_lang::ty::{PrimitiveType, Type};
use stackless_bytecode_generator::{
    stackless_bytecode::StacklessBytecode::{self, *},
    stackless_bytecode_generator::{StacklessFunction, StacklessModuleGenerator},
};

use crate::cli::Options;
use crate::spec_translator::SpecTranslator;
use crate::{
    boogie_helpers::{
        boogie_field_name, boogie_function_name, boogie_local_type, boogie_struct_name,
        boogie_struct_type_value, boogie_type_check, boogie_type_value, boogie_type_values,
        boogie_var_before_borrow,
    },
    code_writer::CodeWriter,
};
use bytecode_to_boogie::lifetime_analysis::LifetimeAnalysis;
use bytecode_to_boogie::stackless_control_flow_graph::StacklessControlFlowGraph;
use num::Zero;

pub struct BoogieTranslator<'env> {
    env: &'env GlobalEnv,
    writer: &'env CodeWriter,
    options: &'env Options,
}

pub struct ModuleTranslator<'env> {
    writer: &'env CodeWriter,
    options: &'env Options,
    module_env: ModuleEnv<'env>,
    stackless_bytecode: Vec<StacklessFunction>,
}

impl<'env> BoogieTranslator<'env> {
    pub fn new(env: &'env GlobalEnv, options: &'env Options, writer: &'env CodeWriter) -> Self {
        Self {
            env,
            writer,
            options,
        }
    }

    pub fn translate(&mut self) {
        // generate definitions for all modules.
        for module_env in self.env.get_modules() {
            ModuleTranslator::new(self, module_env).translate();
        }
    }
}

impl<'env> ModuleTranslator<'env> {
    /// Creates a new module translator. Calls the stackless bytecode generator and wraps
    /// result into the translator.
    fn new(parent: &'env BoogieTranslator, module: ModuleEnv<'env>) -> Self {
        let stackless_bytecode =
            StacklessModuleGenerator::new(module.get_verified_module()).generate_module();
        Self {
            writer: parent.writer,
            options: parent.options,
            module_env: module,
            stackless_bytecode,
        }
    }

    /// Returns true if for the module no code should be produced because its already defined
    /// in the prelude.
    pub fn is_module_provided_by_prelude(&self) -> bool {
        let name = self.module_env.get_name();
        self.module_env.symbol_pool().string(name.name()).as_str() == "Vector"
            && name.addr().is_zero()
    }

    /// Translates this module.
    fn translate(&mut self) {
        if self.is_module_provided_by_prelude() {
            return;
        }
        info!(
            "translating module {}",
            self.module_env
                .get_name()
                .display(self.module_env.symbol_pool())
        );
        self.writer
            .set_location(&self.module_env.env.internal_loc());
        let spec_translator = SpecTranslator::new(self.writer, &self.module_env, false);
        spec_translator.translate_spec_vars();
        spec_translator.translate_spec_funs();
        self.translate_structs();
        self.translate_functions();
    }

    /// Translates all structs in the module.
    fn translate_structs(&self) {
        emitln!(
            self.writer,
            "\n\n// ** structs of module {}\n",
            self.module_env
                .get_name()
                .display(self.module_env.symbol_pool())
        );
        for struct_env in self.module_env.get_structs() {
            // Set the location to internal so we don't see locations of pack/unpack
            // in execution traces.
            self.writer
                .set_location(&self.module_env.env.internal_loc());
            self.translate_struct_type(&struct_env);
            if !struct_env.is_native() {
                self.translate_struct_accessors(&struct_env);
            }
        }
    }

    /// Translates the given struct.
    fn translate_struct_type(&self, struct_env: &StructEnv<'_>) {
        // Emit TypeName
        let struct_name = boogie_struct_name(&struct_env);
        emitln!(self.writer, "const unique {}: TypeName;", struct_name);

        // Emit FieldNames
        for (i, field_env) in struct_env.get_fields().enumerate() {
            let field_name = boogie_field_name(&field_env);
            emitln!(
                self.writer,
                "const {}: FieldName;\naxiom {} == {};",
                field_name,
                field_name,
                i
            );
        }

        // Emit TypeValue constructor function.
        let type_args = struct_env
            .get_type_parameters()
            .iter()
            .enumerate()
            .map(|(i, _)| format!("$tv{}: TypeValue", i))
            .join(", ");
        let mut field_types = String::from("EmptyTypeValueArray");
        for field_env in struct_env.get_fields() {
            field_types = format!(
                "ExtendTypeValueArray({}, {})",
                field_types,
                boogie_type_value(self.module_env.env, &field_env.get_type())
            );
        }
        let type_value = format!("StructType({}, {})", struct_name, field_types);
        if struct_name == "LibraAccount_T" {
            // Special treatment of well-known resource LibraAccount_T. The type_value
            // function is forward-declared in the prelude, here we only add an axiom for
            // it.
            emitln!(
                self.writer,
                "axiom {}_type_value() == {};",
                struct_name,
                type_value
            );
        } else {
            emitln!(
                self.writer,
                "function {}_type_value({}): TypeValue {{\n    {}\n}}",
                struct_name,
                type_args,
                type_value
            );
        }

        // Emit invariant functions.
        let spec_translator = SpecTranslator::new(self.writer, &struct_env.module_env, false);
        spec_translator.translate_invariant_functions(&struct_env);
    }

    /// Translates struct accessors (pack/unpack).
    fn translate_struct_accessors(&self, struct_env: &StructEnv<'_>) {
        // Pack function
        let type_args_str = struct_env
            .get_type_parameters()
            .iter()
            .map(|TypeParameter(s, _)| {
                format!("{}: TypeValue", s.display(struct_env.symbol_pool()))
            })
            .join(", ");
        let args_str = struct_env
            .get_fields()
            .map(|field_env| {
                format!(
                    "{}: Value",
                    field_env.get_name().display(struct_env.symbol_pool())
                )
            })
            .join(", ");
        emitln!(
            self.writer,
            "procedure {{:inline 1}} {}_pack($file_id: int, $byte_index: int, $var_idx: int, {}) returns ($struct: Value)\n{{",
            boogie_struct_name(struct_env),
            separate(vec![type_args_str, args_str.clone()], ", ")
        );
        self.writer.indent();
        let mut fields_str = String::from("EmptyValueArray");
        for field_env in struct_env.get_fields() {
            let type_check = boogie_type_check(
                self.module_env.env,
                &format!("{}", field_env.get_name().display(struct_env.symbol_pool())),
                &field_env.get_type(),
            );
            emit!(self.writer, &type_check);
            fields_str = format!(
                "ExtendValueArray({}, {})",
                fields_str,
                field_env.get_name().display(struct_env.symbol_pool())
            );
        }
        emitln!(self.writer, "$struct := Vector({});", fields_str);

        // Generate $DebugTrackLocal so we can see the constructed value before invariant
        // evaluation may abort.
        emitln!(
            self.writer,
            "if ($byte_index > 0) { assume $DebugTrackLocal($file_id, $byte_index, $var_idx, $struct); }"
        );

        // Insert invariant code.
        let spec_translator = SpecTranslator::new(self.writer, &struct_env.module_env, false);
        spec_translator.emit_pack_invariants(struct_env, "$struct");

        self.writer.unindent();
        emitln!(self.writer, "}\n");

        // Unpack function
        emitln!(
            self.writer,
            "procedure {{:inline 1}} {}_unpack($struct: Value) returns ({})\n{{",
            boogie_struct_name(struct_env),
            args_str
        );
        self.writer.indent();
        emitln!(self.writer, "assume is#Vector($struct);");
        for field_env in struct_env.get_fields() {
            emitln!(
                self.writer,
                "{} := $SelectField($struct, {});",
                field_env.get_name().display(struct_env.symbol_pool()),
                boogie_field_name(&field_env)
            );
            let type_check = boogie_type_check(
                self.module_env.env,
                &format!("{}", field_env.get_name().display(struct_env.symbol_pool())),
                &field_env.get_type(),
            );
            emit!(self.writer, &type_check);
        }

        // Insert invariant checking code.
        let spec_translator = SpecTranslator::new(self.writer, &struct_env.module_env, false);
        spec_translator.emit_unpack_invariants(struct_env, "$struct");

        self.writer.unindent();
        emitln!(self.writer, "}\n");
    }

    /// Translates all functions in the module.
    fn translate_functions(&self) {
        emitln!(
            self.writer,
            "\n\n// ** functions of module {}\n",
            self.module_env
                .get_name()
                .display(self.module_env.symbol_pool())
        );
        let mut num_fun_specified = 0;
        let mut num_fun = 0;
        for func_env in self.module_env.get_functions() {
            if !func_env.is_native() {
                num_fun += 1;
            }
            if !func_env.get_specification().is_empty() && !func_env.is_native() {
                num_fun_specified += 1;
            }
            self.writer.set_location(&func_env.get_loc());
            self.translate_function(&func_env);
        }
        if num_fun > 0 {
            info!(
                "{} out of {} functions are specified in module {}",
                num_fun_specified,
                num_fun,
                self.module_env
                    .get_name()
                    .display(self.module_env.symbol_pool())
            );
        }
    }

    /// Translates the given function.
    fn translate_function(&self, func_env: &FunctionEnv<'_>) {
        if func_env.is_native() {
            if self.options.native_stubs {
                self.generate_function_sig(func_env, true);
                emit!(self.writer, ";");
                self.generate_function_spec(func_env);
                emitln!(self.writer);
            }
            return;
        }

        // generate inline function with function body
        self.generate_function_sig(func_env, true); // inlined version of function
        self.generate_function_spec(func_env);
        self.generate_inline_function_body(func_env);
        emitln!(self.writer);

        // generate the _verify version of the function which calls inline version for standalone
        // verification.
        self.generate_function_sig(func_env, false); // no inline
        self.generate_verify_function_body(func_env); // function body just calls inlined version
    }

    /// Translates one bytecode instruction.
    fn translate_bytecode<F: Fn(usize) -> usize>(
        &'env self,
        func_env: &FunctionEnv<'_>,
        offset: u16,
        bytecode: &StacklessBytecode,
        effective_dest_func: F,
        mutual_local_borrows: &BTreeSet<usize>,
        local_to_before_borrow_idx: &BTreeMap<usize, usize>,
        released_before_borrows: &BTreeSet<usize>,
    ) {
        // Set location of this code in the CodeWriter.
        let loc = func_env.get_bytecode_loc(offset);
        self.writer.set_location(&loc);
        // emitln!(self.writer, "// {}", loc); // DEBUG

        // Helper functions to update a local including debug information.
        let update_and_track_local = |idx: usize, value: &str| {
            self.update_and_track_local(func_env, loc.clone(), idx, value)
        };
        let track_local = |idx: usize, value: &str| {
            if idx < func_env.get_local_count() {
                self.track_local(func_env, loc.clone(), idx, value)
            } else {
                "".to_string()
            }
        };
        let track_return = |idx: usize| {
            self.track_local(
                func_env,
                loc.clone(),
                func_env.get_local_count() + idx,
                &format!("$ret{}", idx),
            )
        };

        // Helper function to update debug info for potential updates of references.
        let track_mutual_refs = || {
            let mut res = vec![];
            for idx in mutual_local_borrows {
                res.push(self.track_local(
                    func_env,
                    loc.clone(),
                    *idx,
                    &format!("$GetLocal($m, $frame + {})", idx),
                ));
            }
            for (idx, Parameter(_, ty)) in func_env.get_parameters().iter().enumerate() {
                if ty.is_mutual_reference() {
                    // Add reference parameter because we also want to debug track them when
                    // references are written.
                    res.push(self.track_local(
                        func_env,
                        loc.clone(),
                        idx,
                        &format!(
                                "{}",
                                func_env
                                    .get_local_name(idx)
                                    .display(self.module_env.symbol_pool())
                            ),
                    ));
                }
            }
            res.join("\n")
        };

        // Helper to save a borrowed value before mutation starts.
        let save_borrowed_value = |dest: &usize| {
            if let Some(idx) = local_to_before_borrow_idx.get(dest) {
                // Save the value before mutation happens.
                let name = boogie_var_before_borrow(*idx);
                emitln!(self.writer, "{} := $Dereference($m, $t{});", name, dest);
                emitln!(self.writer, "{}_ref := $t{};", name, dest);
            }
        };

        let propagate_abort = &format!(
            "if ($abort_flag) {{\n  assume $DebugTrackAbort({}, {});\n  goto Label_Abort;\n}}",
            func_env.module_env.env.file_id_to_idx(loc.file_id()),
            loc.span().start(),
        );
        match bytecode {
            Branch(target) => emitln!(self.writer, "goto Label_{};", target),
            BrTrue(target, idx) => emitln!(
                self.writer,
                "$tmp := $GetLocal($m, $frame + {});\nif (b#Boolean($tmp)) {{ goto Label_{}; }}",
                idx,
                target,
            ),
            BrFalse(target, idx) => emitln!(
                self.writer,
                "$tmp := $GetLocal($m, $frame + {});\nif (!b#Boolean($tmp)) {{ goto Label_{}; }}",
                idx,
                target,
            ),
            MoveLoc(dest, src) => {
                if self.get_local_type(func_env, *dest).is_reference() {
                    emitln!(
                        self.writer,
                        "call $t{} := CopyOrMoveRef({});",
                        dest,
                        func_env
                            .get_local_name(*src as usize)
                            .display(func_env.symbol_pool())
                    );
                } else {
                    emitln!(
                        self.writer,
                        "call $tmp := CopyOrMoveValue($GetLocal($m, $frame + {}));",
                        src
                    );
                    emitln!(self.writer, &update_and_track_local(*dest, "$tmp"));
                }
            }
            CopyLoc(dest, src) => {
                if self.get_local_type(func_env, *dest).is_reference() {
                    emitln!(
                        self.writer,
                        "call $t{} := CopyOrMoveRef({});",
                        dest,
                        func_env
                            .get_local_name(*src as usize)
                            .display(func_env.symbol_pool())
                    )
                } else {
                    emitln!(
                        self.writer,
                        "call $tmp := CopyOrMoveValue($GetLocal($m, $frame + {}));",
                        src
                    );
                    emitln!(self.writer, &update_and_track_local(*dest, "$tmp"));
                }
            }
            StLoc(dest, src) => {
                if self.get_local_type(func_env, *dest as usize).is_reference() {
                    let name = func_env.get_local_name(*dest as usize);
                    emitln!(
                        self.writer,
                        "call {} := CopyOrMoveRef($t{});",
                        name.display(func_env.symbol_pool()),
                        src
                    );
                    emitln!(
                        self.writer,
                        &track_local(
                            *dest as usize,
                            &format!("{}", name.display(func_env.symbol_pool()))
                        )
                    );
                } else {
                    emitln!(
                        self.writer,
                        "call $tmp := CopyOrMoveValue($GetLocal($m, $frame + {}));",
                        src
                    );
                    emitln!(self.writer, &update_and_track_local(*dest as usize, "$tmp"));
                }
            }
            BorrowLoc(dest, src) => {
                emitln!(
                    self.writer,
                    "call $t{} := BorrowLoc($frame + {});",
                    dest,
                    src,
                );
                save_borrowed_value(dest);
            }
            ReadRef(dest, src) => {
                emitln!(self.writer, "call $tmp := ReadRef($t{});", src);
                emit!(
                    self.writer,
                    &boogie_type_check(
                        self.module_env.env,
                        "$tmp",
                        &self.get_local_type(func_env, *dest)
                    )
                );
                emitln!(self.writer, &update_and_track_local(*dest, "$tmp"));
            }
            WriteRef(dest, src) => {
                emitln!(
                    self.writer,
                    "call WriteRef($t{}, $GetLocal($m, $frame + {}));\n{}",
                    dest,
                    src,
                    track_mutual_refs()
                );
            }
            FreezeRef(dest, src) => {
                emitln!(self.writer, "call $t{} := FreezeRef($t{});", dest, src)
            }
            Call(dests, callee_index, type_actuals, args) => {
                let callee_env = self.module_env.get_called_function(*callee_index);
                let mut dest_str = String::new();
                let mut args_str = String::new();
                let mut dest_type_assumptions = vec![];
                let mut tmp_assignments = vec![];

                args_str.push_str(&boogie_type_values(
                    func_env.module_env.env,
                    &func_env.module_env.get_type_actuals(*type_actuals),
                ));
                if !args_str.is_empty() && !args.is_empty() {
                    args_str.push_str(", ");
                }
                args_str.push_str(
                    &args
                        .iter()
                        .map(|arg_idx| {
                            if self.get_local_type(func_env, *arg_idx).is_reference() {
                                format!("$t{}", arg_idx)
                            } else {
                                format!("$GetLocal($m, $frame + {})", arg_idx)
                            }
                        })
                        .join(", "),
                );
                dest_str.push_str(
                    &dests
                        .iter()
                        .map(|dest_idx| {
                            let dest = format!("$t{}", dest_idx);
                            let dest_type = &self.get_local_type(func_env, *dest_idx);
                            dest_type_assumptions.push(boogie_type_check(
                                self.module_env.env,
                                &dest,
                                dest_type,
                            ));
                            if !dest_type.is_reference() {
                                tmp_assignments.push(update_and_track_local(*dest_idx, &dest));
                            } else {
                                tmp_assignments.push(track_local(*dest_idx, &dest));
                            }
                            dest
                        })
                        .join(", "),
                );
                if dest_str == "" {
                    emitln!(
                        self.writer,
                        "call {}({});",
                        boogie_function_name(&callee_env),
                        args_str
                    );
                } else {
                    emitln!(
                        self.writer,
                        "call {} := {}({});",
                        dest_str,
                        boogie_function_name(&callee_env),
                        args_str
                    );
                }
                emitln!(self.writer, propagate_abort);
                for s in &dest_type_assumptions {
                    emitln!(self.writer, s);
                }
                for s in &tmp_assignments {
                    emitln!(self.writer, s);
                }
                if callee_env.is_mutating() {
                    let s = track_mutual_refs();
                    if !s.is_empty() {
                        emitln!(self.writer, &s);
                    }
                }
            }
            Pack(dest, struct_def_index, type_actuals, fields) => {
                let struct_env = func_env.module_env.get_struct_by_def_idx(*struct_def_index);
                let effective_dest = effective_dest_func(*dest);
                let track_args = if effective_dest < func_env.get_local_count() {
                    format!(
                        "{}, {}, {}",
                        func_env.module_env.env.file_id_to_idx(loc.file_id()),
                        loc.span().start(),
                        effective_dest_func(*dest),
                    )
                } else {
                    "0, 0, 0".to_string()
                };
                let args_str = func_env
                    .module_env
                    .get_type_actuals(*type_actuals)
                    .iter()
                    .map(|s| boogie_type_value(self.module_env.env, s))
                    .chain(
                        fields
                            .iter()
                            .map(|i| format!("$GetLocal($m, $frame + {})", i)),
                    )
                    .join(", ");
                emitln!(
                    self.writer,
                    "call $tmp := {}_pack({}, {});",
                    boogie_struct_name(&struct_env),
                    track_args,
                    args_str
                );
                emitln!(self.writer, &update_and_track_local(*dest, "$tmp"));
            }
            Unpack(dests, struct_def_index, _, src) => {
                let struct_env = &func_env.module_env.get_struct_by_def_idx(*struct_def_index);
                let mut dests_str = String::new();
                let mut tmp_assignments = vec![];
                for dest in dests.iter() {
                    if !dests_str.is_empty() {
                        dests_str.push_str(", ");
                    }
                    let dest_str = &format!("$t{}", dest);
                    let dest_type = &self.get_local_type(func_env, *dest);
                    dests_str.push_str(dest_str);
                    if !dest_type.is_reference() {
                        tmp_assignments.push(update_and_track_local(*dest, &dest_str));
                    } else {
                        tmp_assignments.push(track_local(*dest, &dest_str));
                    }
                }
                emitln!(
                    self.writer,
                    "call {} := {}_unpack($GetLocal($m, $frame + {}));",
                    dests_str,
                    boogie_struct_name(struct_env),
                    src,
                );
                for s in &tmp_assignments {
                    emitln!(self.writer, s);
                }
            }
            BorrowField(dest, src, field_def_index) => {
                let struct_env = self.module_env.get_struct_of_field(field_def_index);
                let field_env = &struct_env.get_field_by_def_idx(*field_def_index);
                emitln!(
                    self.writer,
                    "call $t{} := BorrowField($t{}, {});",
                    dest,
                    src,
                    boogie_field_name(field_env)
                );
                save_borrowed_value(dest);
            }
            Exists(dest, addr, struct_def_index, type_actuals) => {
                let resource_type = boogie_struct_type_value(
                    self.module_env.env,
                    self.module_env.get_id(),
                    self.module_env.get_struct_id(*struct_def_index),
                    &self.module_env.get_type_actuals(*type_actuals),
                );
                emitln!(
                    self.writer,
                    "call $tmp := Exists($GetLocal($m, $frame + {}), {});",
                    addr,
                    resource_type
                );
                emitln!(self.writer, &update_and_track_local(*dest, "$tmp"));
            }
            BorrowGlobal(dest, addr, struct_def_index, type_actuals) => {
                let resource_type = boogie_struct_type_value(
                    self.module_env.env,
                    self.module_env.get_id(),
                    self.module_env.get_struct_id(*struct_def_index),
                    &self.module_env.get_type_actuals(*type_actuals),
                );
                emitln!(
                    self.writer,
                    "call $t{} := BorrowGlobal($GetLocal($m, $frame + {}), {});",
                    dest,
                    addr,
                    resource_type,
                );
                emitln!(self.writer, propagate_abort);
                save_borrowed_value(dest);
            }
            MoveToSender(src, struct_def_index, type_actuals) => {
                let resource_type = boogie_struct_type_value(
                    self.module_env.env,
                    self.module_env.get_id(),
                    self.module_env.get_struct_id(*struct_def_index),
                    &self.module_env.get_type_actuals(*type_actuals),
                );
                emitln!(
                    self.writer,
                    "call MoveToSender({}, $GetLocal($m, $frame + {}));",
                    resource_type,
                    src,
                );
                emitln!(self.writer, propagate_abort);
            }
            MoveFrom(dest, src, struct_def_index, type_actuals) => {
                let resource_type = boogie_struct_type_value(
                    self.module_env.env,
                    self.module_env.get_id(),
                    self.module_env.get_struct_id(*struct_def_index),
                    &self.module_env.get_type_actuals(*type_actuals),
                );
                emitln!(
                    self.writer,
                    "call $tmp := MoveFrom($GetLocal($m, $frame + {}), {});",
                    src,
                    resource_type,
                );
                emitln!(self.writer, &update_and_track_local(*dest, "$tmp"));
                emit!(
                    self.writer,
                    &boogie_type_check(
                        self.module_env.env,
                        &format!("$t{}", dest),
                        &self.get_local_type(func_env, *dest)
                    )
                );
                emitln!(self.writer, propagate_abort);
            }
            Ret(rets) => {
                // Enforce invariants on mutable references not returned.
                let mut before_borrows_to_check = BTreeMap::new();
                for (local_idx, before_borrow_idx) in local_to_before_borrow_idx {
                    if !before_borrows_to_check.contains_key(before_borrow_idx) {
                        let ty = self.get_local_type(func_env, *local_idx);
                        before_borrows_to_check.insert(before_borrow_idx, ty);
                    }
                }
                for ret in rets {
                    if let Some(before_borrow_idx) = local_to_before_borrow_idx.get(ret) {
                        before_borrows_to_check.remove(&before_borrow_idx);
                    }
                }

                for (before_borrow_idx, ty) in before_borrows_to_check {
                    if !released_before_borrows.contains(before_borrow_idx) {
                        self.enforce_borrowed_invariant(func_env, &ty, *before_borrow_idx);
                    }
                }
                for (i, r) in rets.iter().enumerate() {
                    if self.get_local_type(func_env, *r).is_reference() {
                        emitln!(self.writer, "$ret{} := $t{};", i, r);
                    } else {
                        emitln!(self.writer, "$ret{} := $GetLocal($m, $frame + {});", i, r);
                    }
                    emitln!(self.writer, &track_return(i));
                }
                emitln!(self.writer, "return;");
            }
            LdTrue(idx) => {
                emitln!(self.writer, "call $tmp := LdTrue();");
                emitln!(self.writer, &update_and_track_local(*idx, "$tmp"));
            }
            LdFalse(idx) => {
                emitln!(self.writer, "call $tmp := LdFalse();");
                emitln!(self.writer, &update_and_track_local(*idx, "$tmp"));
            }
            LdU8(idx, num) => {
                emitln!(self.writer, "call $tmp := LdConst({});", num);
                emitln!(self.writer, &update_and_track_local(*idx, "$tmp"));
            }
            LdU64(idx, num) => {
                emitln!(self.writer, "call $tmp := LdConst({});", num);
                emitln!(self.writer, &update_and_track_local(*idx, "$tmp"));
            }
            LdU128(idx, num) => {
                emitln!(self.writer, "call $tmp := LdConst({});", num);
                emitln!(self.writer, &update_and_track_local(*idx, "$tmp"));
            }
            CastU8(dest, src) => {
                emitln!(
                    self.writer,
                    "call $tmp := CastU8($GetLocal($m, $frame + {}));",
                    src
                );
                emitln!(self.writer, propagate_abort);
                emitln!(self.writer, &update_and_track_local(*dest, "$tmp"));
            }
            CastU64(dest, src) => {
                emitln!(
                    self.writer,
                    "call $tmp := CastU64($GetLocal($m, $frame + {}));",
                    src
                );
                emitln!(self.writer, propagate_abort);
                emitln!(self.writer, &update_and_track_local(*dest, "$tmp"));
            }
            CastU128(dest, src) => {
                emitln!(
                    self.writer,
                    "call $tmp := CastU128($GetLocal($m, $frame + {}));",
                    src
                );
                emitln!(self.writer, propagate_abort);
                emitln!(self.writer, &update_and_track_local(*dest, "$tmp"));
            }
            LdAddr(idx, addr_idx) => {
                let addr_int = self.module_env.get_address(*addr_idx);
                emitln!(self.writer, "call $tmp := LdAddr({});", addr_int);
                emitln!(self.writer, &update_and_track_local(*idx, "$tmp"));
            }
            Not(dest, operand) => {
                emitln!(
                    self.writer,
                    "call $tmp := Not($GetLocal($m, $frame + {}));",
                    operand
                );
                emitln!(self.writer, &update_and_track_local(*dest, "$tmp"));
            }
            Add(dest, op1, op2) => {
                let add_type = match self.get_local_type(func_env, *dest) {
                    Type::Primitive(PrimitiveType::U8) => "U8",
                    Type::Primitive(PrimitiveType::U64) => "U64",
                    Type::Primitive(PrimitiveType::U128) => "U128",
                    _ => unreachable!(),
                };
                emitln!(
                    self.writer,
                    "call $tmp := Add{}($GetLocal($m, $frame + {}), $GetLocal($m, $frame + {}));",
                    add_type,
                    op1,
                    op2
                );
                emitln!(self.writer, propagate_abort);
                emitln!(self.writer, &update_and_track_local(*dest, "$tmp"));
            }
            Sub(dest, op1, op2) => {
                emitln!(
                    self.writer,
                    "call $tmp := Sub($GetLocal($m, $frame + {}), $GetLocal($m, $frame + {}));",
                    op1,
                    op2
                );
                emitln!(self.writer, propagate_abort);
                emitln!(self.writer, &update_and_track_local(*dest, "$tmp"));
            }
            Mul(dest, op1, op2) => {
                let mul_type = match self.get_local_type(func_env, *dest) {
                    Type::Primitive(PrimitiveType::U8) => "U8",
                    Type::Primitive(PrimitiveType::U64) => "U64",
                    Type::Primitive(PrimitiveType::U128) => "U128",
                    _ => unreachable!(),
                };
                emitln!(
                    self.writer,
                    "call $tmp := Mul{}($GetLocal($m, $frame + {}), $GetLocal($m, $frame + {}));",
                    mul_type,
                    op1,
                    op2
                );
                emitln!(self.writer, propagate_abort);
                emitln!(self.writer, &update_and_track_local(*dest, "$tmp"));
            }
            Div(dest, op1, op2) => {
                emitln!(
                    self.writer,
                    "call $tmp := Div($GetLocal($m, $frame + {}), $GetLocal($m, $frame + {}));",
                    op1,
                    op2
                );
                emitln!(self.writer, propagate_abort);
                emitln!(self.writer, &update_and_track_local(*dest, "$tmp"));
            }
            Mod(dest, op1, op2) => {
                emitln!(
                    self.writer,
                    "call $tmp := Mod($GetLocal($m, $frame + {}), $GetLocal($m, $frame + {}));",
                    op1,
                    op2
                );
                emitln!(self.writer, propagate_abort);
                emitln!(self.writer, &update_and_track_local(*dest, "$tmp"));
            }
            Lt(dest, op1, op2) => {
                emitln!(
                    self.writer,
                    "call $tmp := Lt($GetLocal($m, $frame + {}), $GetLocal($m, $frame + {}));",
                    op1,
                    op2
                );
                emitln!(self.writer, &update_and_track_local(*dest, "$tmp"));
            }
            Gt(dest, op1, op2) => {
                emitln!(
                    self.writer,
                    "call $tmp := Gt($GetLocal($m, $frame + {}), $GetLocal($m, $frame + {}));",
                    op1,
                    op2
                );
                emitln!(self.writer, &update_and_track_local(*dest, "$tmp"));
            }
            Le(dest, op1, op2) => {
                emitln!(
                    self.writer,
                    "call $tmp := Le($GetLocal($m, $frame + {}), $GetLocal($m, $frame + {}));",
                    op1,
                    op2
                );
                emitln!(self.writer, &update_and_track_local(*dest, "$tmp"));
            }
            Ge(dest, op1, op2) => {
                emitln!(
                    self.writer,
                    "call $tmp := Ge($GetLocal($m, $frame + {}), $GetLocal($m, $frame + {}));",
                    op1,
                    op2
                );
                emitln!(self.writer, &update_and_track_local(*dest, "$tmp"));
            }
            Or(dest, op1, op2) => {
                emitln!(
                    self.writer,
                    "call $tmp := Or($GetLocal($m, $frame + {}), $GetLocal($m, $frame + {}));",
                    op1,
                    op2
                );
                emitln!(self.writer, &update_and_track_local(*dest, "$tmp"));
            }
            And(dest, op1, op2) => {
                emitln!(
                    self.writer,
                    "call $tmp := And($GetLocal($m, $frame + {}), $GetLocal($m, $frame + {}));",
                    op1,
                    op2
                );
                emitln!(self.writer, &update_and_track_local(*dest, "$tmp"));
            }
            Eq(dest, op1, op2) => {
                emitln!(
                    self.writer,
                    "$tmp := Boolean(IsEqual($GetLocal($m, $frame + {}), $GetLocal($m, $frame + {})));",
                    op1,
                    op2
                );
                emitln!(self.writer, &update_and_track_local(*dest, "$tmp"));
            }
            Neq(dest, op1, op2) => {
                emitln!(
                    self.writer,
                    "$tmp := Boolean(!IsEqual($GetLocal($m, $frame + {}), $GetLocal($m, $frame + {})));",
                    op1,
                    op2
                );
                emitln!(self.writer, &update_and_track_local(*dest, "$tmp"));
            }
            BitOr(_, _, _) | BitAnd(_, _, _) | Xor(_, _, _) => {
                emitln!(
                    self.writer,
                    "// bit operation not supported: {:?}",
                    bytecode
                );
            }
            Abort(_) => {
                // Below we introduce a dummy `if` for $DebugTrackAbort to ensure boogie creates
                // a execution trace entry for this statement.
                emitln!(
                    self.writer,
                    "if (true) {{ assume $DebugTrackAbort({}, {}); }}",
                    func_env.module_env.env.file_id_to_idx(loc.file_id()),
                    loc.span().start(),
                );
                emitln!(self.writer, "goto Label_Abort;")
            }
            GetGasRemaining(idx) => {
                emitln!(self.writer, "call $tmp := GetGasRemaining();");
                emitln!(self.writer, &update_and_track_local(*idx, "$tmp"));
            }
            GetTxnSequenceNumber(idx) => {
                emitln!(self.writer, "call $tmp := GetTxnSequenceNumber();");
                emitln!(self.writer, &update_and_track_local(*idx, "$tmp"));
            }
            GetTxnPublicKey(idx) => {
                emitln!(self.writer, "call $tmp := GetTxnPublicKey();");
                emitln!(self.writer, &update_and_track_local(*idx, "$tmp"));
            }
            GetTxnSenderAddress(idx) => {
                emitln!(self.writer, "call $tmp := GetTxnSenderAddress();");
                emitln!(self.writer, &update_and_track_local(*idx, "$tmp"));
            }
            GetTxnMaxGasUnits(idx) => {
                emitln!(self.writer, "call $tmp := GetTxnMaxGasUnits();");
                emitln!(self.writer, &update_and_track_local(*idx, "$tmp"));
            }
            GetTxnGasUnitPrice(idx) => {
                emitln!(self.writer, "call $tmp := GetTxnGasUnitPrice();");
                emitln!(self.writer, &update_and_track_local(*idx, "$tmp"));
            }
            _ => emitln!(self.writer, "// unimplemented instruction: {:?}", bytecode),
        }
        emitln!(self.writer);
    }

    /// Return a string for a boogie procedure header.
    /// if inline = true, add the inline attribute and use the plain function name
    /// for the procedure name. Also inject pre/post conditions if defined.
    /// Else, generate the function signature without the ":inline" attribute, and
    /// append _verify to the function name.
    fn generate_function_sig(&self, func_env: &FunctionEnv<'_>, inline: bool) {
        let (args, rets) = self.generate_function_args_and_returns(func_env);
        if inline {
            emit!(
                self.writer,
                "procedure {{:inline 1}} {} ({}) returns ({})",
                boogie_function_name(func_env),
                args,
                rets,
            )
        } else {
            emit!(
                self.writer,
                "procedure {}_verify ({}) returns ({})",
                boogie_function_name(func_env),
                args,
                rets
            )
        }
    }

    /// Generate boogie representation of function args and return args.
    fn generate_function_args_and_returns(&self, func_env: &FunctionEnv<'_>) -> (String, String) {
        let args = func_env
            .get_type_parameters()
            .iter()
            .map(|TypeParameter(s, _)| format!("{}: TypeValue", s.display(func_env.symbol_pool())))
            .chain(func_env.get_parameters().iter().map(|Parameter(s, ty)| {
                format!(
                    "{}: {}",
                    s.display(func_env.symbol_pool()),
                    boogie_local_type(ty)
                )
            }))
            .join(", ");
        let rets = func_env
            .get_return_types()
            .iter()
            .enumerate()
            .map(|(i, ref s)| format!("$ret{}: {}", i, boogie_local_type(s)))
            .join(", ");
        (args, rets)
    }

    /// Return string for the function specification.
    fn generate_function_spec(&self, func_env: &FunctionEnv<'_>) {
        emitln!(self.writer);
        SpecTranslator::new(self.writer, &func_env.module_env, true).translate_conditions(func_env);
    }

    /// Return string for body of verify function, which is just a call to the
    /// inline version of the function.
    fn generate_verify_function_body(&self, func_env: &FunctionEnv<'_>) {
        // Set the location to internal so it won't be counted for execution traces
        self.writer
            .set_location(&self.module_env.env.internal_loc());

        let args = func_env
            .get_type_parameters()
            .iter()
            .map(|TypeParameter(s, _)| format!("{}", s.display(func_env.symbol_pool())))
            .chain(
                func_env
                    .get_parameters()
                    .iter()
                    .map(|Parameter(s, _)| format!("{}", s.display(func_env.symbol_pool()))),
            )
            .join(", ");
        let rets = (0..func_env.get_return_count())
            .map(|i| format!("$ret{}", i))
            .join(", ");
        let assumptions = "    call $InitVerification();\n";
        if rets.is_empty() {
            emit!(
                self.writer,
                "\n{{\n{}    call {}({});\n}}\n\n",
                assumptions,
                boogie_function_name(func_env),
                args
            )
        } else {
            emit!(
                self.writer,
                "\n{{\n{}    call {} := {}({});\n}}\n\n",
                assumptions,
                rets,
                boogie_function_name(func_env),
                args
            )
        }
    }

    /// This generates boogie code for everything after the function signature
    /// The function body is only generated for the "inline" version of the function.
    fn generate_inline_function_body(&self, func_env: &FunctionEnv<'_>) {
        let code = &self.stackless_bytecode[func_env.get_def_idx().0 as usize];

        // Identify all the branching targets so we can insert labels in front of them. Also
        // calculate mutual borrows.
        let mut branching_targets = BTreeSet::new();
        let mut mutual_local_borrows = BTreeSet::new();

        // Map from local indices to indices of before_borrow variables.
        let mut local_to_before_borrow_idx = BTreeMap::new();
        let mut released_before_borrows = BTreeSet::new();
        let mut num_mut_refs = 0;
        let cfg = StacklessControlFlowGraph::new(&code.code);

        // Set of dead refs at each code offset.
        let offset_to_dead_refs = LifetimeAnalysis::analyze(&cfg, &code.code, &code.local_types);

        for bytecode in code.code.iter() {
            match bytecode {
                Branch(target) | BrTrue(target, _) | BrFalse(target, _) => {
                    branching_targets.insert(*target as usize);
                }
                BorrowLoc(dst, src) => {
                    let ty = self.get_local_type(func_env, *dst);
                    if ty.is_mutual_reference() {
                        mutual_local_borrows.insert(*src as usize);
                        if self.has_update_invariant_on_release(&ty) {
                            local_to_before_borrow_idx.insert(*dst, num_mut_refs);
                            num_mut_refs += 1;
                        }
                    }
                }
                BorrowGlobal(dst, ..) | BorrowField(dst, ..) => {
                    let ty = self.get_local_type(func_env, *dst);
                    if ty.is_mutual_reference() && self.has_update_invariant_on_release(&ty) {
                        local_to_before_borrow_idx.insert(*dst, num_mut_refs);
                        num_mut_refs += 1;
                    }
                }
                MoveLoc(dst, src) => {
                    // If src has update invariants to check for then insert the dst to
                    // the map and point to the same before_borrow index so that we can
                    // later use dst to find the before_borrow index
                    let src_idx = *src as usize;
                    if local_to_before_borrow_idx.contains_key(&src_idx) {
                        let idx = local_to_before_borrow_idx[&src_idx];
                        local_to_before_borrow_idx.insert(*dst, idx);
                    }
                }
                StLoc(dst, src) => {
                    if local_to_before_borrow_idx.contains_key(src) {
                        let idx = local_to_before_borrow_idx[src];
                        local_to_before_borrow_idx.insert(*dst as usize, idx);
                    }
                }
                _ => {}
            }
        }

        // Be sure to set back location to the whole function definition as a default, otherwise
        // we may get unassigned code locations associated with condition locations.
        self.writer.set_location(&func_env.get_loc());

        emitln!(self.writer, "{");
        self.writer.indent();

        // Generate local variable declarations. They need to appear first in boogie.
        emitln!(self.writer, "// declare local variables");
        let num_args = func_env.get_parameters().len();
        for i in num_args..code.local_types.len() {
            let local_name = func_env.get_local_name(i);
            let local_type = &self.module_env.globalize_signature(&code.local_types[i]);
            emitln!(
                self.writer,
                "var {}: {}; // {}",
                local_name.display(func_env.symbol_pool()),
                boogie_local_type(local_type),
                boogie_type_value(self.module_env.env, local_type)
            );
        }
        emitln!(self.writer, "var $tmp: Value;");
        emitln!(self.writer, "var $frame: int;");
        emitln!(self.writer, "var $saved_m: Memory;");
        for idx in 0..num_mut_refs {
            let name = boogie_var_before_borrow(idx);
            emitln!(self.writer, "var {}: Value;", name);
            emitln!(self.writer, "var {}_ref: Reference;", name);
        }

        emitln!(self.writer, "\n// initialize function execution");
        emitln!(self.writer, "assume !$abort_flag;");
        emitln!(self.writer, "$saved_m := $m;");
        emitln!(self.writer, "$frame := $local_counter;");

        emitln!(self.writer, "\n// process and type check arguments");
        for i in 0..num_args {
            let local_name = func_env.get_local_name(i);
            let local_str = format!("{}", local_name.display(func_env.symbol_pool()));
            let local_type = &self.module_env.globalize_signature(&code.local_types[i]);
            let type_check = boogie_type_check(self.module_env.env, &local_str, local_type);
            emit!(self.writer, &type_check);
            if !local_type.is_reference() {
                emitln!(
                    self.writer,
                    &self.update_and_track_local(func_env, func_env.get_loc(), i, &local_str)
                );
            } else {
                emitln!(
                    self.writer,
                    &self.track_local(func_env, func_env.get_loc(), i, &local_str)
                );
            }
        }

        emitln!(self.writer, "\n// increase the local counter ");
        emitln!(
            self.writer,
            "$local_counter := $local_counter + {};",
            code.local_types.len()
        );

        emitln!(self.writer, "\n// bytecode translation starts here");

        // Generate bytecode
        for (offset, bytecode) in code.code.iter().enumerate() {
            // insert labels for branching targets
            if branching_targets.contains(&offset) {
                self.writer.unindent();
                emitln!(self.writer, "Label_{}:", offset);
                self.writer.indent();
            }
            // Compute effective destination to enhance debug experience. This looks ahead
            // at the next instructions to detect a simple aliasing via copy or move and use that
            // destination as it might be a user variable whereas this instruction has a temporary
            // destination introduced by stackless bytecode transformation.
            // TODO: the stackless bytecode should optimize away unnecessary copy/moves, so we
            // don't need this. The below transformation is only correct for stackless code
            // of certain shape
            let effective_dest_func = |dest: usize| -> usize {
                if dest >= func_env.get_local_count() && offset + 1 < code.code.len() {
                    if let Pack(temp_dest, ..) = &code.code[offset] {
                        match &code.code[offset + 1] {
                            MoveLoc(effective_dest, src) | CopyLoc(effective_dest, src) => {
                                if *src as usize == *temp_dest {
                                    return *effective_dest;
                                }
                            }
                            StLoc(effective_dest, src) => {
                                if *src == *temp_dest {
                                    return *effective_dest as usize;
                                }
                            }
                            _ => {}
                        }
                    }
                }
                dest
            };
            self.translate_bytecode(
                func_env,
                offset as u16,
                bytecode,
                effective_dest_func,
                &mutual_local_borrows,
                &local_to_before_borrow_idx,
                &released_before_borrows,
            );

            // Enforce invariants on references going out of scope.
            if let Some(dead_refs) = offset_to_dead_refs.get(&(offset as u16)) {
                for ref_idx in dead_refs {
                    if let Some(idx) = local_to_before_borrow_idx.get(&(*ref_idx as usize)) {
                        let ty = self.get_local_type(func_env, *ref_idx as usize);
                        self.enforce_borrowed_invariant(func_env, &ty, *idx);
                        released_before_borrows.insert(*idx);
                    }
                }
            }
        }

        // Generate abort exit.
        let end_loc = func_env.get_loc().at_end();
        self.writer.set_location(&end_loc);
        self.writer.unindent();
        emitln!(self.writer, "Label_Abort:");
        self.writer.indent();
        emitln!(self.writer, "$abort_flag := true;");
        emitln!(self.writer, "$m := $saved_m;");
        for (i, ty) in func_env.get_return_types().iter().enumerate() {
            let ret_str = format!("$ret{}", i);
            if ty.is_reference() {
                emitln!(self.writer, "{} := DefaultReference;", &ret_str);
            } else {
                emitln!(self.writer, "{} := DefaultValue;", &ret_str);
            }
        }
        self.writer.unindent();
        emitln!(self.writer, "}");
    }

    /// Looks up the type of a local in the stackless bytecode representation.
    fn get_local_type(&self, func_env: &FunctionEnv<'_>, local_idx: usize) -> Type {
        self.module_env.globalize_signature(
            &self.stackless_bytecode[func_env.get_def_idx().0 as usize].local_types[local_idx],
        )
    }

    /// Determines whether this type has update invariants that shall be
    /// enforced on mutual reference release.
    fn has_update_invariant_on_release(&'env self, ty: &Type) -> bool {
        if let Type::Reference(true, bt) = &ty {
            if let Type::Struct(module_idx, struct_idx, _) = bt.as_ref() {
                let struct_env = self
                    .module_env
                    .env
                    .get_module(*module_idx)
                    .into_struct(*struct_idx);
                if !struct_env.get_data_invariants().is_empty()
                    || !struct_env.get_update_invariants().is_empty()
                {
                    return true;
                }
            }
        }
        false
    }

    // Enforce the invariant of a borrowed value after mutation ended.
    fn enforce_borrowed_invariant(&self, func_env: &FunctionEnv<'_>, ty: &Type, idx: usize) {
        let var_name = boogie_var_before_borrow(idx);
        if let Type::Reference(true, bt) = ty {
            if let Type::Struct(module_idx, struct_idx, _) = bt.as_ref() {
                let struct_env = func_env
                    .module_env
                    .env
                    .get_module(*module_idx)
                    .into_struct(*struct_idx);
                emitln!(
                    self.writer,
                    "call {}_update_inv({}, $Dereference($m, {}_ref));",
                    boogie_struct_name(&struct_env),
                    var_name,
                    var_name
                );
            }
        }
    }

    /// Updates a local, injecting debug information if available.
    fn update_and_track_local(
        &self,
        func_env: &FunctionEnv<'_>,
        loc: Loc,
        idx: usize,
        value: &str,
    ) -> String {
        let update = format!("$m := $UpdateLocal($m, $frame + {}, {});", idx, value);
        if idx >= func_env.get_local_count() {
            // skip debug tracking for temporaries
            return update;
        }
        let debug_update = self.track_local(func_env, loc, idx, value);
        if !debug_update.is_empty() {
            format!("{}\n{}", update, debug_update)
        } else {
            update
        }
    }

    /// Generates an update of the model debug variable at given location.
    fn track_local(&self, func_env: &FunctionEnv<'_>, loc: Loc, idx: usize, value: &str) -> String {
        if self.options.omit_model_debug {
            return "".to_string();
        }
        let ty = if idx < func_env.get_local_count() {
            func_env.get_local_type(idx)
        } else {
            func_env.get_return_types()[idx - func_env.get_local_count()].clone()
        };
        if let Type::Reference(_, bt) = ty {
            let deref = format!("$Dereference($m, {})", value);
            let type_check = boogie_type_check(self.module_env.env, &deref, &*bt);
            // Below we introduce a dummy `if` for $DebugTrackAbort to ensure boogie creates
            // a execution trace entry for this statement.
            format!(
                "{}if (true) {{ assume $DebugTrackLocal({}, {}, {}, {}); }}",
                type_check,
                func_env.module_env.env.file_id_to_idx(loc.file_id()),
                loc.span().start(),
                idx,
                deref
            )
        } else {
            format!(
                "if (true) {{ assume $DebugTrackLocal({}, {}, {}, {}); }}",
                func_env.module_env.env.file_id_to_idx(loc.file_id()),
                loc.span().start(),
                idx,
                value
            )
        }
    }
}

/// Separates elements in vector, dropping empty ones.
fn separate(elems: Vec<String>, sep: &str) -> String {
    elems.iter().filter(|s| !s.is_empty()).join(sep)
}
