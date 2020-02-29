// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module translates the bytecode of a module to Boogie code.

use std::collections::BTreeSet;

use codespan::ByteOffset;
use itertools::Itertools;
#[allow(unused_imports)]
use log::{debug, info};

use codespan::Span;
use libra_types::{account_address::AccountAddress, language_storage::ModuleId};
use move_ir_types::location::{Loc, Spanned};
use stackless_bytecode_generator::{
    stackless_bytecode::StacklessBytecode::{self, *},
    stackless_bytecode_generator::{StacklessFunction, StacklessModuleGenerator},
};

use crate::{
    boogie_helpers::{
        boogie_declare_global, boogie_field_name, boogie_function_name, boogie_invariant_target,
        boogie_local_type, boogie_struct_name, boogie_struct_type_value, boogie_synthetic_name,
        boogie_type_check, boogie_type_value, boogie_type_values, boogie_var_before_borrow,
    },
    cli::InvariantModel,
    code_writer::CodeWriter,
    driver::PSEUDO_PRELUDE_MODULE,
    env::{
        FunctionEnv, GlobalEnv, GlobalType, ModuleEnv, Parameter, StructEnv, TypeParameter,
        ERROR_TYPE,
    },
    spec_translator::SpecTranslator,
};

pub struct BoogieTranslator<'env> {
    env: &'env GlobalEnv,
    writer: &'env CodeWriter,
}

pub struct ModuleTranslator<'env> {
    writer: &'env CodeWriter,
    module_env: ModuleEnv<'env>,
    stackless_bytecode: Vec<StacklessFunction>,
}

/// Returns true if for the module no code should be produced because its already defined
/// in the prelude.
pub fn is_module_provided_by_prelude(id: &ModuleId) -> bool {
    id.name().as_str() == "Vector"
        && *id.address() == AccountAddress::from_hex_literal("0x0").unwrap()
}

impl<'env> BoogieTranslator<'env> {
    pub fn new(env: &'env GlobalEnv, writer: &'env CodeWriter) -> Self {
        Self { env, writer }
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
            StacklessModuleGenerator::new(module.get_verified_module().as_inner())
                .generate_module();
        Self {
            writer: parent.writer,
            module_env: module,
            stackless_bytecode,
        }
    }

    /// Translates this module.
    fn translate(&mut self) {
        if !is_module_provided_by_prelude(self.module_env.get_id()) {
            info!("translating module {}", self.module_env.get_id().name());
            self.writer.set_location(
                self.module_env.get_module_idx(),
                Spanned::unsafe_no_loc(()).loc,
            );
            self.translate_synthetics();
            self.translate_structs();
            self.translate_functions();
        }
    }

    fn translate_synthetics(&self) {
        emitln!(
            self.writer,
            "\n\n// ** synthetics of module {}\n",
            self.module_env.get_id().name()
        );
        let mut seen = BTreeSet::new();
        for (syn, ty) in self.module_env.get_synthetics() {
            if !seen.insert(syn.value.name.to_string()) {
                self.module_env.error(
                    syn.loc,
                    &format!("duplicate declaration of synthetic `{}`", syn.value.name),
                    (),
                );
                continue;
            }
            if ty == &ERROR_TYPE {
                continue;
            }
            if ty.is_reference() {
                self.module_env.error(
                    syn.loc,
                    &format!("synthetic `{}` cannot have reference type", syn.value.name),
                    (),
                );
                continue;
            }
            let boogie_name = boogie_synthetic_name(&self.module_env, syn.value.name.as_str());
            emitln!(
                self.writer,
                &boogie_declare_global(&self.module_env.env, &boogie_name, &ty)
            );
        }
    }

    /// Translates all structs in the module.
    fn translate_structs(&self) {
        emitln!(
            self.writer,
            "\n\n// ** structs of module {}\n",
            self.module_env.get_id().name()
        );
        for struct_env in self.module_env.get_structs() {
            // Set the location to the PSEUDO module so we don't see locations of pack/unpack
            // in execution traces.
            self.writer
                .set_location(PSEUDO_PRELUDE_MODULE, Spanned::unsafe_no_loc(()).loc);
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
            .map(|(i, _)| format!("tv{}: TypeValue", i))
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
        let spec_translator = SpecTranslator::new(
            self.writer,
            &struct_env.module_env,
            struct_env.get_type_parameters(),
            false,
        );
        spec_translator.translate_invariant_functions(&struct_env);
    }

    /// Translates struct accessors (pack/unpack).
    fn translate_struct_accessors(&self, struct_env: &StructEnv<'_>) {
        // Pack function
        let type_args_str = struct_env
            .get_type_parameters()
            .iter()
            .map(|TypeParameter(ref i, _)| format!("{}: TypeValue", i))
            .join(", ");
        let args_str = struct_env
            .get_fields()
            .map(|field_env| format!("{}: Value", field_env.get_name()))
            .join(", ");
        emitln!(
            self.writer,
            "procedure {{:inline 1}} Pack_{}(module_idx: int, func_idx: int, var_idx: int, code_idx: int, {}) returns (_struct: Value)\n{{",
            boogie_struct_name(struct_env),
            separate(vec![type_args_str, args_str.clone()], ", ")
        );
        self.writer.indent();
        let mut fields_str = String::from("EmptyValueArray");
        for field_env in struct_env.get_fields() {
            let type_check = boogie_type_check(
                self.module_env.env,
                field_env.get_name().as_str(),
                &field_env.get_type(),
            );
            emit!(self.writer, &type_check);
            fields_str = format!("ExtendValueArray({}, {})", fields_str, field_env.get_name());
        }
        emitln!(self.writer, "_struct := Vector({});", fields_str);

        // Generate $DebugTrackLocal so we can see the constructed value before invariant
        // evaluation may abort.
        emitln!(
            self.writer,
            "if (code_idx > 0) { assume $DebugTrackLocal(module_idx, func_idx, var_idx, code_idx, _struct); }"
        );

        // Insert invariant code.
        let spec_translator = SpecTranslator::new(
            self.writer,
            &struct_env.module_env,
            struct_env.get_type_parameters(),
            false,
        );
        spec_translator.emit_pack_invariants(struct_env, "_struct");

        self.writer.unindent();
        emitln!(self.writer, "}\n");

        // Unpack function
        emitln!(
            self.writer,
            "procedure {{:inline 1}} Unpack_{}(_struct: Value) returns ({})\n{{",
            boogie_struct_name(struct_env),
            args_str
        );
        self.writer.indent();
        emitln!(self.writer, "assume is#Vector(_struct);");
        for field_env in struct_env.get_fields() {
            emitln!(
                self.writer,
                "{} := SelectField(_struct, {});",
                field_env.get_name(),
                boogie_field_name(&field_env)
            );
            let type_check = boogie_type_check(
                self.module_env.env,
                field_env.get_name().as_str(),
                &field_env.get_type(),
            );
            emit!(self.writer, &type_check);
        }

        // Insert invariant checking code.
        let spec_translator = SpecTranslator::new(
            self.writer,
            &struct_env.module_env,
            struct_env.get_type_parameters(),
            false,
        );
        spec_translator.emit_unpack_invariants(struct_env, "_struct");

        self.writer.unindent();
        emitln!(self.writer, "}\n");
    }

    /// Translates all functions in the module.
    fn translate_functions(&self) {
        emitln!(
            self.writer,
            "\n\n// ** functions of module {}\n",
            self.module_env.get_id().name()
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
            self.writer
                .set_location(self.module_env.get_module_idx(), func_env.get_loc());
            self.translate_function(&func_env);
        }
        if num_fun > 0 {
            info!(
                "{} out of {} functions are specified in module {}",
                num_fun_specified,
                num_fun,
                self.module_env.get_id().name()
            );
        }
    }

    /// Translates the given function.
    fn translate_function(&self, func_env: &FunctionEnv<'_>) {
        if func_env.is_native() {
            if self.module_env.env.options.native_stubs {
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
        update_invariant_on_release: &[GlobalType],
    ) {
        // Set location of this code in the CodeWriter.
        let loc = func_env.get_bytecode_loc(offset);
        self.writer
            .set_location(self.module_env.get_module_idx(), loc);
        // emitln!(self.writer, "// {}", loc); // DEBUG

        // Helper functions to update a local including debug information.
        let update_and_track_local =
            |idx: usize, value: &str| self.update_and_track_local(func_env, loc, idx, value);
        let track_local = |idx: usize, value: &str| {
            if idx < func_env.get_local_count() {
                self.track_local(func_env, loc, idx, value)
            } else {
                "".to_string()
            }
        };
        let track_return = |idx: usize| {
            self.track_local(
                func_env,
                loc,
                func_env.get_local_count() + idx,
                &format!("__ret{}", idx),
            )
        };

        // Helper function to update debug info for potential updates of references.
        let track_mutual_refs = || {
            let mut res = vec![];
            for idx in mutual_local_borrows {
                res.push(self.track_local(
                    func_env,
                    loc,
                    *idx,
                    &format!("GetLocal(__m, __frame + {})", idx),
                ));
            }
            for (idx, Parameter(_, ty)) in func_env.get_parameters().iter().enumerate() {
                if ty.is_mutual_reference() {
                    // Add reference parameter because we also want to debug track them when
                    // references are written.
                    res.push(self.track_local(func_env, loc, idx, &func_env.get_local_name(idx)));
                }
            }
            res.join("\n")
        };

        // Helper to save a borrowed value before mutation starts.
        let save_borrowed_value = |dest: &usize| {
            let dest_ty = self.get_local_type(func_env, *dest);
            if let Some(idx) = update_invariant_on_release
                .iter()
                .position(|ty| ty == &dest_ty)
            {
                // Save the value before mutation happens.
                let name = boogie_var_before_borrow(idx);
                emitln!(self.writer, "{} := Dereference(__m, __t{});", name, dest);
                emitln!(self.writer, "{}_ref := __t{};", name, dest);
            }
        };

        // Helper to enforce the invariant of a borrowed value after mutation ended.
        let enforce_borrowed_invariant = |ty: &GlobalType, idx: usize| {
            let var_name = boogie_var_before_borrow(idx);
            if let GlobalType::MutableReference(bt) = ty {
                if let GlobalType::Struct(module_idx, struct_idx, _) = bt.as_ref() {
                    let struct_env = func_env
                        .module_env
                        .env
                        .get_module(*module_idx)
                        .into_get_struct(*struct_idx);
                    emitln!(
                        self.writer,
                        "call ${}_update_inv({}, Dereference(__m, {}_ref));",
                        boogie_struct_name(&struct_env),
                        var_name,
                        var_name
                    );
                }
            }
        };

        let propagate_abort = &format!(
            "if (__abort_flag) {{\n  assume $DebugTrackAbort({}, {}, {});\n  goto Label_Abort;\n}}",
            func_env.module_env.get_module_idx(),
            func_env.get_def_idx(),
            loc.span().start(),
        );
        match bytecode {
            Branch(target) => emitln!(self.writer, "goto Label_{};", target),
            BrTrue(target, idx) => emitln!(
                self.writer,
                "__tmp := GetLocal(__m, __frame + {});\nif (b#Boolean(__tmp)) {{ goto Label_{}; }}",
                idx,
                target,
            ),
            BrFalse(target, idx) => emitln!(
                self.writer,
                "__tmp := GetLocal(__m, __frame + {});\nif (!b#Boolean(__tmp)) {{ goto Label_{}; }}",
                idx,
                target,
            ),
            MoveLoc(dest, src) => {
                if self.get_local_type(func_env, *dest).is_reference() {
                    emitln!(
                        self.writer,
                        "call __t{} := CopyOrMoveRef({});",
                        dest,
                        func_env.get_local_name(*src as usize)
                    );
                } else {
                    emitln!(
                        self.writer,
                        "call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + {}));",
                        src
                    );
                    emitln!(self.writer, &update_and_track_local(*dest, "__tmp"));
                }
            }
            CopyLoc(dest, src) => {
                if self.get_local_type(func_env, *dest).is_reference() {
                    emitln!(
                        self.writer,
                        "call __t{} := CopyOrMoveRef({});",
                        dest,
                        func_env.get_local_name(*src as usize)
                    )
                } else {
                    emitln!(
                        self.writer,
                        "call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + {}));",
                        src
                    );
                    emitln!(self.writer, &update_and_track_local(*dest, "__tmp"));
                }
            }
            StLoc(dest, src) => {
                if self.get_local_type(func_env, *dest as usize).is_reference() {
                    let name = func_env.get_local_name(*dest as usize);
                    emitln!(
                        self.writer,
                        "call {} := CopyOrMoveRef(__t{});",
                        &name,
                        src
                    );
                    emitln!(self.writer, &track_local(*dest as usize, &name))
                } else {
                    emitln!(
                        self.writer,
                        "call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + {}));",
                        src
                    );
                    emitln!(self.writer, &update_and_track_local(*dest as usize, "__tmp"));
                }
            }
            BorrowLoc(dest, src) => {
                emitln!(
                    self.writer,
                    "call __t{} := BorrowLoc(__frame + {}, {});",
                    dest,
                    src,
                    boogie_type_value(func_env.module_env.env,
                       &func_env.get_local_type(*src as usize))
                );
                save_borrowed_value(dest);
            }
            ReadRef(dest, src) => {
                emitln!(self.writer, "call __tmp := ReadRef(__t{});", src);
                emit!(
                    self.writer,
                    &boogie_type_check(
                        self.module_env.env,
                        "__tmp",
                        &self.get_local_type(func_env, *dest)
                    )
                );
                emitln!(self.writer, &update_and_track_local(*dest, "__tmp"));
            }
            WriteRef(dest, src) => {
                // TODO: for invariants, without analysis of reaching definitions, we approximate the
                // types a WriteRef can effect by _all_ structs defined in the compilation target. This
                // should be replaced by a proper analysis.
                let reaching_root_types = func_env.module_env.env.get_all_structs_with_invariants();
                let write_ref = format!("call WriteRef(__t{}, GetLocal(__m, __frame + {}));\n{}", dest, src, track_mutual_refs());
                let spec_translator = SpecTranslator::new(self.writer, &func_env.module_env, func_env.get_type_parameters(), false);
                if self.module_env.env.options.invariant_model != InvariantModel::WriteRefBased
                    || !spec_translator.needs_update_invariant_check(&reaching_root_types) {
                    emitln!(self.writer, &write_ref);
                } else {
                    let target = format!("__t{}", dest);
                    let before = boogie_invariant_target(true);
                    let after = boogie_invariant_target(false);
                    emitln!(
                      self.writer,
                      "{} := Dereference(__m, RootReference({}));",
                      before,
                      target
                    );
                    emitln!(self.writer, &write_ref);
                    emitln!(
                      self.writer,
                      "{} := Dereference(__m, RootReference({}));",
                      after,
                      target
                    );
                    spec_translator.emit_update_invariant_check(&target, &before, &after, &reaching_root_types);
                }
            }
            FreezeRef(dest, src) => emitln!(self.writer, "call __t{} := FreezeRef(__t{});", dest, src),
            Call(dests, callee_index, type_actuals, args) => {
                let (callee_module_env, callee_def_idx) =
                    self.module_env.get_callee_info(*callee_index);
                let callee_env = callee_module_env.get_function(callee_def_idx);
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
                                format!("__t{}", arg_idx)
                            } else {
                                format!("GetLocal(__m, __frame + {})", arg_idx)
                            }
                        })
                        .join(", "),
                );
                dest_str.push_str(
                    &dests
                        .iter()
                        .map(|dest_idx| {
                            let dest = format!("__t{}", dest_idx);
                            let dest_type = &self.get_local_type(func_env, *dest_idx);
                            dest_type_assumptions.push(boogie_type_check(
                                self.module_env.env,
                                &dest,
                                dest_type,
                            ));
                            if !dest_type.is_reference() {
                                tmp_assignments.push(
                                    update_and_track_local(*dest_idx, &dest));
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
                let struct_env = func_env.module_env.get_struct(*struct_def_index);
                let effective_dest = effective_dest_func(*dest);
                let track_args =
                    if effective_dest < func_env.get_local_count() {
                        format!("{}, {}, {}, {}", func_env.module_env.get_module_idx(),
                                func_env.get_def_idx(), effective_dest_func(*dest), loc.span().start())
                    } else {
                        "0, 0, 0, 0".to_string()
                    };
                let args_str = func_env
                    .module_env
                    .get_type_actuals(*type_actuals)
                    .iter()
                    .map(|s| boogie_type_value(self.module_env.env, s))
                    .chain(
                        fields
                            .iter()
                            .map(|i| format!("GetLocal(__m, __frame + {})", i)),
                    )
                    .join(", ");
                emitln!(
                    self.writer,
                    "call __tmp := Pack_{}({}, {});",
                    boogie_struct_name(&struct_env),
                    track_args,
                    args_str
                );
                emitln!(
                    self.writer,
                    &update_and_track_local(*dest, "__tmp")
                );
            }
            Unpack(dests, struct_def_index, _, src) => {
                let struct_env = &func_env.module_env.get_struct(*struct_def_index);
                let mut dests_str = String::new();
                let mut tmp_assignments = vec![];
                for dest in dests.iter() {
                    if !dests_str.is_empty() {
                        dests_str.push_str(", ");
                    }
                    let dest_str = &format!("__t{}", dest);
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
                    "call {} := Unpack_{}(GetLocal(__m, __frame + {}));",
                    dests_str,
                    boogie_struct_name(struct_env),
                    src,
                );
                for s in &tmp_assignments {
                    emitln!(self.writer, s);
                }
            }
            BorrowField(dest, src, field_def_index) => {
                let struct_env = self.module_env.get_struct_of_field(*field_def_index);
                let field_env = &struct_env.get_field(*field_def_index);
                emitln!(
                    self.writer,
                    "call __t{} := BorrowField(__t{}, {});",
                    dest,
                    src,
                    boogie_field_name(field_env)
                );
                save_borrowed_value(dest);
            }
            Exists(dest, addr, struct_def_index, type_actuals) => {
                let resource_type = boogie_struct_type_value(
                    self.module_env.env,
                    self.module_env.get_module_idx(),
                    *struct_def_index,
                    &self.module_env.get_type_actuals(*type_actuals),
                );
                emitln!(
                    self.writer,
                    "call __tmp := Exists(GetLocal(__m, __frame + {}), {});",
                    addr,
                    resource_type
                );
                emitln!(self.writer, &update_and_track_local(*dest, "__tmp"));
            }
            BorrowGlobal(dest, addr, struct_def_index, type_actuals) => {
                let resource_type = boogie_struct_type_value(
                    self.module_env.env,
                    self.module_env.get_module_idx(),
                    *struct_def_index,
                    &self.module_env.get_type_actuals(*type_actuals),
                );
                emitln!(
                    self.writer,
                    "call __t{} := BorrowGlobal(GetLocal(__m, __frame + {}), {});",
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
                    self.module_env.get_module_idx(),
                    *struct_def_index,
                    &self.module_env.get_type_actuals(*type_actuals),
                );
                emitln!(
                    self.writer,
                    "call MoveToSender({}, GetLocal(__m, __frame + {}));",
                    resource_type,
                    src,
                );
                emitln!(self.writer, propagate_abort);
            }
            MoveFrom(dest, src, struct_def_index, type_actuals) => {
                let resource_type = boogie_struct_type_value(
                    self.module_env.env,
                    self.module_env.get_module_idx(),
                    *struct_def_index,
                    &self.module_env.get_type_actuals(*type_actuals),
                );
                emitln!(
                    self.writer,
                    "call __tmp := MoveFrom(GetLocal(__m, __frame + {}), {});",
                    src,
                    resource_type,
                );
                emitln!(self.writer, &update_and_track_local(*dest, "__tmp"));
                emit!(
                    self.writer,
                    &boogie_type_check(
                        self.module_env.env,
                        &format!("__t{}", dest),
                        &self.get_local_type(func_env, *dest)
                    )
                );
                emitln!(self.writer, propagate_abort);
            }
            Ret(rets) => {
                for (i, r) in rets.iter().enumerate() {
                    if self.get_local_type(func_env, *r).is_reference() {
                        emitln!(self.writer, "__ret{} := __t{};", i, r);
                    } else {
                        emitln!(self.writer, "__ret{} := GetLocal(__m, __frame + {});", i, r);
                    }
                    emitln!(self.writer, &track_return(i));
                }
                // Enforce invariants for mutual borrows in this scope.
                // TODO: this is a hack until we have lifetime analysis. In fact we should
                // enforce whenever the lifetime of mutual borrow ends, which maybe way
                // before the function returns. Also the variable may be overridden in the meantime,
                // so we only currently enforce the invariant of the most recent borrow here.
                for (idx, ty) in update_invariant_on_release.iter().enumerate() {
                    enforce_borrowed_invariant(ty, idx);
                }
                emitln!(self.writer, "return;");
            }
            LdTrue(idx) => {
                emitln!(self.writer, "call __tmp := LdTrue();");
                emitln!(self.writer, &update_and_track_local(*idx, "__tmp"));
            }
            LdFalse(idx) => {
                emitln!(self.writer, "call __tmp := LdFalse();");
                emitln!(self.writer, &update_and_track_local(*idx, "__tmp"));
            }
            LdU8(idx, num) => {
                emitln!(self.writer, "call __tmp := LdConst({});", num);
                emitln!(self.writer, &update_and_track_local(*idx, "__tmp"));
            }
            LdU64(idx, num) => {
                emitln!(self.writer, "call __tmp := LdConst({});", num);
                emitln!(self.writer, &update_and_track_local(*idx, "__tmp"));
            }
            LdU128(idx, num) => {
                emitln!(self.writer, "call __tmp := LdConst({});", num);
                emitln!(self.writer, &update_and_track_local(*idx, "__tmp"));
            }
            CastU8(dest, src) => {
                emitln!(
                    self.writer,
                    "call __tmp := CastU8(GetLocal(__m, __frame + {}));",
                    src
                );
                emitln!(self.writer, propagate_abort);
                emitln!(self.writer, &update_and_track_local(*dest, "__tmp"));
            }
            CastU64(dest, src) => {
                emitln!(
                    self.writer,
                    "call __tmp := CastU64(GetLocal(__m, __frame + {}));",
                    src
                );
                emitln!(self.writer, propagate_abort);
                emitln!(self.writer, &update_and_track_local(*dest, "__tmp"));
            }
            CastU128(dest, src) => {
                emitln!(
                    self.writer,
                    "call __tmp := CastU128(GetLocal(__m, __frame + {}));",
                    src
                );
                emitln!(self.writer, propagate_abort);
                emitln!(self.writer, &update_and_track_local(*dest, "__tmp"));
            }
            LdAddr(idx, addr_idx) => {
                let addr_int = self.module_env.get_address(*addr_idx);
                emitln!(self.writer, "call __tmp := LdAddr({});", addr_int);
                emitln!(self.writer, &update_and_track_local(*idx, "__tmp"));
            }
            Not(dest, operand) => {
                emitln!(
                    self.writer,
                    "call __tmp := Not(GetLocal(__m, __frame + {}));",
                    operand
                );
                emitln!(self.writer, &update_and_track_local(*dest, "__tmp"));
            }
            Add(dest, op1, op2) => {
                let add_type = match self.get_local_type(func_env, *dest) {
                    GlobalType::U8 => "U8",
                    GlobalType::U64 => "U64",
                    GlobalType::U128 => "U128",
                    _ => unreachable!(),
                };
                emitln!(
                    self.writer,
                    "call __tmp := Add{}(GetLocal(__m, __frame + {}), GetLocal(__m, __frame + {}));",
                    add_type,
                    op1,
                    op2
                );
                emitln!(self.writer, propagate_abort);
                emitln!(self.writer, &update_and_track_local(*dest, "__tmp"));
            }
            Sub(dest, op1, op2) => {
                emitln!(
                    self.writer,
                    "call __tmp := Sub(GetLocal(__m, __frame + {}), GetLocal(__m, __frame + {}));",
                    op1,
                    op2
                );
                emitln!(self.writer, propagate_abort);
                emitln!(self.writer, &update_and_track_local(*dest, "__tmp"));
            }
            Mul(dest, op1, op2) => {
                let mul_type = match self.get_local_type(func_env, *dest) {
                    GlobalType::U8 => "U8",
                    GlobalType::U64 => "U64",
                    GlobalType::U128 => "U128",
                    _ => unreachable!(),
                };
                emitln!(
                    self.writer,
                    "call __tmp := Mul{}(GetLocal(__m, __frame + {}), GetLocal(__m, __frame + {}));",
                    mul_type,
                    op1,
                    op2
                );
                emitln!(self.writer, propagate_abort);
                emitln!(self.writer, &update_and_track_local(*dest, "__tmp"));
            }
            Div(dest, op1, op2) => {
                emitln!(
                    self.writer,
                    "call __tmp := Div(GetLocal(__m, __frame + {}), GetLocal(__m, __frame + {}));",
                    op1,
                    op2
                );
                emitln!(self.writer, propagate_abort);
                emitln!(self.writer, &update_and_track_local(*dest, "__tmp"));
            }
            Mod(dest, op1, op2) => {
                emitln!(
                    self.writer,
                    "call __tmp := Mod(GetLocal(__m, __frame + {}), GetLocal(__m, __frame + {}));",
                    op1,
                    op2
                );
                emitln!(self.writer, propagate_abort);
                emitln!(self.writer, &update_and_track_local(*dest, "__tmp"));
            }
            Lt(dest, op1, op2) => {
                emitln!(
                    self.writer,
                    "call __tmp := Lt(GetLocal(__m, __frame + {}), GetLocal(__m, __frame + {}));",
                    op1,
                    op2
                );
                emitln!(self.writer, &update_and_track_local(*dest, "__tmp"));
            }
            Gt(dest, op1, op2) => {
                emitln!(
                    self.writer,
                    "call __tmp := Gt(GetLocal(__m, __frame + {}), GetLocal(__m, __frame + {}));",
                    op1,
                    op2
                );
                emitln!(self.writer, &update_and_track_local(*dest, "__tmp"));
            }
            Le(dest, op1, op2) => {
                emitln!(
                    self.writer,
                    "call __tmp := Le(GetLocal(__m, __frame + {}), GetLocal(__m, __frame + {}));",
                    op1,
                    op2
                );
                emitln!(self.writer, &update_and_track_local(*dest, "__tmp"));
            }
            Ge(dest, op1, op2) => {
                emitln!(
                    self.writer,
                    "call __tmp := Ge(GetLocal(__m, __frame + {}), GetLocal(__m, __frame + {}));",
                    op1,
                    op2
                );
                emitln!(self.writer, &update_and_track_local(*dest, "__tmp"));
            }
            Or(dest, op1, op2) => {
                emitln!(
                    self.writer,
                    "call __tmp := Or(GetLocal(__m, __frame + {}), GetLocal(__m, __frame + {}));",
                    op1,
                    op2
                );
                emitln!(self.writer, &update_and_track_local(*dest, "__tmp"));
            }
            And(dest, op1, op2) => {
                emitln!(
                    self.writer,
                    "call __tmp := And(GetLocal(__m, __frame + {}), GetLocal(__m, __frame + {}));",
                    op1,
                    op2
                );
                emitln!(self.writer, &update_and_track_local(*dest, "__tmp"));
            }
            Eq(dest, op1, op2) => {
                emitln!(
                    self.writer,
                    "__tmp := Boolean(IsEqual(GetLocal(__m, __frame + {}), GetLocal(__m, __frame + {})));",
                    op1,
                    op2
                );
                emitln!(self.writer, &update_and_track_local(*dest, "__tmp"));
            }
            Neq(dest, op1, op2) => {
                emitln!(
                    self.writer,
                    "__tmp := Boolean(!IsEqual(GetLocal(__m, __frame + {}), GetLocal(__m, __frame + {})));",
                    op1,
                    op2
                );
                emitln!(self.writer, &update_and_track_local(*dest, "__tmp"));
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
                    "if (true) {{ assume $DebugTrackAbort({}, {}, {}); }}",
                    func_env.module_env.get_module_idx(),
                    func_env.get_def_idx(),
                    loc.span().start(),
                );
                emitln!(self.writer, "goto Label_Abort;")
            },
            GetGasRemaining(idx) => {
                emitln!(self.writer, "call __tmp := GetGasRemaining();");
                emitln!(self.writer, &update_and_track_local(*idx, "__tmp"));
            }
            GetTxnSequenceNumber(idx) => {
                emitln!(self.writer, "call __tmp := GetTxnSequenceNumber();");
                emitln!(self.writer, &update_and_track_local(*idx, "__tmp"));
            }
            GetTxnPublicKey(idx) => {
                emitln!(self.writer, "call __tmp := GetTxnPublicKey();");
                emitln!(self.writer, &update_and_track_local(*idx, "__tmp"));
            }
            GetTxnSenderAddress(idx) => {
                emitln!(self.writer, "call __tmp := GetTxnSenderAddress();");
                emitln!(self.writer, &update_and_track_local(*idx, "__tmp"));
            }
            GetTxnMaxGasUnits(idx) => {
                emitln!(self.writer, "call __tmp := GetTxnMaxGasUnits();");
                emitln!(self.writer, &update_and_track_local(*idx, "__tmp"));
            }
            GetTxnGasUnitPrice(idx) => {
                emitln!(self.writer, "call __tmp := GetTxnGasUnitPrice();");
                emitln!(self.writer, &update_and_track_local(*idx, "__tmp"));
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
            .map(|TypeParameter(ref i, _)| format!("{}: TypeValue", i))
            .chain(
                func_env
                    .get_parameters()
                    .iter()
                    .map(|Parameter(ref i, ref s)| format!("{}: {}", i, boogie_local_type(s))),
            )
            .join(", ");
        let rets = func_env
            .get_return_types()
            .iter()
            .enumerate()
            .map(|(i, ref s)| format!("__ret{}: {}", i, boogie_local_type(s)))
            .join(", ");
        (args, rets)
    }

    /// Return string for the function specification.
    fn generate_function_spec(&self, func_env: &FunctionEnv<'_>) {
        emitln!(self.writer);
        SpecTranslator::new(
            self.writer,
            &func_env.module_env,
            func_env.get_type_parameters(),
            true,
        )
        .translate_conditions(func_env);
    }

    /// Return string for body of verify function, which is just a call to the
    /// inline version of the function.
    fn generate_verify_function_body(&self, func_env: &FunctionEnv<'_>) {
        // Set the location to pseudo module so it won't be counted for execution traces
        self.writer
            .set_location(PSEUDO_PRELUDE_MODULE, Spanned::unsafe_no_loc(()).loc);

        let args = func_env
            .get_type_parameters()
            .iter()
            .map(|TypeParameter(i, _)| i.as_str().to_string())
            .chain(
                func_env
                    .get_parameters()
                    .iter()
                    .map(|Parameter(i, _)| i.as_str().to_string()),
            )
            .join(", ");
        let rets = (0..func_env.get_return_count())
            .map(|i| format!("__ret{}", i))
            .join(", ");
        let assumptions = "    call InitVerification();\n";
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
        let mut update_invariant_on_release = vec![];
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
                            update_invariant_on_release.push(ty)
                        }
                    }
                }
                BorrowGlobal(dst, ..) => {
                    let ty = self.get_local_type(func_env, *dst);
                    if ty.is_mutual_reference() && self.has_update_invariant_on_release(&ty) {
                        update_invariant_on_release.push(ty)
                    }
                }
                _ => {}
            }
        }

        // Be sure to set back location to the whole function definition as a default, otherwise
        // we may get unassigned code locations associated with condition locations.
        self.writer
            .set_location(self.module_env.get_module_idx(), func_env.get_loc());

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
                local_name,
                boogie_local_type(local_type),
                boogie_type_value(self.module_env.env, local_type)
            );
        }
        emitln!(self.writer, "var __tmp: Value;");
        emitln!(self.writer, "var __frame: int;");
        emitln!(self.writer, "var __saved_m: Memory;");
        if self.module_env.env.options.invariant_model == InvariantModel::WriteRefBased {
            emitln!(
                self.writer,
                "var {}: Value;",
                boogie_invariant_target(false)
            );
            emitln!(self.writer, "var {}: Value;", boogie_invariant_target(true));
        } else {
            for idx in 0..update_invariant_on_release.len() {
                let name = boogie_var_before_borrow(idx);
                emitln!(self.writer, "var {}: Value;", name);
                emitln!(self.writer, "var {}_ref: Reference;", name);
            }
        }

        emitln!(self.writer, "\n// initialize function execution");
        emitln!(self.writer, "assume !__abort_flag;");
        emitln!(self.writer, "__saved_m := __m;");
        emitln!(self.writer, "__frame := __local_counter;");

        emitln!(self.writer, "\n// process and type check arguments");
        for i in 0..num_args {
            let local_name = func_env.get_local_name(i);
            let local_type = &self.module_env.globalize_signature(&code.local_types[i]);
            let type_check = boogie_type_check(self.module_env.env, &local_name, local_type);
            emit!(self.writer, &type_check);
            if !local_type.is_reference() {
                emitln!(
                    self.writer,
                    &self.update_and_track_local(func_env, func_env.get_loc(), i, &local_name)
                );
            } else {
                emitln!(
                    self.writer,
                    &self.track_local(func_env, func_env.get_loc(), i, &local_name)
                );
            }
        }

        emitln!(self.writer, "\n// increase the local counter ");
        emitln!(
            self.writer,
            "__local_counter := __local_counter + {};",
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
                &update_invariant_on_release,
            );
        }

        // Generate abort exit.
        let mut end_loc = func_env.get_loc();
        if end_loc.span().end().0 > 0 {
            end_loc = Loc::new(
                end_loc.file(),
                Span::new(end_loc.span().end() - ByteOffset(1), end_loc.span().end()),
            );
        }
        self.writer
            .set_location(self.module_env.get_module_idx(), end_loc);
        self.writer.unindent();
        emitln!(self.writer, "Label_Abort:");
        self.writer.indent();
        emitln!(self.writer, "__abort_flag := true;");
        emitln!(self.writer, "__m := __saved_m;");
        for (i, sig) in func_env.get_return_types().iter().enumerate() {
            let ret_str = format!("__ret{}", i);
            if sig.is_reference() {
                emitln!(self.writer, "{} := DefaultReference;", &ret_str);
            } else {
                emitln!(self.writer, "{} := DefaultValue;", &ret_str);
            }
        }
        self.writer.unindent();
        emitln!(self.writer, "}");
    }

    /// Looks up the type of a local in the stackless bytecode representation.
    fn get_local_type(&self, func_env: &FunctionEnv<'_>, local_idx: usize) -> GlobalType {
        self.module_env.globalize_signature(
            &self.stackless_bytecode[func_env.get_def_idx().0 as usize].local_types[local_idx],
        )
    }

    /// Determines whether this type has update invariants that shall be
    /// enforced on mutual reference release.
    fn has_update_invariant_on_release(&'env self, ty: &GlobalType) -> bool {
        if self.module_env.env.options.invariant_model == InvariantModel::LifetimeBased {
            if let GlobalType::MutableReference(bt) = &ty {
                if let GlobalType::Struct(module_idx, struct_idx, _) = bt.as_ref() {
                    let struct_env = self
                        .module_env
                        .env
                        .get_module(*module_idx)
                        .into_get_struct(*struct_idx);
                    if !struct_env.get_data_invariants().is_empty()
                        || !struct_env.get_update_invariants().is_empty()
                    {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Updates a local, injecting debug information if available.
    fn update_and_track_local(
        &self,
        func_env: &FunctionEnv<'_>,
        loc: Loc,
        idx: usize,
        value: &str,
    ) -> String {
        let update = format!("__m := UpdateLocal(__m, __frame + {}, {});", idx, value);
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
        if self.module_env.env.options.omit_model_debug {
            return "".to_string();
        }
        let sig = if idx < func_env.get_local_count() {
            func_env.get_local_type(idx)
        } else {
            func_env.get_return_types()[idx - func_env.get_local_count()].clone()
        };
        if let GlobalType::Reference(bt) | GlobalType::MutableReference(bt) = sig {
            let deref = format!("Dereference(__m, {})", value);
            let type_check = boogie_type_check(self.module_env.env, &deref, &*bt);
            format!(
                "{}assume $DebugTrackLocal({}, {}, {}, {}, {});",
                type_check,
                func_env.module_env.get_module_idx(),
                func_env.get_def_idx(),
                idx,
                loc.span().start(),
                deref
            )
        } else {
            format!(
                "assume $DebugTrackLocal({}, {}, {}, {}, {});",
                func_env.module_env.get_module_idx(),
                func_env.get_def_idx(),
                idx,
                loc.span().start(),
                value
            )
        }
    }
}

/// Separates elements in vector, dropping empty ones.
fn separate(elems: Vec<String>, sep: &str) -> String {
    elems.iter().filter(|s| !s.is_empty()).join(sep)
}
