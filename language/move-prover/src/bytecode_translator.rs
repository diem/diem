// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module translates the bytecode of a module to Boogie code.

use std::collections::BTreeSet;

use itertools::Itertools;
#[allow(unused_imports)]
use log::{debug, info, log, warn, Level};

use spec_lang::{
    code_writer::CodeWriter,
    emit, emitln,
    env::{GlobalEnv, Loc, ModuleEnv, StructEnv, TypeParameter},
    ty::{PrimitiveType, Type},
};
use stackless_bytecode_generator::{
    function_target::FunctionTarget,
    function_target_pipeline::FunctionTargetsHolder,
    stackless_bytecode::{
        AssignKind, BorrowNode,
        Bytecode::{self, *},
        Constant, Operation, SpecBlockId,
    },
};
use vm::file_format::CodeOffset;

use crate::{
    boogie_helpers::{
        boogie_byte_blob, boogie_field_name, boogie_function_name, boogie_local_type,
        boogie_requires_well_formed, boogie_struct_name, boogie_struct_type_value,
        boogie_type_value, boogie_type_values, boogie_well_formed_check, WellFormedMode,
    },
    cli::{Options, VerificationScope},
    spec_translator::SpecTranslator,
};

pub struct BoogieTranslator<'env> {
    env: &'env GlobalEnv,
    writer: &'env CodeWriter,
    options: &'env Options,
    targets: &'env FunctionTargetsHolder,
}

pub struct ModuleTranslator<'env> {
    writer: &'env CodeWriter,
    options: &'env Options,
    module_env: ModuleEnv<'env>,
    targets: &'env FunctionTargetsHolder,
}

/// A struct encapsulating information which is threaded through translating the bytecodes of
/// a single function. This holds information which is relevant across multiple bytecode
/// instructions, like borrowing information and label offsets.
struct BytecodeContext {
    /// Set of mutable references, represented by local index. Used for debug tracking. Currently,
    /// after each mutation (either by an instruction or by call to a function with mutable
    /// parameters), we dump tracking info for all the variables in this set. This is a vast
    /// over-approximation; however, the execution trace visualizer will remove redundant
    /// entries, so it is more of a performance concern.
    mutable_refs: BTreeSet<usize>,
}

impl Default for BytecodeContext {
    fn default() -> Self {
        Self {
            mutable_refs: BTreeSet::new(),
        }
    }
}

impl<'env> BoogieTranslator<'env> {
    pub fn new(
        env: &'env GlobalEnv,
        options: &'env Options,
        targets: &'env FunctionTargetsHolder,
        writer: &'env CodeWriter,
    ) -> Self {
        Self {
            env,
            targets,
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
        Self {
            writer: parent.writer,
            options: parent.options,
            module_env: module,
            targets: &parent.targets,
        }
    }

    /// Translates this module.
    fn translate(&mut self) {
        log!(
            if self.module_env.is_in_dependency() {
                Level::Debug
            } else {
                Level::Info
            },
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

        let mut param_types = String::from("MapConstTypeValue(DefaultTypeValue())");
        let type_param_count = struct_env.get_type_parameters().len();
        for i in 0..type_param_count {
            param_types = format!("{}[{} := $tv{}]", param_types, i, i);
        }
        let type_param_array = format!("TypeValueArray({}, {})", param_types, type_param_count);
        let mut field_types = String::from("MapConstTypeValue(DefaultTypeValue())");
        for field_env in struct_env.get_fields() {
            field_types = format!(
                "{}[{} := {}]",
                field_types,
                field_env.get_offset(),
                boogie_type_value(self.module_env.env, &field_env.get_type())
            );
        }
        let field_array = format!(
            "TypeValueArray({}, {})",
            field_types,
            struct_env.get_field_count()
        );
        let type_value = format!(
            "StructType({}, {}, {})",
            struct_name, type_param_array, field_array
        );
        if struct_name == "$LibraAccount_T" {
            // Special treatment of well-known resource LibraAccount_T. The type_value
            // function is forward-declared in the prelude, here we only add an axiom for it.
            emitln!(
                self.writer,
                "axiom {}_type_value() == {};",
                struct_name,
                type_value
            );
        } else if struct_name == "$LibraAccount_Balance" {
            // Special treatment of well-known resource LibraAccount_Balance. The type_value
            // function is forward-declared in the prelude, here we only add an axiom for it.
            emitln!(
                self.writer,
                "axiom (forall $tv0: TypeValue :: {}_type_value($tv0) == {});",
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
            separate(vec![type_args_str.clone(), args_str.clone()], ", ")
        );
        self.writer.indent();
        let mut ctor_expr = "MapConstValue(DefaultValue())".to_owned();
        for field_env in struct_env.get_fields() {
            let field_param =
                &format!("{}", field_env.get_name().display(struct_env.symbol_pool()));
            let type_check = boogie_well_formed_check(
                self.module_env.env,
                field_param,
                &field_env.get_type(),
                WellFormedMode::Default,
            );
            emit!(self.writer, &type_check);
            ctor_expr = format!(
                "{}[{} := {}]",
                ctor_expr,
                field_env.get_offset(),
                field_param
            );
        }
        emitln!(
            self.writer,
            "$struct := Vector(ValueArray({}, {}));",
            ctor_expr,
            struct_env.get_field_count()
        );

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
            "procedure {{:inline 1}} {}_unpack({}) returns ({})\n{{",
            boogie_struct_name(struct_env),
            separate(vec![type_args_str, "$struct: Value".to_string()], ", "),
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
            let type_check = boogie_well_formed_check(
                self.module_env.env,
                &format!("{}", field_env.get_name().display(struct_env.symbol_pool())),
                &field_env.get_type(),
                WellFormedMode::Default,
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
            if !func_env.get_spec().has_conditions() && !func_env.is_native() {
                num_fun_specified += 1;
            }
            self.writer.set_location(&func_env.get_loc());
            self.translate_function(&self.targets.get_target(&func_env));
        }
        if num_fun > 0 && !self.module_env.is_in_dependency() {
            debug!(
                "{} out of {} functions have (directly or indirectly) \
                 specifications in module `{}`",
                num_fun_specified,
                num_fun,
                self.module_env
                    .get_name()
                    .display_full(self.module_env.symbol_pool())
            );
        }
    }
}

impl<'env> ModuleTranslator<'env> {
    /// Translates the given function.
    fn translate_function(&self, func_target: &FunctionTarget<'_>) {
        if func_target.is_native() {
            if self.options.native_stubs {
                self.generate_function_sig(func_target, true);
                emit!(self.writer, ";");
                self.generate_function_spec(func_target);
                emitln!(self.writer);
            }
            return;
        }

        // generate inline function with function body
        self.generate_function_sig(func_target, true); // inlined version of function
        self.generate_function_args_requires_well_formed(func_target);
        self.generate_function_spec(func_target);
        self.generate_inline_function_body(func_target);
        emitln!(self.writer);

        // If the function should not have a `_verify` entry point, stop here.
        if !self.should_generate_verify(func_target) {
            return;
        }

        // generate the _verify version of the function which calls inline version for standalone
        // verification.
        self.generate_function_sig(func_target, false); // no inline
        self.generate_function_args_requires_well_formed(func_target);
        self.generate_verify_function_body(func_target); // function body just calls inlined version
    }

    /// Determines whether we should generate the `_verify` entry point for a function, which
    /// triggers its standalone verification.
    fn should_generate_verify(&self, func_target: &FunctionTarget<'_>) -> bool {
        if func_target.func_env.module_env.is_in_dependency() {
            // Never generate verify method for functions from dependencies.
            return false;
        }
        // We look up the `verify` pragma property first in this function, then in
        // the module, and finally fall back to the value of option `--verify`.
        let default = || match self.options.verify_scope {
            VerificationScope::Public => func_target.func_env.is_public(),
            VerificationScope::All => true,
            VerificationScope::None => false,
        };
        func_target.is_pragma_true("verify", default)
    }

    /// Return a string for a boogie procedure header.
    /// if inline = true, add the inline attribute and use the plain function name
    /// for the procedure name. Also inject pre/post conditions if defined.
    /// Else, generate the function signature without the ":inline" attribute, and
    /// append _verify to the function name.
    fn generate_function_sig(&self, func_target: &FunctionTarget<'_>, inline: bool) {
        let (args, rets) = self.generate_function_args_and_returns(func_target);
        if inline {
            emit!(
                self.writer,
                "procedure {{:inline 1}} {} ({}) returns ({})",
                boogie_function_name(func_target.func_env),
                args,
                rets,
            )
        } else {
            emit!(
                self.writer,
                "procedure {}_verify ({}) returns ({})",
                boogie_function_name(func_target.func_env),
                args,
                rets
            )
        }
    }

    /// Generate boogie representation of function args and return args.
    fn generate_function_args_and_returns(
        &self,
        func_target: &FunctionTarget<'_>,
    ) -> (String, String) {
        let args = func_target
            .get_type_parameters()
            .iter()
            .map(|TypeParameter(s, _)| {
                format!("{}: TypeValue", s.display(func_target.symbol_pool()))
            })
            .chain((0..func_target.get_parameter_count()).map(|i| {
                let s = func_target.get_local_name(i);
                let ty = func_target.get_local_type(i);
                format!(
                    "{}: {}",
                    s.display(func_target.symbol_pool()),
                    boogie_local_type(ty)
                )
            }))
            .join(", ");
        let rets = func_target
            .get_return_types()
            .iter()
            .enumerate()
            .map(|(i, ref s)| format!("$ret{}: {}", i, boogie_local_type(s)))
            .join(", ");
        (args, rets)
    }

    /// Generate preconditions to make sure procedure parameters are well formed
    fn generate_function_args_requires_well_formed(&self, func_target: &FunctionTarget<'_>) {
        emitln!(self.writer);
        let num_args = func_target.get_parameter_count();
        let mode = if func_target.is_public() {
            // For public functions, we always include invariants in type assumptions for parameters,
            // even for mutable references.
            WellFormedMode::WithInvariant
        } else {
            WellFormedMode::Default
        };
        for i in 0..num_args {
            let local_name = func_target.get_local_name(i);
            let local_str = format!("{}", local_name.display(func_target.symbol_pool()));
            let local_type = func_target.get_local_type(i);
            let type_check = boogie_requires_well_formed(
                self.module_env.env,
                &local_str,
                local_type,
                mode,
                &self.options.template_context.type_requires,
            );
            emit!(self.writer, &type_check);
        }
    }

    /// Emit code for the function specification.
    fn generate_function_spec(&self, func_target: &FunctionTarget<'_>) {
        SpecTranslator::new_for_spec_in_impl(self.writer, func_target, true).translate_conditions();
    }

    /// Emit code for spec inside function implementation.
    fn generate_function_spec_inside_impl(
        &self,
        func_target: &FunctionTarget<'_>,
        block_id: SpecBlockId,
    ) {
        SpecTranslator::new_for_spec_in_impl(self.writer, func_target, true)
            .translate_conditions_inside_impl(block_id);
    }

    /// Return string for body of verify function, which is just a call to the
    /// inline version of the function.
    fn generate_verify_function_body(&self, func_target: &FunctionTarget<'_>) {
        // Set the location to internal so it won't be counted for execution traces
        self.writer
            .set_location(&self.module_env.env.internal_loc());
        emitln!(self.writer, "{");
        self.writer.indent();

        // Generate assumes for top-level verification entry
        // (a) init prelude specific stuff.
        emitln!(self.writer, "call $InitVerification();");

        // (b) assume implicit preconditions.
        let spec_translator = SpecTranslator::new_for_spec_in_impl(self.writer, func_target, false);
        spec_translator.assume_preconditions();

        // (c) assume reference parameters to be based on the Param(i) Location, ensuring
        // they are disjoint from all other references. This prevents aliasing and is justified as
        // follows:
        // - for mutual references, by their exclusive access in Move.
        // - for immutable references, by that mutation is not possible, and they are equivalent
        //   to some given but arbitrary value.
        for i in 0..func_target.get_parameter_count() {
            let ty = func_target.get_local_type(i);
            if ty.is_reference() {
                let name = func_target
                    .symbol_pool()
                    .string(func_target.get_local_name(i));
                emitln!(self.writer, "assume l#Reference({}) == Param({});", name, i);
                emitln!(self.writer, "assume size#Path(p#Reference({})) == 0;", name);
            }
        }

        // Generate call to inlined function.
        let args = func_target
            .get_type_parameters()
            .iter()
            .map(|TypeParameter(s, _)| format!("{}", s.display(func_target.symbol_pool())))
            .chain((0..func_target.get_parameter_count()).map(|i| {
                format!(
                    "{}",
                    func_target
                        .get_local_name(i)
                        .display(func_target.symbol_pool())
                )
            }))
            .join(", ");
        let rets = (0..func_target.get_return_count())
            .map(|i| format!("$ret{}", i))
            .join(", ");
        if rets.is_empty() {
            emitln!(
                self.writer,
                "call {}({});",
                boogie_function_name(func_target.func_env),
                args
            )
        } else {
            emitln!(
                self.writer,
                "call {} := {}({});",
                rets,
                boogie_function_name(func_target.func_env),
                args
            )
        }
        self.writer.unindent();
        emitln!(self.writer, "}");
        emitln!(self.writer);
    }

    /// This generates boogie code for everything after the function signature
    /// The function body is only generated for the "inline" version of the function.
    fn generate_inline_function_body(&self, func_target: &FunctionTarget<'_>) {
        let code = func_target.get_bytecode();

        // Construct context for bytecode translation.
        let mut context = BytecodeContext::default();

        // Walk over the bytecode and collect various context information.
        for bytecode in code {
            match bytecode {
                Call(_, dsts, oper, _) => {
                    use Operation::*;
                    match oper {
                        BorrowLoc | BorrowGlobal(..) | BorrowField(..) => {
                            let dst = dsts[0];
                            let ty = func_target.get_local_type(dst);
                            if ty.is_mutable_reference() {
                                // Track that we create a mutable reference here.
                                context.mutable_refs.insert(dst);
                            }
                        }
                        _ => {}
                    }
                }
                Assign(_, dst, src, AssignKind::Move) | Assign(_, dst, src, AssignKind::Store) => {
                    // Propagate information from src to dst.
                    if context.mutable_refs.contains(src) {
                        context.mutable_refs.insert(*dst);
                    }
                }
                _ => {}
            }
        }

        // Be sure to set back location to the whole function definition as a default, otherwise
        // we may get unassigned code locations associated with condition locations.
        self.writer.set_location(&func_target.get_loc());

        emitln!(self.writer, "{");
        self.writer.indent();

        // Generate local variable declarations. They need to appear first in boogie.
        emitln!(self.writer, "// declare local variables");
        let num_args = func_target.get_parameter_count();
        for i in num_args..func_target.get_local_count() {
            let local_name = func_target.get_local_name(i);
            let local_type = func_target.get_local_type(i);
            emitln!(
                self.writer,
                "var {}: {}; // {}",
                local_name.display(func_target.symbol_pool()),
                boogie_local_type(local_type),
                boogie_type_value(self.module_env.env, local_type)
            );
        }
        emitln!(self.writer, "var $tmp: Value;");
        emitln!(self.writer, "var $saved_m: Memory;");

        emitln!(self.writer, "\n// initialize function execution");
        emitln!(self.writer, "assert !$abort_flag;");
        emitln!(self.writer, "$saved_m := $m;");

        emitln!(self.writer, "\n// bytecode translation starts here");

        // Generate bytecode
        for (offset, bytecode) in code.iter().enumerate() {
            self.translate_bytecode(func_target, &context, offset as CodeOffset, bytecode);
        }

        // Generate abort exit.
        let end_loc = func_target.get_loc().at_end();
        self.writer.set_location(&end_loc);
        self.writer.unindent();
        emitln!(self.writer, "Abort:");
        self.writer.indent();
        emitln!(self.writer, "$abort_flag := true;");
        emitln!(self.writer, "$m := $saved_m;");
        for (i, ty) in func_target.get_return_types().iter().enumerate() {
            let ret_str = format!("$ret{}", i);
            if ty.is_reference() {
                emitln!(self.writer, "{} := DefaultReference;", &ret_str);
            } else {
                emitln!(self.writer, "{} := DefaultValue();", &ret_str);
            }
        }
        self.writer.unindent();
        emitln!(self.writer, "}");
    }

    /// Translates one bytecode instruction.
    fn translate_bytecode(
        &'env self,
        func_target: &FunctionTarget<'_>,
        ctx: &BytecodeContext,
        offset: u16,
        bytecode: &Bytecode,
    ) {
        // Set location of this code in the CodeWriter.
        let loc = func_target.get_bytecode_loc(bytecode.get_attr_id());
        self.writer.set_location(&loc);
        emitln!(self.writer, "// {}", bytecode.display(func_target));

        // Helper function to get an Rc<String> for a local.
        let str_local = |idx: usize| {
            func_target
                .symbol_pool()
                .string(func_target.get_local_name(idx))
        };

        // Helper functions to update a local including debug tracking.
        let update_and_track_local = |idx: usize, value: &str| {
            self.update_and_track_local(func_target, loc.clone(), idx, value)
        };

        // Helper functions to debug track a local.
        let track_local =
            |idx: usize, value: &str| self.track_local(func_target, loc.clone(), idx, value);

        // Helper functions to debug track a return value.
        let track_return = |idx: usize| {
            self.track_local(
                func_target,
                loc.clone(),
                func_target.get_local_count() + idx,
                &format!("$ret{}", idx),
            )
        };

        // Helper function to debug track potential updates of references.
        let track_mutable_refs = |ctx: &BytecodeContext| {
            for idx in &ctx.mutable_refs {
                if *idx < func_target.get_local_count() {
                    let s =
                        self.track_local(func_target, loc.clone(), *idx, str_local(*idx).as_str());
                    if !s.is_empty() {
                        emitln!(self.writer, &s);
                    }
                }
            }
            // Add reference parameter because we also want to debug track them when
            // references are written.
            for idx in 0..func_target.get_parameter_count() {
                let ty = func_target.get_local_type(idx);
                if ty.is_mutable_reference() {
                    let s =
                        self.track_local(func_target, loc.clone(), idx, str_local(idx).as_str());
                    if !s.is_empty() {
                        emitln!(self.writer, &s);
                    }
                }
            }
        };

        let propagate_abort = || {
            format!(
                "if ($abort_flag) {{\n  assume $DebugTrackAbort({}, {});\n  goto Abort;\n}}",
                func_target
                    .func_env
                    .module_env
                    .env
                    .file_id_to_idx(loc.file_id()),
                loc.span().start(),
            )
        };

        // Translate the bytecode instruction.
        match bytecode {
            UnpackRef(_, src) => {
                self.enforce_before_update_invariant(func_target, *src);
            }
            PackRef(_, src) => {
                self.enforce_after_update_invariant(func_target, *src);
            }
            WriteBack(_, dest, src) => {
                use BorrowNode::*;
                match dest {
                    GlobalRoot(_) => {
                        emitln!(self.writer, "call WritebackToGlobal({});", str_local(*src));
                    }
                    LocalRoot(idx) => {
                        emitln!(
                            self.writer,
                            "call {} := WritebackToValue({}, {}, {});",
                            str_local(*idx),
                            str_local(*src),
                            idx,
                            str_local(*idx)
                        );
                    }
                    Reference(idx) => {
                        emitln!(
                            self.writer,
                            "call {} := WritebackToReference({}, {});",
                            str_local(*idx),
                            str_local(*src),
                            str_local(*idx)
                        );
                    }
                }
            }
            Splice(_, dest, srcs) => {
                assert!(!srcs.is_empty());
                emitln!(
                    self.writer,
                    "call {} := Splice{}({}, {});",
                    str_local(*dest),
                    srcs.len(),
                    srcs.iter()
                        .map(|(pos, idx)| format!("{}, {}", pos, str_local(*idx)))
                        .join(", "),
                    str_local(*dest)
                );
            }
            SpecBlock(_, block_id) => {
                self.generate_function_spec_inside_impl(func_target, *block_id);
            }
            Label(_, label) => {
                self.writer.unindent();
                emitln!(self.writer, "L{}:", label.as_usize());
                self.writer.indent();
            }
            Jump(_, target) => emitln!(self.writer, "goto L{};", target.as_usize()),
            Branch(_, then_target, else_target, idx) => emitln!(
                self.writer,
                "$tmp := {};\nif (b#Boolean($tmp)) {{ goto L{}; }} else {{ goto L{}; }}",
                str_local(*idx),
                then_target.as_usize(),
                else_target.as_usize(),
            ),
            Assign(_, dest, src, _) => {
                if func_target.get_local_type(*dest).is_reference() {
                    emitln!(
                        self.writer,
                        "call {} := $CopyOrMoveRef({});",
                        str_local(*dest),
                        str_local(*src)
                    );
                    let track = track_local(*dest, str_local(*dest).as_str());
                    if !track.is_empty() {
                        emitln!(self.writer, &track);
                    }
                } else {
                    emitln!(
                        self.writer,
                        "call $tmp := $CopyOrMoveValue({});",
                        str_local(*src)
                    );
                    emit!(
                        self.writer,
                        &boogie_well_formed_check(
                            self.module_env.env,
                            "$tmp",
                            &func_target.get_local_type(*dest),
                            WellFormedMode::Default
                        )
                    );
                    emitln!(self.writer, &update_and_track_local(*dest, "$tmp"));
                }
            }
            Ret(_, rets) => {
                for (i, r) in rets.iter().enumerate() {
                    emitln!(self.writer, "$ret{} := {};", i, str_local(*r));
                    emitln!(self.writer, &track_return(i));
                }
                emitln!(self.writer, "return;");
            }
            Load(_, idx, c) => {
                let value = match c {
                    Constant::Bool(true) => "Boolean(true)".to_string(),
                    Constant::Bool(false) => "Boolean(false)".to_string(),
                    Constant::U8(num) => format!("Integer({})", num),
                    Constant::U64(num) => format!("Integer({})", num),
                    Constant::U128(num) => format!("Integer({})", num),
                    Constant::Address(val) => format!("Address({})", val),
                    Constant::TxnSenderAddress => "$TxnSender($txn)".to_string(),
                    Constant::ByteArray(val) => boogie_byte_blob(val),
                };
                emitln!(self.writer, "$tmp := {};", value);
                emitln!(self.writer, &update_and_track_local(*idx, "$tmp"));
            }
            Call(_, dests, oper, srcs) => {
                use Operation::*;
                match oper {
                    BorrowLoc => {
                        let src = srcs[0];
                        let dest = dests[0];
                        emitln!(
                            self.writer,
                            "call {} := $BorrowLoc({}, {});",
                            str_local(dest),
                            src,
                            str_local(src)
                        );
                        emit!(
                            self.writer,
                            &boogie_well_formed_check(
                                self.module_env.env,
                                str_local(dest).as_str(),
                                &func_target.get_local_type(dest),
                                // At the begining of a borrow, invariant holds.
                                WellFormedMode::WithInvariant,
                            )
                        );
                    }
                    ReadRef => {
                        let src = srcs[0];
                        let dest = dests[0];
                        emitln!(self.writer, "call $tmp := $ReadRef({});", str_local(src));
                        emit!(
                            self.writer,
                            &boogie_well_formed_check(
                                self.module_env.env,
                                "$tmp",
                                &func_target.get_local_type(dest),
                                WellFormedMode::Default
                            )
                        );
                        emitln!(self.writer, &update_and_track_local(dest, "$tmp"));
                    }
                    WriteRef => {
                        let reference = srcs[0];
                        let value = srcs[1];
                        emitln!(
                            self.writer,
                            "call {} := $WriteRef({}, {});",
                            str_local(reference),
                            str_local(reference),
                            str_local(value),
                        );
                        track_mutable_refs(ctx);
                    }
                    FreezeRef => unreachable!(), // eliminated by eliminate_imm_refs
                    Function(mid, fid, type_actuals) => {
                        let callee_env = self.module_env.env.get_module(*mid).into_function(*fid);
                        // If this is a call to a function from another module, assume the module invariants
                        // if any. This is correct because module invariants are guaranteed to hold whenever
                        // code outside of the module is executed.
                        if callee_env.module_env.get_id()
                            != func_target.func_env.module_env.get_id()
                        {
                            let spec_translator =
                                SpecTranslator::new(self.writer, &callee_env.module_env, false)
                                    .set_type_args(type_actuals.clone());
                            spec_translator
                                .assume_module_preconditions(&self.targets.get_target(&callee_env));
                        }

                        let mut dest_str = String::new();
                        let mut args_str = String::new();
                        let mut dest_type_assumptions = vec![];
                        args_str.push_str(&boogie_type_values(
                            func_target.func_env.module_env.env,
                            type_actuals,
                        ));
                        if !args_str.is_empty() && !srcs.is_empty() {
                            args_str.push_str(", ");
                        }
                        args_str.push_str(
                            &srcs
                                .iter()
                                .map(|arg_idx| format!("{}", str_local(*arg_idx)))
                                .join(", "),
                        );
                        dest_str.push_str(
                            &dests
                                .iter()
                                .map(|dest_idx| {
                                    let dest = str_local(*dest_idx).to_string();
                                    let dest_type = &func_target.get_local_type(*dest_idx);
                                    dest_type_assumptions.push(boogie_well_formed_check(
                                        self.module_env.env,
                                        &dest,
                                        dest_type,
                                        WellFormedMode::Default,
                                    ));
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
                        emitln!(self.writer, &propagate_abort());
                        for s in &dest_type_assumptions {
                            emitln!(self.writer, s);
                        }
                    }
                    Pack(mid, sid, type_actuals) => {
                        let dest = dests[0];
                        let struct_env = func_target
                            .func_env
                            .module_env
                            .env
                            .get_module(*mid)
                            .into_struct(*sid);
                        let effective_dest = self.compute_effective_dest(func_target, offset, dest);
                        let track_args = if effective_dest < func_target.get_user_local_count() {
                            format!(
                                "{}, {}, {}",
                                func_target
                                    .func_env
                                    .module_env
                                    .env
                                    .file_id_to_idx(loc.file_id()),
                                loc.span().start(),
                                effective_dest,
                            )
                        } else {
                            "0, 0, 0".to_string()
                        };
                        let args_str = type_actuals
                            .iter()
                            .map(|s| boogie_type_value(self.module_env.env, s))
                            .chain(srcs.iter().map(|i| format!("{}", str_local(*i))))
                            .join(", ");
                        emitln!(
                            self.writer,
                            "call $tmp := {}_pack({}, {});",
                            boogie_struct_name(&struct_env),
                            track_args,
                            args_str
                        );
                        emitln!(self.writer, &update_and_track_local(dest, "$tmp"));
                    }

                    Unpack(mid, sid, type_actuals) => {
                        let src = srcs[0];
                        let struct_env = func_target
                            .func_env
                            .module_env
                            .env
                            .get_module(*mid)
                            .into_struct(*sid);
                        let mut dests_str = String::new();
                        let mut tmp_assignments = vec![];
                        for dest in dests.iter() {
                            if !dests_str.is_empty() {
                                dests_str.push_str(", ");
                            }
                            let dest_str = str_local(*dest);
                            let dest_type = &func_target.get_local_type(*dest);
                            dests_str.push_str(dest_str.as_str());
                            if !dest_type.is_reference() {
                                tmp_assignments.push(update_and_track_local(*dest, &dest_str));
                            } else {
                                tmp_assignments.push(track_local(*dest, &dest_str));
                            }
                        }
                        let args_str = type_actuals
                            .iter()
                            .map(|s| boogie_type_value(self.module_env.env, s))
                            .chain(vec![format!("{}", str_local(src))])
                            .join(", ");
                        emitln!(
                            self.writer,
                            "call {} := {}_unpack({});",
                            dests_str,
                            boogie_struct_name(&struct_env),
                            args_str,
                        );
                        for s in &tmp_assignments {
                            emitln!(self.writer, s);
                        }
                    }
                    BorrowField(mid, sid, _, field_offset) => {
                        let src = srcs[0];
                        let dest = dests[0];
                        let struct_env = func_target
                            .func_env
                            .module_env
                            .env
                            .get_module(*mid)
                            .into_struct(*sid);
                        let field_env = &struct_env.get_field_by_offset(*field_offset);
                        emitln!(
                            self.writer,
                            "call {} := $BorrowField({}, {});",
                            str_local(dest),
                            str_local(src),
                            boogie_field_name(field_env)
                        );
                        emit!(
                            self.writer,
                            &boogie_well_formed_check(
                                self.module_env.env,
                                str_local(dest).as_str(),
                                &func_target.get_local_type(dest),
                                WellFormedMode::Default
                            )
                        );
                    }
                    GetField(mid, sid, _, field_offset) => {
                        let src = srcs[0];
                        let dest = dests[0];
                        let struct_env = func_target
                            .func_env
                            .module_env
                            .env
                            .get_module(*mid)
                            .into_struct(*sid);
                        let field_env = &struct_env.get_field_by_offset(*field_offset);
                        emitln!(
                            self.writer,
                            "call $tmp := {}({}, {});",
                            if func_target.get_local_type(src).is_reference() {
                                "$GetFieldFromReference"
                            } else {
                                "$GetFieldFromValue"
                            },
                            str_local(src),
                            boogie_field_name(field_env)
                        );
                        emit!(
                            self.writer,
                            &boogie_well_formed_check(
                                self.module_env.env,
                                "$tmp",
                                &func_target.get_local_type(dest),
                                WellFormedMode::Default
                            )
                        );
                        emitln!(self.writer, &update_and_track_local(dest, "$tmp"));
                    }
                    Exists(mid, sid, type_actuals) => {
                        let addr = srcs[0];
                        let dest = dests[0];
                        let resource_type =
                            boogie_struct_type_value(self.module_env.env, *mid, *sid, type_actuals);
                        emitln!(
                            self.writer,
                            "call $tmp := $Exists({}, {});",
                            str_local(addr),
                            resource_type
                        );
                        emitln!(self.writer, &update_and_track_local(dest, "$tmp"));
                    }
                    BorrowGlobal(mid, sid, type_actuals) => {
                        let addr = srcs[0];
                        let dest = dests[0];
                        let resource_type =
                            boogie_struct_type_value(self.module_env.env, *mid, *sid, type_actuals);
                        emitln!(
                            self.writer,
                            "call {} := $BorrowGlobal({}, {});",
                            str_local(dest),
                            str_local(addr),
                            resource_type,
                        );
                        emitln!(self.writer, &propagate_abort());
                        emit!(
                            self.writer,
                            &boogie_well_formed_check(
                                self.module_env.env,
                                str_local(dest).as_str(),
                                &func_target.get_local_type(dest),
                                // At the beginning of a borrow, invariants always hold
                                WellFormedMode::WithInvariant,
                            )
                        );
                    }
                    GetGlobal(mid, sid, type_actuals) => {
                        let addr = srcs[0];
                        let dest = dests[0];
                        let resource_type =
                            boogie_struct_type_value(self.module_env.env, *mid, *sid, type_actuals);
                        emitln!(
                            self.writer,
                            "call $tmp := $GetGlobal({}, {});",
                            str_local(addr),
                            resource_type,
                        );
                        emitln!(self.writer, &propagate_abort());
                        emit!(
                            self.writer,
                            &boogie_well_formed_check(
                                self.module_env.env,
                                "$tmp",
                                &func_target.get_local_type(dest),
                                // At the beginning of a borrow, invariants always hold
                                WellFormedMode::WithInvariant,
                            )
                        );
                        emitln!(self.writer, &update_and_track_local(dest, "$tmp"));
                    }
                    MoveToSender(mid, sid, type_actuals) => {
                        let src = srcs[0];
                        let resource_type =
                            boogie_struct_type_value(self.module_env.env, *mid, *sid, type_actuals);
                        emitln!(
                            self.writer,
                            "call $MoveToSender({}, {});",
                            resource_type,
                            str_local(src),
                        );
                        emitln!(self.writer, &propagate_abort());
                    }
                    MoveFrom(mid, sid, type_actuals) => {
                        let src = srcs[0];
                        let dest = dests[0];
                        let resource_type =
                            boogie_struct_type_value(self.module_env.env, *mid, *sid, type_actuals);
                        emitln!(
                            self.writer,
                            "call $tmp := $MoveFrom({}, {});",
                            str_local(src),
                            resource_type,
                        );
                        emitln!(self.writer, &propagate_abort());
                        emit!(
                            self.writer,
                            &boogie_well_formed_check(
                                self.module_env.env,
                                "$tmp",
                                &func_target.get_local_type(dest),
                                WellFormedMode::Default
                            )
                        );
                        emitln!(self.writer, &update_and_track_local(dest, "$tmp"));
                    }
                    CastU8 => {
                        let src = srcs[0];
                        let dest = dests[0];
                        emitln!(self.writer, "call $tmp := $CastU8({});", str_local(src));
                        emitln!(self.writer, &propagate_abort());
                        emitln!(self.writer, &update_and_track_local(dest, "$tmp"));
                    }
                    CastU64 => {
                        let src = srcs[0];
                        let dest = dests[0];
                        emitln!(self.writer, "call $tmp := $CastU64({});", str_local(src));
                        emitln!(self.writer, &propagate_abort());
                        emitln!(self.writer, &update_and_track_local(dest, "$tmp"));
                    }
                    CastU128 => {
                        let src = srcs[0];
                        let dest = dests[0];
                        emitln!(self.writer, "call $tmp := $CastU128({});", str_local(src));
                        emitln!(self.writer, &propagate_abort());
                        emitln!(self.writer, &update_and_track_local(dest, "$tmp"));
                    }
                    Not => {
                        let src = srcs[0];
                        let dest = dests[0];
                        emitln!(self.writer, "call $tmp := $Not({});", str_local(src));
                        emitln!(self.writer, &update_and_track_local(dest, "$tmp"));
                    }
                    Add => {
                        let dest = dests[0];
                        let op1 = srcs[0];
                        let op2 = srcs[1];
                        let add_type = match func_target.get_local_type(dest) {
                            Type::Primitive(PrimitiveType::U8) => "U8",
                            Type::Primitive(PrimitiveType::U64) => "U64",
                            Type::Primitive(PrimitiveType::U128) => "U128",
                            _ => unreachable!(),
                        };
                        emitln!(
                            self.writer,
                            "call $tmp := $Add{}({}, {});",
                            add_type,
                            str_local(op1),
                            str_local(op2)
                        );
                        emitln!(self.writer, &propagate_abort());
                        emitln!(self.writer, &update_and_track_local(dest, "$tmp"));
                    }
                    Sub => {
                        let dest = dests[0];
                        let op1 = srcs[0];
                        let op2 = srcs[1];
                        emitln!(
                            self.writer,
                            "call $tmp := $Sub({}, {});",
                            str_local(op1),
                            str_local(op2)
                        );
                        emitln!(self.writer, &propagate_abort());
                        emitln!(self.writer, &update_and_track_local(dest, "$tmp"));
                    }
                    Mul => {
                        let dest = dests[0];
                        let op1 = srcs[0];
                        let op2 = srcs[1];
                        let mul_type = match func_target.get_local_type(dest) {
                            Type::Primitive(PrimitiveType::U8) => "U8",
                            Type::Primitive(PrimitiveType::U64) => "U64",
                            Type::Primitive(PrimitiveType::U128) => "U128",
                            _ => unreachable!(),
                        };
                        emitln!(
                            self.writer,
                            "call $tmp := $Mul{}({}, {});",
                            mul_type,
                            str_local(op1),
                            str_local(op2)
                        );
                        emitln!(self.writer, &propagate_abort());
                        emitln!(self.writer, &update_and_track_local(dest, "$tmp"));
                    }
                    Div => {
                        let dest = dests[0];
                        let op1 = srcs[0];
                        let op2 = srcs[1];
                        emitln!(
                            self.writer,
                            "call $tmp := $Div({}, {});",
                            str_local(op1),
                            str_local(op2)
                        );
                        emitln!(self.writer, &propagate_abort());
                        emitln!(self.writer, &update_and_track_local(dest, "$tmp"));
                    }
                    Mod => {
                        let dest = dests[0];
                        let op1 = srcs[0];
                        let op2 = srcs[1];
                        emitln!(
                            self.writer,
                            "call $tmp := $Mod({}, {});",
                            str_local(op1),
                            str_local(op2)
                        );
                        emitln!(self.writer, &propagate_abort());
                        emitln!(self.writer, &update_and_track_local(dest, "$tmp"));
                    }
                    Shl => {
                        let dest = dests[0];
                        let op1 = srcs[0];
                        let op2 = srcs[1];
                        emitln!(
                            self.writer,
                            "call $tmp := $Shl({}, {});",
                            str_local(op1),
                            str_local(op2)
                        );
                        emitln!(self.writer, &update_and_track_local(dest, "$tmp"));
                    }
                    Shr => {
                        let dest = dests[0];
                        let op1 = srcs[0];
                        let op2 = srcs[1];
                        emitln!(
                            self.writer,
                            "call $tmp := $Shr({}, {});",
                            str_local(op1),
                            str_local(op2)
                        );
                        emitln!(self.writer, &update_and_track_local(dest, "$tmp"));
                    }
                    Lt => {
                        let dest = dests[0];
                        let op1 = srcs[0];
                        let op2 = srcs[1];
                        emitln!(
                            self.writer,
                            "call $tmp := $Lt({}, {});",
                            str_local(op1),
                            str_local(op2)
                        );
                        emitln!(self.writer, &update_and_track_local(dest, "$tmp"));
                    }
                    Gt => {
                        let dest = dests[0];
                        let op1 = srcs[0];
                        let op2 = srcs[1];
                        emitln!(
                            self.writer,
                            "call $tmp := $Gt({}, {});",
                            str_local(op1),
                            str_local(op2)
                        );
                        emitln!(self.writer, &update_and_track_local(dest, "$tmp"));
                    }
                    Le => {
                        let dest = dests[0];
                        let op1 = srcs[0];
                        let op2 = srcs[1];
                        emitln!(
                            self.writer,
                            "call $tmp := $Le({}, {});",
                            str_local(op1),
                            str_local(op2)
                        );
                        emitln!(self.writer, &update_and_track_local(dest, "$tmp"));
                    }
                    Ge => {
                        let dest = dests[0];
                        let op1 = srcs[0];
                        let op2 = srcs[1];
                        emitln!(
                            self.writer,
                            "call $tmp := $Ge({}, {});",
                            str_local(op1),
                            str_local(op2)
                        );
                        emitln!(self.writer, &update_and_track_local(dest, "$tmp"));
                    }
                    Or => {
                        let dest = dests[0];
                        let op1 = srcs[0];
                        let op2 = srcs[1];
                        emitln!(
                            self.writer,
                            "call $tmp := $Or({}, {});",
                            str_local(op1),
                            str_local(op2)
                        );
                        emitln!(self.writer, &update_and_track_local(dest, "$tmp"));
                    }
                    And => {
                        let dest = dests[0];
                        let op1 = srcs[0];
                        let op2 = srcs[1];
                        emitln!(
                            self.writer,
                            "call $tmp := $And({}, {});",
                            str_local(op1),
                            str_local(op2)
                        );
                        emitln!(self.writer, &update_and_track_local(dest, "$tmp"));
                    }
                    Eq => {
                        let dest = dests[0];
                        let op1 = srcs[0];
                        let op2 = srcs[1];
                        emitln!(
                            self.writer,
                            "$tmp := Boolean(IsEqual({}, {}));",
                            str_local(op1),
                            str_local(op2)
                        );
                        emitln!(self.writer, &update_and_track_local(dest, "$tmp"));
                    }
                    Neq => {
                        let dest = dests[0];
                        let op1 = srcs[0];
                        let op2 = srcs[1];
                        emitln!(
                            self.writer,
                            "$tmp := Boolean(!IsEqual({}, {}));",
                            str_local(op1),
                            str_local(op2)
                        );
                        emitln!(self.writer, &update_and_track_local(dest, "$tmp"));
                    }
                    BitOr | BitAnd | Xor => {
                        emitln!(
                            self.writer,
                            "// bit operation not supported: {:?}\nassert false;",
                            bytecode
                        );
                    }
                    Destroy => {}
                }
            }
            Abort(..) => {
                // Below we introduce a dummy `if` for $DebugTrackAbort to ensure boogie creates
                // a execution trace entry for this statement.
                emitln!(
                    self.writer,
                    "if (true) {{ assume $DebugTrackAbort({}, {}); }}",
                    func_target
                        .func_env
                        .module_env
                        .env
                        .file_id_to_idx(loc.file_id()),
                    loc.span().start(),
                );
                emitln!(self.writer, "goto Abort;")
            }
            Nop(..) => {}
        }

        emitln!(self.writer);
    }

    // Compute effective destination to enhance debug experience. This looks ahead
    // at the next instructions to detect a simple aliasing via copy or move and use that
    // destination as it might be a user variable whereas this instruction has a temporary
    // destination introduced by stackless bytecode transformation.
    // TODO: the stackless bytecode should optimize away unnecessary copy/moves, so we
    // don't need this. The below transformation is only correct for stackless code
    // of certain shape
    fn compute_effective_dest(
        &self,
        func_target: &FunctionTarget<'_>,
        offset: CodeOffset,
        dest: usize,
    ) -> usize {
        let code = func_target.get_bytecode();
        if dest >= func_target.get_local_count() && offset as usize + 1 < code.len() {
            if let Call(_, temp_dests, Operation::Pack(..), ..) = &code[offset as usize] {
                if let Assign(_, effective_dest, src, _) = &code[offset as usize + 1] {
                    if *src == temp_dests[0] {
                        return *effective_dest;
                    }
                }
            }
        }
        dest
    }

    /// If ty is a mutable reference to a struct, return its environment.
    fn get_referred_struct(&self, ty: &Type) -> Option<(StructEnv<'_>, Vec<Type>)> {
        if let Type::Reference(true, bt) = &ty {
            if let Type::Struct(module_idx, struct_idx, type_args) = bt.as_ref() {
                return Some((
                    self.module_env
                        .env
                        .get_module(*module_idx)
                        .into_struct(*struct_idx),
                    type_args.clone(),
                ));
            }
        }
        None
    }

    /// Enforce the invariant of an updated value before mutation starts. Does nothing if there
    /// is no before-update invariant.
    fn enforce_before_update_invariant(&self, func_target: &FunctionTarget<'_>, idx: usize) {
        if let Some((struct_env, type_args)) =
            self.get_referred_struct(func_target.get_local_type(idx))
        {
            if SpecTranslator::has_before_update_invariant(&struct_env) {
                let name = func_target
                    .symbol_pool()
                    .string(func_target.get_local_name(idx));
                let args_str = type_args
                    .iter()
                    .map(|ty| boogie_type_value(self.module_env.env, ty))
                    .chain(vec![format!("$Dereference({})", name)])
                    .join(", ");
                emitln!(
                    self.writer,
                    "call {}_before_update_inv({});",
                    boogie_struct_name(&struct_env),
                    args_str,
                );
            }
        }
    }

    /// Enforce the invariant of an updated value after mutation ended. Does nothing if there is
    /// no after-update invariant.
    fn enforce_after_update_invariant(&self, func_target: &FunctionTarget<'_>, idx: usize) {
        if let Some((struct_env, type_args)) =
            self.get_referred_struct(func_target.get_local_type(idx))
        {
            if SpecTranslator::has_after_update_invariant(&struct_env) {
                let name = func_target
                    .symbol_pool()
                    .string(func_target.get_local_name(idx));
                let args_str = type_args
                    .iter()
                    .map(|ty| boogie_type_value(self.module_env.env, ty))
                    .chain(vec![format!("$Dereference({})", name)])
                    .join(", ");
                emitln!(
                    self.writer,
                    "call {}_after_update_inv({});",
                    boogie_struct_name(&struct_env),
                    args_str,
                );
            }
        }
    }

    /// Updates a local, injecting debug information if available.
    fn update_and_track_local(
        &self,
        func_target: &FunctionTarget<'_>,
        loc: Loc,
        idx: usize,
        value: &str,
    ) -> String {
        let name = func_target
            .symbol_pool()
            .string(func_target.get_local_name(idx));
        let update = format!("{} := {};", name, value);
        let debug_update = self.track_local(func_target, loc, idx, value);
        if !debug_update.is_empty() {
            format!("{}\n{}", update, debug_update)
        } else {
            update
        }
    }

    /// Generates an update of the model debug variable at given location.
    fn track_local(
        &self,
        func_target: &FunctionTarget<'_>,
        loc: Loc,
        idx: usize,
        value: &str,
    ) -> String {
        // Check whether this is a temporary, which we do not want to track. Indices >=
        // local_count are return values which we do track.
        if idx >= func_target.get_user_local_count() && idx < func_target.get_local_count() {
            return "".to_string();
        }
        let ty = if idx < func_target.get_local_count() {
            func_target.get_local_type(idx)
        } else {
            func_target.get_return_type(idx - func_target.get_local_count())
        };
        let value = if ty.is_reference() {
            format!("$Dereference({})", value)
        } else {
            value.to_string()
        };
        format!(
            "if (true) {{ assume $DebugTrackLocal({}, {}, {}, {}); }}",
            func_target
                .func_env
                .module_env
                .env
                .file_id_to_idx(loc.file_id()),
            loc.span().start(),
            idx,
            value
        )
    }
}

/// Separates elements in vector, dropping empty ones.
fn separate(elems: Vec<String>, sep: &str) -> String {
    elems.iter().filter(|s| !s.is_empty()).join(sep)
}
