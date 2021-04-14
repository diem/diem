// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module translates the bytecode of a module to Boogie code.

use std::collections::{BTreeMap, BTreeSet};

use itertools::Itertools;
#[allow(unused_imports)]
use log::{debug, info, log, warn, Level};

use bytecode::{
    function_target::FunctionTarget,
    function_target_pipeline::FunctionTargetsHolder,
    stackless_bytecode::{
        BorrowEdge, BorrowNode, Bytecode, Constant, HavocKind, Operation, StrongEdge,
    },
    verification_analysis,
};
use move_model::{
    code_writer::CodeWriter,
    emit, emitln,
    model::{GlobalEnv, ModuleEnv, StructEnv, TypeParameter},
    pragmas::{ADDITION_OVERFLOW_UNCHECKED_PRAGMA, SEED_PRAGMA, TIMEOUT_PRAGMA},
    ty::{PrimitiveType, Type},
};

use crate::{
    boogie_helpers::{
        boogie_byte_blob, boogie_debug_track_abort, boogie_debug_track_local,
        boogie_debug_track_return, boogie_equality_for_type, boogie_field_sel,
        boogie_function_name, boogie_local_type, boogie_modifies_memory_name,
        boogie_resource_memory_name, boogie_struct_name, boogie_temp, boogie_type_suffix,
        boogie_type_suffix_for_struct, boogie_type_value, boogie_type_value_array,
        boogie_type_values, boogie_well_formed_check,
    },
    spec_translator::SpecTranslator,
};
use boogie_backend::options::BoogieOptions;
use bytecode::{
    function_target_pipeline::FunctionVariant,
    stackless_bytecode::{AbortAction, PropKind},
};
use codespan::LineIndex;
use move_model::{
    ast::TempIndex,
    model::{Loc, NodeId, QualifiedId, StructId},
};

pub struct BoogieTranslator<'env> {
    env: &'env GlobalEnv,
    options: &'env BoogieOptions,
    writer: &'env CodeWriter,
    targets: &'env FunctionTargetsHolder,
    used_structs: BTreeSet<QualifiedId<StructId>>,
}

pub struct ModuleTranslator<'env> {
    writer: &'env CodeWriter,
    module_env: ModuleEnv<'env>,
    spec_translator: SpecTranslator<'env>,
    options: &'env BoogieOptions,
    targets: &'env FunctionTargetsHolder,
}

impl<'env> BoogieTranslator<'env> {
    pub fn new(
        env: &'env GlobalEnv,
        options: &'env BoogieOptions,
        targets: &'env FunctionTargetsHolder,
        writer: &'env CodeWriter,
    ) -> Self {
        Self {
            env,
            options,
            targets,
            writer,
            used_structs: Self::compute_used_structs(env, targets),
        }
    }

    /// Compute the set of structs which are used  by the verified or inlined
    /// functions in this program. This is used to prune generated equality for the
    /// structs which are actually in scope.
    fn compute_used_structs(
        env: &GlobalEnv,
        targets: &FunctionTargetsHolder,
    ) -> BTreeSet<QualifiedId<StructId>> {
        fn collect_structs(env: &GlobalEnv, set: &mut BTreeSet<QualifiedId<StructId>>, ty: &Type) {
            let mut visitor = |ty: &Type| {
                if let Type::Struct(mid, sid, _) = ty {
                    if set.insert(mid.qualified(*sid)) {
                        for field in env.get_module(*mid).into_struct(*sid).get_fields() {
                            collect_structs(env, set, &field.get_type())
                        }
                    }
                }
            };
            ty.visit(&mut visitor);
        }

        let mut res = BTreeSet::new();
        for module in env.get_modules() {
            for fun in module.get_functions() {
                for (_, target) in targets.get_targets(&fun) {
                    let info = verification_analysis::get_info(&target);
                    if info.verified || info.inlined {
                        for idx in 0..target.get_local_count() {
                            collect_structs(env, &mut res, &target.get_local_type(idx));
                        }
                    }
                }
            }
        }
        res
    }

    pub fn translate(&mut self) {
        // generate definitions for all modules.
        for module_env in self.env.get_modules() {
            ModuleTranslator::new(self, module_env).translate();
        }

        // generate stratified equality
        if !self.options.vector_theory.is_extensional() {
            self.translate_stratified_equality();
            self.translate_struct_equality();
        }
    }

    pub fn translate_stratified_equality(&self) {
        for depth in 0..self.options.stratification_depth {
            emitln!(
                self.writer,
                "function $IsEqual_{}(v1: $Value, v2: $Value): bool {{",
                if depth == 0 {
                    "stratified".to_string()
                } else {
                    format!("level{}", depth)
                }
            );
            self.writer.with_indent(|| {
                emitln!(self.writer, "(v1 == v2)");
                if depth == self.options.stratification_depth - 1 {
                    return;
                }
                emitln!(
                    self.writer,
                    "\
|| (is#$Vector(v1) && is#$Vector(v2) &&
   (var vec1, vec2 := $Unbox_vec(v1), $Unbox_vec(v2);
    LenVec(vec1) == LenVec(vec2) &&
    (forall i: int :: 0 <= i && i < LenVec(vec1) ==>
        $IsEqual_level{}(ReadVec(vec1,i), ReadVec(vec2,i)))))",
                    depth + 1
                );
                emitln!(self.writer, "|| $IsEqual_some_struct(v1, v2)");
            });
            emitln!(self.writer, "}");
        }
    }

    fn translate_struct_equality(&self) {
        emitln!(
            self.writer,
            "function {:inline} $IsEqual_some_struct(v1: $Value, v2: $Value): bool {"
        );
        self.writer.with_indent(|| {
            let mut sep = "";
            for module_env in self.env.get_modules() {
                for struct_env in module_env.get_structs() {
                    if !self.used_structs.contains(&struct_env.get_qualified_id()) {
                        continue;
                    }
                    if struct_env.is_native_or_intrinsic()
                        || struct_has_native_equality(&struct_env, self.options)
                    {
                        continue;
                    }
                    emitln!(self.writer, "{} (", sep);
                    self.writer.with_indent(|| {
                        let suffix = boogie_type_suffix_for_struct(&struct_env);
                        emitln!(
                            self.writer,
                            "$IsValidBox{}(v1) && $IsValidBox{}(v2) &&",
                            suffix,
                            suffix
                        );
                        emitln!(
                            self.writer,
                            "$IsEqual{}($Unbox{}(v1), $Unbox{}(v2))",
                            suffix,
                            suffix,
                            suffix
                        );
                    });
                    emitln!(self.writer, ")");
                    sep = "|| ";
                }
            }
            if sep.is_empty() {
                // No struct in this compilation.
                emitln!(self.writer, "false");
            }
        });
        emitln!(self.writer, "}");
    }
}

impl<'env> ModuleTranslator<'env> {
    /// Creates a new module translator.
    fn new(parent: &'env BoogieTranslator, module: ModuleEnv<'env>) -> Self {
        Self {
            writer: parent.writer,
            options: parent.options,
            module_env: module,
            spec_translator: SpecTranslator::new(parent.writer, &parent.env, parent.options),
            targets: &parent.targets,
        }
    }

    /// Translates this module.
    fn translate(&mut self) {
        log!(
            if !self.module_env.is_target() {
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
        self.spec_translator.translate_spec_vars(&self.module_env);
        self.spec_translator.translate_spec_funs(&self.module_env);
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
            if struct_env.is_native_or_intrinsic() {
                continue;
            }
            // Set the location to internal so we don't see locations of pack/unpack
            // in execution traces.
            self.writer
                .set_location(&self.module_env.env.internal_loc());
            self.translate_struct_type(&struct_env);
        }
    }

    /// Translates the given struct.
    fn translate_struct_type(&self, struct_env: &StructEnv<'_>) {
        let env = struct_env.module_env.env;
        // Emit TypeName
        let struct_name = boogie_struct_name(&struct_env);
        emitln!(self.writer, "const unique {}_name: $TypeName;", struct_name);

        // Emit data type
        emitln!(self.writer, "type {{:datatype}} {};", struct_name);

        // Emit constructor
        let fields = struct_env
            .get_fields()
            .map(|field| {
                format!(
                    "{}: {}",
                    field.get_name().display(env.symbol_pool()),
                    boogie_local_type(env, &field.get_type())
                )
            })
            .join(", ");
        emitln!(
            self.writer,
            "function {{:constructor}} {}({}): {};",
            struct_name,
            fields,
            struct_name
        );

        // Emit $Value embedding
        let suffix = boogie_type_suffix_for_struct(struct_env);
        emitln!(
            self.writer,
            "function {{:constructor}} $Box{}(s: {}): $Value;",
            suffix,
            struct_name
        );
        self.emit_function(
            &format!("$Unbox{}(x: $Value): {}", suffix, struct_name),
            || {
                emitln!(self.writer, "s#$Box_{}(x)", struct_name);
            },
        );
        self.emit_function(&format!("$IsValidBox{}(x: $Value): bool", suffix), || {
            emitln!(self.writer, "is#$Box_{}(x)", struct_name);
        });

        // Emit TypeValue constructor function.
        let type_params = struct_env
            .get_type_parameters()
            .iter()
            .enumerate()
            .map(|(i, _)| format!("$tv{}: $TypeValue", i))
            .join(", ");
        self.emit_function(
            &format!("{}_type_value({}): $TypeValue", struct_name, type_params),
            || {
                let type_args = struct_env
                    .get_type_parameters()
                    .iter()
                    .enumerate()
                    .map(|(i, _)| Type::TypeParameter(i as u16))
                    .collect_vec();
                emitln!(
                    self.writer,
                    "$StructType({}_name, {})",
                    struct_name,
                    boogie_type_value_array(env, &type_args)
                );
            },
        );

        // Emit $UpdateField functions.
        let fields = struct_env.get_fields().collect_vec();
        for (pos, field_env) in fields.iter().enumerate() {
            let field_name = field_env.get_name().display(env.symbol_pool()).to_string();
            self.emit_function(
                &format!(
                    "$Update{}_{}(s: {}, x: {}): {}",
                    suffix,
                    field_name,
                    struct_name,
                    boogie_local_type(env, &field_env.get_type()),
                    struct_name
                ),
                || {
                    let args = fields
                        .iter()
                        .enumerate()
                        .map(|(p, f)| {
                            if p == pos {
                                "x".to_string()
                            } else {
                                format!("{}(s)", boogie_field_sel(f))
                            }
                        })
                        .join(", ");
                    emitln!(self.writer, "{}({})", struct_name, args);
                },
            );
        }

        // Emit equality
        self.emit_function_with_attr(
            "",
            &format!(
                "$IsEqual{}(s1: {}, s2: {}): bool",
                suffix, struct_name, struct_name
            ),
            || {
                if struct_has_native_equality(struct_env, self.options) {
                    emitln!(self.writer, "s1 == s2")
                } else {
                    let mut sep = "";
                    for field in &fields {
                        let sel_fun = boogie_field_sel(field);
                        let field_suffix = boogie_type_suffix(env, &field.get_type());
                        emitln!(
                            self.writer,
                            "{}$IsEqual{}({}(s1), {}(s2))",
                            sep,
                            field_suffix,
                            sel_fun,
                            sel_fun
                        );
                        sep = "&& ";
                    }
                }
            },
        );

        // Emit WriteBackToValue
        self.emit_procedure(
            &format!(
                "$WritebackToValue{}(src: $Mutation, idx: int, vdst: {}) returns (vdst': {})",
                suffix, struct_name, struct_name
            ),
            || {
                emitln!(self.writer, "if (l#$Mutation(src) == $Local(idx)) {");
                self.writer.with_indent(|| {
                    emitln!(self.writer, "vdst' := $Unbox{}($Dereference(src));", suffix);
                });
                emitln!(self.writer, "} else {");
                self.writer
                    .with_indent(|| emitln!(self.writer, "vdst' := vdst;"));
                emitln!(self.writer, "}");
            },
        );

        // Emit BorrowField/WritebackToField for each field
        for field in &fields {
            let field_name = field.get_name().display(env.symbol_pool()).to_string();
            let field_type = field.get_type();
            let field_type_suffix = boogie_type_suffix(env, &field_type);
            self.emit_procedure(
                &format!(
                    "$BorrowField{}_{}(src: $Mutation) returns (dst: $Mutation)",
                    suffix, field_name
                ),
                || {
                    emitln!(self.writer, "var p: $Path;");
                    emitln!(self.writer, "var size: int;");
                    emitln!(
                        self.writer,
                        "var x: {};",
                        boogie_local_type(env, &field_type)
                    );
                    emitln!(self.writer, "p := p#$Mutation(src);");
                    emitln!(self.writer, "size := size#$Path(p);");
                    emitln!(
                        self.writer,
                        "p := $Path(p#$Path(p)[size := {}], size + 1);",
                        field.get_offset()
                    );
                    emitln!(
                        self.writer,
                        "x := {}($Unbox{}($Dereference(src)));",
                        boogie_field_sel(field),
                        suffix
                    );
                    emitln!(
                        self.writer,
                        "dst := $Mutation(l#$Mutation(src), p, $Box{}(x));",
                        field_type_suffix
                    );
                },
            );
            self.emit_procedure(
                &format!(
                    "$WritebackToField{}_{}(src: $Mutation, dst: $Mutation) \
                       returns (dst': $Mutation)",
                    suffix, field_name
                ),
                || {
                    emitln!(
                        self.writer,
                        "var s: {};\nvar x: {};",
                        struct_name,
                        boogie_local_type(env, &field_type)
                    );
                    emitln!(self.writer, "if (l#$Mutation(dst) == l#$Mutation(src)) {");
                    self.writer.with_indent(|| {
                        emitln!(self.writer, "s := $Unbox{}($Dereference(dst));", suffix);
                        emitln!(
                            self.writer,
                            "x := $Unbox{}($Dereference(src));",
                            field_type_suffix
                        );
                        emitln!(self.writer, "s := $Update{}_{}(s, x);", suffix, field_name);
                        emitln!(
                            self.writer,
                            "dst' := $UpdateMutation(dst, $Box{}(s));",
                            suffix
                        );
                    });
                    emitln!(self.writer, "} else {");
                    self.writer
                        .with_indent(|| emitln!(self.writer, "dst' := dst;"));
                    emitln!(self.writer, "}");
                },
            );
        }

        if struct_env.is_resource() {
            // Emit memory variable.
            let memory_name =
                boogie_resource_memory_name(env, struct_env.get_qualified_id(), &None);
            emitln!(self.writer, "var {}: $Memory {};", memory_name, struct_name);

            // Emit modifies procedure.
            self.emit_procedure(
                &format!("$Modifies{}(ta: Vec $TypeValue, a: int)", suffix),
                || {
                    emitln!(self.writer, "var fresh: $Memory {};", struct_name);
                    emitln!(
                        self.writer,
                        "{} := $ResourceCopy({}, fresh, ta, a);",
                        memory_name,
                        memory_name
                    );
                },
            );

            // Emit $MoveTo/$MoveFrom procedures
            self.emit_procedure(
                &format!(
                    "$MoveTo{}(ta: Vec $TypeValue, a: int, v: {})",
                    suffix, struct_name
                ),
                || {
                    emitln!(
                        self.writer,
                        "if ($ResourceExists({}, ta, a)) {{ call $ExecFailureAbort(); return; }}",
                        memory_name,
                    );
                    emitln!(
                        self.writer,
                        "{} := $ResourceUpdate({}, ta, a, v);",
                        memory_name,
                        memory_name
                    );
                },
            );
            self.emit_procedure(
                &format!(
                    "$MoveFrom{}(ta: Vec $TypeValue, a: int) returns (res: {})",
                    suffix, struct_name
                ),
                || {
                    emitln!(
                        self.writer,
                        "if (!$ResourceExists({}, ta, a)) {{ call $ExecFailureAbort(); return; }}",
                        memory_name,
                    );
                    emitln!(
                        self.writer,
                        "res := $ResourceValue({}, ta, a);",
                        memory_name
                    );
                    emitln!(
                        self.writer,
                        "{} := $ResourceRemove({}, ta, a);",
                        memory_name,
                        memory_name
                    );
                },
            );

            // Emit $BorrowGlobal/$GetGlobal/$WritebackToGlobal procedures
            self.emit_procedure(
                &format!(
                    "$BorrowGlobal{}(a: int, ta: Vec $TypeValue) returns (dst: $Mutation)",
                    suffix,
                ),
                || {
                    emitln!(
                        self.writer,
                        "if (!$ResourceExists({}, ta, a)) {{ call $ExecFailureAbort(); return; }}",
                        memory_name
                    );
                    emitln!(self.writer,
                        "dst := $Mutation($Global(ta, a), $EmptyPath, $Box{}(contents#$Memory({})[ta, a]));",
                        suffix, memory_name);
                },
            );
            self.emit_procedure(
                &format!(
                    "$GetGlobal{}(a: int, ta: Vec $TypeValue) returns (dst: {})",
                    suffix, struct_name
                ),
                || {
                    emitln!(
                        self.writer,
                        "if (!$ResourceExists({}, ta, a)) {{ call $ExecFailureAbort(); return; }}",
                        memory_name
                    );
                    emitln!(
                        self.writer,
                        "dst := $ResourceValue({}, ta, a);",
                        memory_name
                    );
                },
            );
            self.emit_procedure(
                &format!("$WritebackToGlobal{}(src: $Mutation)", suffix),
                || {
                    emitln!(self.writer, "var l: $Location;");
                    emitln!(
                        self.writer,
                        "l := l#$Mutation(src);\n\
                         if (is#$Global(l)) {"
                    );
                    self.writer.indent();
                    emitln!(
                        self.writer,
                        "{} :=\n  $ResourceUpdate({}, ts#$Global(l), a#$Global(l),\
                            \n    $Unbox{}($Dereference(src)));",
                        memory_name,
                        memory_name,
                        suffix
                    );
                    self.writer.unindent();
                    emitln!(self.writer, "}");
                },
            );
        }

        emitln!(self.writer);
    }

    fn emit_procedure(&self, signature: &str, body_fn: impl Fn()) {
        emitln!(self.writer, "procedure {{:inline 1}} {} {{", signature);
        self.writer.indent();
        body_fn();
        self.writer.unindent();
        emitln!(self.writer, "}");
    }

    fn emit_function(&self, signature: &str, body_fn: impl Fn()) {
        self.emit_function_with_attr("{:inline} ", signature, body_fn)
    }

    fn emit_function_with_attr(&self, attr: &str, signature: &str, body_fn: impl Fn()) {
        emitln!(self.writer, "function {}{} {{", attr, signature);
        self.writer.indent();
        body_fn();
        self.writer.unindent();
        emitln!(self.writer, "}");
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
        for func_env in self.module_env.get_functions() {
            if func_env.is_native() || func_env.is_intrinsic() {
                continue;
            }
            let verification_info = verification_analysis::get_info(
                &self
                    .targets
                    .get_target(&func_env, FunctionVariant::Baseline),
            );
            for variant in self.targets.get_target_variants(&func_env) {
                if verification_info.verified && variant.is_verified()
                    || verification_info.inlined && !variant.is_verified()
                {
                    self.translate_function(variant, &self.targets.get_target(&func_env, variant));
                }
            }
        }
    }
}

impl<'env> ModuleTranslator<'env> {
    /// Translates the given function.
    fn translate_function(&self, variant: FunctionVariant, fun_target: &FunctionTarget<'_>) {
        self.generate_function_sig(variant, &fun_target);
        self.generate_function_body(variant, &fun_target);
        emitln!(self.writer);
    }

    /// Return a string for a boogie procedure header. Use inline attribute and name
    /// suffix as indicated by `entry_point`.
    fn generate_function_sig(&self, variant: FunctionVariant, fun_target: &FunctionTarget<'_>) {
        let (args, rets) = self.generate_function_args_and_returns(fun_target);

        let (suffix, attribs) = match variant {
            FunctionVariant::Baseline => ("".to_string(), "{:inline 1} ".to_string()),
            FunctionVariant::Verification(flavor) => {
                let timeout = fun_target
                    .func_env
                    .get_num_pragma(TIMEOUT_PRAGMA, || self.options.vc_timeout);

                let mut attribs = vec![format!("{{:timeLimit {}}} ", timeout)];

                if fun_target.func_env.is_num_pragma_set(SEED_PRAGMA) {
                    let seed = fun_target
                        .func_env
                        .get_num_pragma(SEED_PRAGMA, || self.options.random_seed);
                    attribs.push(format!("{{:random_seed {}}} ", seed));
                };

                if flavor == "inconsistency" {
                    attribs.push(format!(
                        "{{:msg_if_verifies \"inconsistency_detected{}\"}} ",
                        self.loc_str(&fun_target.get_loc())
                    ));
                }

                if flavor.is_empty() {
                    ("$verify".to_string(), attribs.join(""))
                } else {
                    (format!("$verify_{}", flavor), attribs.join(""))
                }
            }
        };
        self.writer.set_location(&fun_target.get_loc());
        emitln!(
            self.writer,
            "procedure {}{}{}({}) returns ({})",
            attribs,
            boogie_function_name(fun_target.func_env),
            suffix,
            args,
            rets,
        )
    }

    /// Generate boogie representation of function args and return args.
    fn generate_function_args_and_returns(
        &self,
        fun_target: &FunctionTarget<'_>,
    ) -> (String, String) {
        let args = fun_target
            .get_type_parameters()
            .iter()
            .map(|TypeParameter(s, _)| {
                format!("{}: $TypeValue", s.display(fun_target.symbol_pool()))
            })
            .chain((0..fun_target.get_parameter_count()).map(|i| {
                let ty = fun_target.get_local_type(i);
                // Boogie does not allow to assign to parameters, so we need to proxy them.
                let prefix = if self.parameter_needs_to_be_mutable(fun_target, i) {
                    "_$"
                } else {
                    "$"
                };
                format!(
                    "{}t{}: {}",
                    prefix,
                    i,
                    boogie_local_type(fun_target.global_env(), ty)
                )
            }))
            .join(", ");
        let mut_ref_count = (0..fun_target.get_parameter_count())
            .filter(|idx| fun_target.get_local_type(*idx).is_mutable_reference())
            .count();
        let rets = fun_target
            .get_return_types()
            .iter()
            .enumerate()
            .map(|(i, ref s)| {
                format!(
                    "$ret{}: {}",
                    i,
                    boogie_local_type(fun_target.global_env(), s)
                )
            })
            // Add implicit return parameters for &mut
            .chain((0..mut_ref_count).map(|i| {
                format!(
                    "$ret{}: $Mutation",
                    usize::saturating_add(fun_target.get_return_count(), i)
                )
            }))
            .join(", ");
        (args, rets)
    }

    /// Generates boogie implementation body.
    fn generate_function_body(&self, variant: FunctionVariant, fun_target: &FunctionTarget<'_>) {
        // Be sure to set back location to the whole function definition as a default.
        self.writer.set_location(&fun_target.get_loc().at_start());

        emitln!(self.writer, "{");
        self.writer.indent();

        // Generate local variable declarations. They need to appear first in boogie.
        emitln!(self.writer, "// declare local variables");
        let num_args = fun_target.get_parameter_count();
        for i in num_args..fun_target.get_local_count() {
            let local_type = fun_target.get_local_type(i);
            emitln!(
                self.writer,
                "var $t{}: {}; // {}",
                i,
                boogie_local_type(fun_target.global_env(), local_type),
                boogie_type_value(self.module_env.env, local_type)
            );
        }
        // Generate declarations for renamed parameters.
        let proxied_parameters = self.get_mutable_parameters(fun_target);
        for (idx, ty) in &proxied_parameters {
            emitln!(
                self.writer,
                "var $t{}: {};",
                idx,
                boogie_local_type(self.module_env.env, ty)
            );
        }
        // Generate declarations for modifies condition.
        fun_target.get_modify_targets().keys().for_each(|ty| {
            emitln!(
                self.writer,
                "var {}: {}",
                boogie_modifies_memory_name(fun_target.global_env(), *ty),
                "[Vec $TypeValue, int]bool;"
            );
        });

        // Declare temporaries for debug tracing and other purposes.
        let code = fun_target.get_bytecode();
        for (boogie_ty, cnt) in self.compute_needed_temps(fun_target) {
            for i in 0..cnt {
                emitln!(
                    self.writer,
                    "var {}: {};",
                    boogie_temp(fun_target.global_env(), &boogie_ty, i),
                    boogie_ty
                );
            }
        }

        // Generate memory snapshot variable declarations.
        let labels = code
            .iter()
            .filter_map(|bc| {
                use Bytecode::*;
                match bc {
                    SaveMem(_, lab, mem) => Some((lab, mem)),
                    SaveSpecVar(..) => panic!("spec var memory snapshots NYI"),
                    _ => None,
                }
            })
            .collect::<BTreeMap<_, _>>();
        for (lab, mem) in labels {
            let name = boogie_resource_memory_name(self.module_env.env, *mem, &Some(*lab));
            emitln!(
                self.writer,
                "var {}: $Memory {};",
                name,
                boogie_struct_name(
                    &self
                        .module_env
                        .env
                        .get_module(mem.module_id)
                        .into_struct(mem.id)
                )
            );
        }

        // Initialize renamed parameters.
        for (idx, _) in proxied_parameters {
            emitln!(self.writer, "$t{} := _$t{};", idx, idx);
        }

        // Initial assumptions
        if variant.is_verified() {
            self.translate_verify_entry_assumptions(fun_target);
        }

        // Generate bytecode
        emitln!(self.writer, "\n// bytecode translation starts here");
        let mut last_tracked_loc = None;
        for bytecode in code.iter() {
            self.translate_bytecode(fun_target, &mut last_tracked_loc, bytecode);
        }

        self.writer.unindent();
        emitln!(self.writer, "}");
    }

    fn get_mutable_parameters(&self, fun_target: &FunctionTarget<'_>) -> Vec<(TempIndex, Type)> {
        (0..fun_target.get_parameter_count())
            .filter_map(|i| {
                if self.parameter_needs_to_be_mutable(fun_target, i) {
                    Some((i, fun_target.get_local_type(i).clone()))
                } else {
                    None
                }
            })
            .collect_vec()
    }

    /// Determines whether the parameter of a function needs to be mutable.
    /// Boogie does not allow to assign to procedure parameters. In some cases
    /// (e.g. for memory instrumentation, but also as a result of copy propagation),
    /// we may need to assign to parameters.
    fn parameter_needs_to_be_mutable(
        &self,
        _fun_target: &FunctionTarget<'_>,
        _idx: TempIndex,
    ) -> bool {
        // For now, we just always say true. This could be optimized because the actual (known
        // so far) sources for mutability are parameters which are used in WriteBack(LocalRoot(p))
        // position.
        true
    }

    fn translate_verify_entry_assumptions(&self, fun_target: &FunctionTarget<'_>) {
        emitln!(self.writer, "\n// verification entrypoint assumptions");

        // Prelude initialization
        emitln!(self.writer, "call $InitVerification();");

        // Assume reference parameters to be based on the Param(i) Location, ensuring
        // they are disjoint from all other references. This prevents aliasing and is justified as
        // follows:
        // - for mutual references, by their exclusive access in Move.
        // - for immutable references, by that mutation is not possible, and they are equivalent
        //   to some given but arbitrary value.
        for i in 0..fun_target.get_parameter_count() {
            let ty = fun_target.get_local_type(i);
            if ty.is_reference() {
                emitln!(self.writer, "assume l#$Mutation($t{}) == $Param({});", i, i);
                emitln!(self.writer, "assume size#$Path(p#$Mutation($t{})) == 0;", i);
            }
        }

        // Initialize modify permissions.
        self.initialize_modifies_permissions(fun_target);
    }

    /// Initializes modifies permissions.
    fn initialize_modifies_permissions(&self, fun_target: &FunctionTarget<'_>) {
        let env = fun_target.global_env();
        for (ty, targets) in fun_target.get_modify_targets() {
            emit!(
                self.writer,
                "{} := {}",
                boogie_modifies_memory_name(fun_target.global_env(), *ty),
                "$ConstMemoryDomain(false)"
            );
            for target in targets {
                let node_id = target.node_id();
                let args = target.call_args();
                let rty = &env.get_node_instantiation(node_id)[0];
                let (_, _, targs) = rty.require_struct();
                let type_args = boogie_type_value_array(env, targs);
                emit!(self.writer, "[{}, a#$Address(", type_args);
                self.spec_translator.translate(&args[0]);
                emit!(self.writer, ") := true]");
            }
            emitln!(self.writer, ";");
        }
    }

    /// Translates one bytecode instruction.
    fn translate_bytecode(
        &'env self,
        fun_target: &FunctionTarget<'_>,
        last_tracked_loc: &mut Option<(Loc, LineIndex)>,
        bytecode: &Bytecode,
    ) {
        use Bytecode::*;
        // Set location of this code in the CodeWriter.
        let attr_id = bytecode.get_attr_id();
        let loc = fun_target.get_bytecode_loc(attr_id);
        self.writer.set_location(&loc);

        // Print location.
        emitln!(
            self.writer,
            "// {} {}",
            bytecode.display(fun_target, &BTreeMap::default()),
            loc.display(self.module_env.env)
        );

        // Print debug comments.
        if let Some(comment) = fun_target.get_debug_comment(attr_id) {
            emitln!(self.writer, "// {}", comment);
        }

        // Track location for execution traces.
        if matches!(bytecode, Call(_, _, Operation::TraceAbort, ..)) {
            // Ensure that aborts always has the precise location instead of the
            // line-approximated one
            *last_tracked_loc = None;
        }
        self.track_loc(fun_target, last_tracked_loc, &loc);

        // Helper function to get a a string for a local
        let str_local = |idx: usize| format!("$t{}", idx);

        // Translate the bytecode instruction.
        match bytecode {
            SpecBlock(..) => panic!("deprecated"),
            SaveMem(_, label, mem) => {
                let snapshot =
                    boogie_resource_memory_name(self.module_env.env, *mem, &Some(*label));
                let current = boogie_resource_memory_name(self.module_env.env, *mem, &None);
                emitln!(self.writer, "{} := {};", snapshot, current);
            }
            SaveSpecVar(_, _label, _var) => {
                panic!("spec var snapshot NYI")
            }
            Prop(id, kind, exp) => match kind {
                PropKind::Assert => {
                    emit!(self.writer, "assert ");
                    let info = fun_target
                        .get_vc_info(*id)
                        .map(|s| s.as_str())
                        .unwrap_or("unknown assertion failed");
                    emit!(
                        self.writer,
                        "{{:msg \"assert_failed{}: {}\"}}\n  ",
                        self.loc_str(&loc),
                        info
                    );
                    self.spec_translator.translate_unboxed(exp);
                    emitln!(self.writer, ";");
                }
                PropKind::Assume => {
                    emit!(self.writer, "assume ");
                    self.spec_translator.translate_unboxed(exp);
                    emitln!(self.writer, ";");
                }
                PropKind::Modifies => {
                    let ty = self.module_env.env.get_node_type(exp.node_id());
                    let (mid, sid, type_args) = ty.require_struct();
                    let struct_env = fun_target.global_env().get_module(mid).into_struct(sid);
                    let suffix = boogie_type_suffix_for_struct(&struct_env);
                    let boogie_type_args = boogie_type_value_array(self.module_env.env, type_args);
                    emit!(
                        self.writer,
                        "call $Modifies{}({}, ",
                        suffix,
                        boogie_type_args
                    );
                    self.spec_translator.translate_unboxed(&exp.call_args()[0]);
                    emitln!(self.writer, ");");
                }
            },
            Label(_, label) => {
                self.writer.unindent();
                emitln!(self.writer, "L{}:", label.as_usize());
                /*
                // TODO: revisit whether we can express what is needed here on bytecode level
                let annotated_func_target = self.targets.get_annotated_target(fun_target.func_env);
                let loop_annotation = annotated_func_target
                    .get_annotations()
                    .get::<LoopAnnotation>()
                    .expect("loop annotation");
                if loop_annotation.loop_targets.contains_key(label) {
                    let targets = &loop_annotation.loop_targets[label];
                    for idx in 0..fun_target.get_local_count() {
                        if let Some(ref_proxy_idx) = fun_target.get_ref_proxy_index(idx) {
                            if targets.contains(ref_proxy_idx) {
                                let ref_proxy_var_name = str_local(*ref_proxy_idx);
                                let proxy_idx = fun_target.get_proxy_index(idx).unwrap();
                                emitln!(
                                    self.writer,
                                    "assume l#$Mutation({}) == $Local({}) && p#$Mutation({}) == $EmptyPath;",
                                    ref_proxy_var_name,
                                    proxy_idx,
                                    ref_proxy_var_name);
                            }
                        }
                    }
                }
                 */
                self.writer.indent();
            }
            Jump(_, target) => emitln!(self.writer, "goto L{};", target.as_usize()),
            Branch(_, then_target, else_target, idx) => emitln!(
                self.writer,
                "if ({}) {{ goto L{}; }} else {{ goto L{}; }}",
                str_local(*idx),
                then_target.as_usize(),
                else_target.as_usize(),
            ),
            Assign(_, dest, src, _) => {
                emitln!(self.writer, "{} := {};", str_local(*dest), str_local(*src));
            }
            Ret(_, rets) => {
                for (i, r) in rets.iter().enumerate() {
                    emitln!(self.writer, "$ret{} := {};", i, str_local(*r));
                }
                // Also assign input to output $mut parameters
                let mut ret_idx = rets.len();
                for i in 0..fun_target.get_parameter_count() {
                    if fun_target.get_local_type(i).is_mutable_reference() {
                        emitln!(self.writer, "$ret{} := {};", ret_idx, str_local(i));
                        ret_idx = usize::saturating_add(ret_idx, 1);
                    }
                }
                emitln!(self.writer, "return;");
            }
            Load(_, idx, c) => {
                let value = match c {
                    Constant::Bool(true) => "true".to_string(),
                    Constant::Bool(false) => "false".to_string(),
                    Constant::U8(num) => num.to_string(),
                    Constant::U64(num) => num.to_string(),
                    Constant::U128(num) => num.to_string(),
                    Constant::Address(val) => val.to_string(),
                    Constant::ByteArray(val) => boogie_byte_blob(self.options, val),
                };
                emitln!(self.writer, "{} := {};", str_local(*idx), value);
            }
            Call(_, dests, oper, srcs, aa) => {
                use Operation::*;
                match oper {
                    FreezeRef => unreachable!(),
                    UnpackRef | UnpackRefDeep | PackRef | PackRefDeep => {
                        // No effect
                    }
                    WriteBack(dest, edge) => {
                        use BorrowNode::*;
                        let src = srcs[0];
                        match dest {
                            GlobalRoot(memory) => {
                                let func = match edge {
                                    BorrowEdge::Weak => {
                                        panic!("unexpected weak edge")
                                    }
                                    BorrowEdge::Strong(StrongEdge::Direct) => "WritebackToGlobal",
                                    _ => {
                                        panic!("Strong global writeback cannot have field")
                                    }
                                };
                                let suffix = boogie_type_suffix_for_struct(
                                    &fun_target.global_env().get_struct_qid(*memory),
                                );
                                emitln!(
                                    self.writer,
                                    "call ${}{}({});",
                                    func,
                                    suffix,
                                    str_local(src),
                                );
                            }
                            LocalRoot(idx) => {
                                let func = match edge {
                                    BorrowEdge::Weak => {
                                        // DNS
                                        panic!("unexpected weak edge")
                                    }
                                    BorrowEdge::Strong(StrongEdge::Direct) => "WritebackToValue",
                                    _ => {
                                        panic!("Strong local writeback cannot have field")
                                    }
                                };
                                let suffix = boogie_type_suffix(
                                    fun_target.global_env(),
                                    fun_target.get_local_type(*idx),
                                );
                                emitln!(
                                    self.writer,
                                    "call {} := ${}{}({}, {}, {});",
                                    str_local(*idx),
                                    func,
                                    suffix,
                                    str_local(src),
                                    idx,
                                    str_local(*idx)
                                );
                            }
                            Reference(idx) => {
                                let func = match edge {
                                    BorrowEdge::Weak => {
                                        panic!("unexpected weak edge")
                                    }
                                    BorrowEdge::Strong(StrongEdge::Direct) => {
                                        "WritebackToReference".to_string()
                                    }
                                    BorrowEdge::Strong(StrongEdge::Field(memory, offset)) => {
                                        let struct_env =
                                            fun_target.global_env().get_struct_qid(*memory);
                                        let field_env = struct_env.get_field_by_offset(*offset);
                                        format!(
                                            "WritebackToField{}_{}",
                                            boogie_type_suffix_for_struct(&struct_env),
                                            field_env.get_name().display(fun_target.symbol_pool())
                                        )
                                    }
                                    BorrowEdge::Strong(StrongEdge::FieldUnknown) => {
                                        "WritebackToVec".to_string()
                                    }
                                };
                                emitln!(
                                    self.writer,
                                    "call {} := ${}({}, {});",
                                    str_local(*idx),
                                    func,
                                    str_local(src),
                                    str_local(*idx),
                                );
                            }
                        }
                    }
                    Splice(map) => {
                        let src = srcs[0];
                        assert!(!map.is_empty());
                        emitln!(
                            self.writer,
                            "call {} := $Splice{}({}, {});",
                            str_local(src),
                            map.len(),
                            map.iter()
                                .map(|(pos, idx)| format!("{}, {}", pos, str_local(*idx)))
                                .join(", "),
                            str_local(src)
                        );
                    }
                    BorrowLoc => {
                        let src = srcs[0];
                        let dest = dests[0];
                        let suffix = boogie_type_suffix(
                            fun_target.global_env(),
                            fun_target.get_local_type(src),
                        );
                        emitln!(
                            self.writer,
                            "call {} := $BorrowLoc({}, $Box{}({}));",
                            str_local(dest),
                            src,
                            suffix,
                            str_local(src)
                        );
                    }
                    ReadRef => {
                        let src = srcs[0];
                        let dest = dests[0];
                        let suffix = boogie_type_suffix(
                            fun_target.global_env(),
                            fun_target.get_local_type(dest),
                        );
                        emitln!(
                            self.writer,
                            "{} := $Unbox{}($Dereference({}));",
                            str_local(dest),
                            suffix,
                            str_local(src)
                        );
                    }
                    WriteRef => {
                        let reference = srcs[0];
                        let value = srcs[1];
                        let suffix = boogie_type_suffix(
                            fun_target.global_env(),
                            fun_target.get_local_type(value),
                        );
                        emitln!(
                            self.writer,
                            "{} := $UpdateMutation({}, $Box{}({}));",
                            str_local(reference),
                            str_local(reference),
                            suffix,
                            str_local(value),
                        );
                    }
                    Function(mid, fid, type_actuals) => {
                        let callee_env = self.module_env.env.get_module(*mid).into_function(*fid);
                        let callee_target = self
                            .targets
                            .get_target(&callee_env, FunctionVariant::Baseline);

                        let args_str = std::iter::once(boogie_type_values(
                            fun_target.func_env.module_env.env,
                            type_actuals,
                        ))
                        .chain(srcs.iter().enumerate().map(|(pos, arg_idx)| {
                            let param_type = callee_target.get_local_type(pos);
                            let arg_type = fun_target.get_local_type(*arg_idx);
                            if param_type.is_type_parameter() {
                                // If the function parameter has a type parameter type, we
                                // need to box it before passing in.
                                format!(
                                    "$Box{}({})",
                                    boogie_type_suffix(fun_target.global_env(), &arg_type),
                                    str_local(*arg_idx)
                                )
                            } else {
                                str_local(*arg_idx)
                            }
                        }))
                        .filter(|s| !s.is_empty())
                        .join(", ");

                        let mut unbox_assigns = vec![];
                        let dest_str = dests
                            .iter()
                            .enumerate()
                            .map(|(pos, idx)| {
                                let return_type = callee_target.get_return_type(pos);
                                let arg_type = fun_target.get_local_type(*idx);
                                if return_type.is_type_parameter() {
                                    // Need to unbox after call
                                    let value_temp = boogie_temp(
                                        fun_target.global_env(),
                                        &boogie_local_type(fun_target.global_env(), return_type),
                                        unbox_assigns.len(),
                                    );
                                    let suffix =
                                        boogie_type_suffix(fun_target.global_env(), arg_type);
                                    unbox_assigns.push(format!(
                                        "{} := $Unbox{}({});",
                                        str_local(*idx),
                                        suffix,
                                        value_temp
                                    ));
                                    value_temp
                                } else {
                                    str_local(*idx)
                                }
                            })
                            // Add implict dest returns for &mut srcs:
                            //  f(x) --> x := f(x)  with t(x) = &mut_
                            .chain(
                                srcs.iter()
                                    .filter(|idx| {
                                        fun_target.get_local_type(**idx).is_mutable_reference()
                                    })
                                    .cloned()
                                    .map(str_local),
                            )
                            .join(",");

                        if dest_str.is_empty() {
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
                            for assign in unbox_assigns {
                                emitln!(self.writer, &assign)
                            }
                        }
                        // Clear the last track location after function call, as the call inserted
                        // location tracks before it returns.
                        *last_tracked_loc = None;
                    }
                    Pack(mid, sid, type_actuals) => {
                        let struct_env = fun_target
                            .func_env
                            .module_env
                            .env
                            .get_module(*mid)
                            .into_struct(*sid);
                        let args = struct_env
                            .get_fields()
                            .enumerate()
                            .map(|(i, field_env)| {
                                if field_env.get_type().is_type_parameter() {
                                    let suffix = boogie_type_suffix(
                                        fun_target.global_env(),
                                        &field_env.get_type().instantiate(type_actuals),
                                    );
                                    format!("$Box{}({})", suffix, str_local(srcs[i]))
                                } else {
                                    str_local(srcs[i])
                                }
                            })
                            .join(", ");
                        emitln!(
                            self.writer,
                            "{} := {}({});",
                            str_local(dests[0]),
                            boogie_struct_name(&struct_env),
                            args
                        );
                    }
                    Unpack(mid, sid, _type_actuals) => {
                        let struct_env = fun_target
                            .func_env
                            .module_env
                            .env
                            .get_module(*mid)
                            .into_struct(*sid);
                        for (i, field_env) in struct_env.get_fields().enumerate() {
                            let mut field_sel =
                                format!("{}({})", boogie_field_sel(&field_env), str_local(srcs[0]));
                            if field_env.get_type().is_type_parameter() {
                                // Need to unbox the result of field selection
                                let suffix = boogie_type_suffix(
                                    fun_target.global_env(),
                                    fun_target.get_local_type(dests[i]),
                                );
                                field_sel = format!("$Unbox{}({})", suffix, field_sel);
                            }
                            emitln!(self.writer, "{} := {};", str_local(dests[i]), field_sel,);
                        }
                    }
                    BorrowField(mid, sid, _, field_offset) => {
                        let src = srcs[0];
                        let dest = dests[0];
                        let struct_env = fun_target.global_env().get_module(*mid).into_struct(*sid);
                        let suffix = boogie_type_suffix_for_struct(&struct_env);
                        let field_env = &struct_env.get_field_by_offset(*field_offset);
                        emitln!(
                            self.writer,
                            "call {} := $BorrowField{}_{}({});",
                            str_local(dest),
                            suffix,
                            field_env.get_name().display(fun_target.symbol_pool()),
                            str_local(src),
                        );
                    }
                    GetField(mid, sid, _, field_offset) => {
                        let src = srcs[0];
                        let dest = dests[0];
                        let struct_env = fun_target.global_env().get_module(*mid).into_struct(*sid);
                        let suffix = boogie_type_suffix_for_struct(&struct_env);
                        let field_env = &struct_env.get_field_by_offset(*field_offset);
                        let src_ty = fun_target.get_local_type(src);
                        let struct_arg = if src_ty.is_reference() {
                            format!("$Unbox{}($Dereference({}))", suffix, str_local(src))
                        } else {
                            str_local(src)
                        };
                        let mut field_val =
                            format!("{}({})", boogie_field_sel(field_env), struct_arg);
                        if field_env.get_type().is_type_parameter() {
                            // Need to unbox for type instantiation
                            field_val = format!(
                                "$Unbox{}({})",
                                boogie_type_suffix(
                                    fun_target.global_env(),
                                    fun_target.get_local_type(dest)
                                ),
                                field_val
                            );
                        }
                        emitln!(self.writer, "{} := {};", str_local(dest), field_val);
                    }
                    Exists(mid, sid, type_actuals) => {
                        let addr = srcs[0];
                        let dest = dests[0];
                        let type_args = boogie_type_value_array(self.module_env.env, type_actuals);
                        let memory = boogie_resource_memory_name(
                            self.module_env.env,
                            mid.qualified(*sid),
                            &None,
                        );
                        emitln!(
                            self.writer,
                            "{} := $ResourceExists({}, {}, {});",
                            str_local(dest),
                            memory,
                            type_args,
                            str_local(addr),
                        );
                    }
                    BorrowGlobal(mid, sid, type_actuals) => {
                        let addr = srcs[0];
                        let dest = dests[0];
                        let type_args = boogie_type_value_array(self.module_env.env, type_actuals);
                        let addr_name = str_local(addr);
                        let suffix = boogie_type_suffix_for_struct(
                            &fun_target.global_env().get_module(*mid).into_struct(*sid),
                        );
                        emitln!(
                            self.writer,
                            "call {} := $BorrowGlobal{}({}, {});",
                            str_local(dest),
                            suffix,
                            addr_name,
                            type_args,
                        );
                    }
                    GetGlobal(mid, sid, type_actuals) => {
                        let addr = srcs[0];
                        let dest = dests[0];
                        let type_args = boogie_type_value_array(self.module_env.env, type_actuals);
                        let suffix = boogie_type_suffix_for_struct(
                            &fun_target.global_env().get_module(*mid).into_struct(*sid),
                        );
                        emitln!(
                            self.writer,
                            "call {} := $GetGlobal{}({}, {});",
                            str_local(dest),
                            suffix,
                            str_local(addr),
                            type_args,
                        );
                    }
                    MoveTo(mid, sid, type_actuals) => {
                        let value = srcs[0];
                        let signer = srcs[1];
                        let type_args = boogie_type_value_array(self.module_env.env, type_actuals);
                        let signer_name = str_local(signer);
                        let suffix = boogie_type_suffix_for_struct(
                            &fun_target.global_env().get_module(*mid).into_struct(*sid),
                        );
                        emitln!(
                            self.writer,
                            "call $MoveTo{}({}, {}, {});",
                            suffix,
                            type_args,
                            signer_name,
                            str_local(value),
                        );
                    }
                    MoveFrom(mid, sid, type_actuals) => {
                        let src = srcs[0];
                        let dest = dests[0];
                        let type_args = boogie_type_value_array(self.module_env.env, type_actuals);
                        let src_name = str_local(src);
                        let suffix = boogie_type_suffix_for_struct(
                            &fun_target.global_env().get_module(*mid).into_struct(*sid),
                        );
                        emitln!(
                            self.writer,
                            "call {} := $MoveFrom{}({}, {});",
                            str_local(dest),
                            suffix,
                            type_args,
                            src_name,
                        );
                    }
                    Havoc(HavocKind::Value) | Havoc(HavocKind::MutationAll) => {
                        let temp_str = str_local(srcs[0]);
                        emitln!(self.writer, "havoc {};", temp_str);
                        // Insert a WellFormed check
                        let ty = fun_target.get_local_type(srcs[0]);
                        let check = boogie_well_formed_check(self.module_env.env, &temp_str, ty);
                        if !check.is_empty() {
                            emitln!(self.writer, &check);
                        }
                    }
                    Havoc(HavocKind::MutationValue) => {
                        let temp_str = str_local(srcs[0]);
                        emitln!(
                            self.writer,
                            "call {} := $HavocMutation({});",
                            temp_str,
                            temp_str
                        );
                        // Insert a WellFormed check
                        let ty = fun_target.get_local_type(srcs[0]);
                        let check = boogie_well_formed_check(self.module_env.env, &temp_str, ty);
                        if !check.is_empty() {
                            emitln!(self.writer, &check);
                        }
                    }
                    Stop => {
                        // the two statements combined terminate any execution trace that reaches it
                        emitln!(self.writer, "assume false;");
                        emitln!(self.writer, "return;");
                    }
                    CastU8 => {
                        let src = srcs[0];
                        let dest = dests[0];
                        emitln!(
                            self.writer,
                            "call {} := $CastU8({});",
                            str_local(dest),
                            str_local(src)
                        );
                    }
                    CastU64 => {
                        let src = srcs[0];
                        let dest = dests[0];
                        emitln!(
                            self.writer,
                            "call {} := $CastU64({});",
                            str_local(dest),
                            str_local(src)
                        );
                    }
                    CastU128 => {
                        let src = srcs[0];
                        let dest = dests[0];
                        emitln!(
                            self.writer,
                            "call {} := $CastU128({});",
                            str_local(dest),
                            str_local(src)
                        );
                    }
                    Not => {
                        let src = srcs[0];
                        let dest = dests[0];
                        emitln!(
                            self.writer,
                            "call {} := $Not({});",
                            str_local(dest),
                            str_local(src)
                        );
                    }
                    Add => {
                        let dest = dests[0];
                        let op1 = srcs[0];
                        let op2 = srcs[1];
                        let unchecked = if fun_target
                            .is_pragma_true(ADDITION_OVERFLOW_UNCHECKED_PRAGMA, || false)
                        {
                            "_unchecked"
                        } else {
                            ""
                        };
                        let add_type = match fun_target.get_local_type(dest) {
                            Type::Primitive(PrimitiveType::U8) => "U8".to_string(),
                            Type::Primitive(PrimitiveType::U64) => format!("U64{}", unchecked),
                            Type::Primitive(PrimitiveType::U128) => format!("U128{}", unchecked),
                            _ => unreachable!(),
                        };
                        emitln!(
                            self.writer,
                            "call {} := $Add{}({}, {});",
                            str_local(dest),
                            add_type,
                            str_local(op1),
                            str_local(op2)
                        );
                    }
                    Sub => {
                        let dest = dests[0];
                        let op1 = srcs[0];
                        let op2 = srcs[1];
                        emitln!(
                            self.writer,
                            "call {} := $Sub({}, {});",
                            str_local(dest),
                            str_local(op1),
                            str_local(op2)
                        );
                    }
                    Mul => {
                        let dest = dests[0];
                        let op1 = srcs[0];
                        let op2 = srcs[1];
                        let mul_type = match fun_target.get_local_type(dest) {
                            Type::Primitive(PrimitiveType::U8) => "U8",
                            Type::Primitive(PrimitiveType::U64) => "U64",
                            Type::Primitive(PrimitiveType::U128) => "U128",
                            _ => unreachable!(),
                        };
                        emitln!(
                            self.writer,
                            "call {} := $Mul{}({}, {});",
                            str_local(dest),
                            mul_type,
                            str_local(op1),
                            str_local(op2)
                        );
                    }
                    Div => {
                        let dest = dests[0];
                        let op1 = srcs[0];
                        let op2 = srcs[1];
                        emitln!(
                            self.writer,
                            "call {} := $Div({}, {});",
                            str_local(dest),
                            str_local(op1),
                            str_local(op2)
                        );
                    }
                    Mod => {
                        let dest = dests[0];
                        let op1 = srcs[0];
                        let op2 = srcs[1];
                        emitln!(
                            self.writer,
                            "call {} := $Mod({}, {});",
                            str_local(dest),
                            str_local(op1),
                            str_local(op2)
                        );
                    }
                    Shl => {
                        let dest = dests[0];
                        let op1 = srcs[0];
                        let op2 = srcs[1];
                        emitln!(
                            self.writer,
                            "call {} := $Shl({}, {});",
                            str_local(dest),
                            str_local(op1),
                            str_local(op2)
                        );
                    }
                    Shr => {
                        let dest = dests[0];
                        let op1 = srcs[0];
                        let op2 = srcs[1];
                        emitln!(
                            self.writer,
                            "call {} := $Shr({}, {});",
                            str_local(dest),
                            str_local(op1),
                            str_local(op2)
                        );
                    }
                    Lt => {
                        let dest = dests[0];
                        let op1 = srcs[0];
                        let op2 = srcs[1];
                        emitln!(
                            self.writer,
                            "call {} := $Lt({}, {});",
                            str_local(dest),
                            str_local(op1),
                            str_local(op2)
                        );
                    }
                    Gt => {
                        let dest = dests[0];
                        let op1 = srcs[0];
                        let op2 = srcs[1];
                        emitln!(
                            self.writer,
                            "call {} := $Gt({}, {});",
                            str_local(dest),
                            str_local(op1),
                            str_local(op2)
                        );
                    }
                    Le => {
                        let dest = dests[0];
                        let op1 = srcs[0];
                        let op2 = srcs[1];
                        emitln!(
                            self.writer,
                            "call {} := $Le({}, {});",
                            str_local(dest),
                            str_local(op1),
                            str_local(op2)
                        );
                    }
                    Ge => {
                        let dest = dests[0];
                        let op1 = srcs[0];
                        let op2 = srcs[1];
                        emitln!(
                            self.writer,
                            "call {} := $Ge({}, {});",
                            str_local(dest),
                            str_local(op1),
                            str_local(op2)
                        );
                    }
                    Or => {
                        let dest = dests[0];
                        let op1 = srcs[0];
                        let op2 = srcs[1];
                        emitln!(
                            self.writer,
                            "call {} := $Or({}, {});",
                            str_local(dest),
                            str_local(op1),
                            str_local(op2)
                        );
                    }
                    And => {
                        let dest = dests[0];
                        let op1 = srcs[0];
                        let op2 = srcs[1];
                        emitln!(
                            self.writer,
                            "call {} := $And({}, {});",
                            str_local(dest),
                            str_local(op1),
                            str_local(op2)
                        );
                    }
                    Eq | Neq => {
                        let dest = dests[0];
                        let op1 = srcs[0];
                        let op2 = srcs[1];
                        let oper = boogie_equality_for_type(
                            fun_target.global_env(),
                            oper == &Eq,
                            fun_target.get_local_type(op1),
                        );
                        emitln!(
                            self.writer,
                            "{} := {}({}, {});",
                            str_local(dest),
                            oper,
                            str_local(op1),
                            str_local(op2)
                        );
                    }
                    BitOr | BitAnd | Xor => {
                        emitln!(
                            self.writer,
                            "// bit operation not supported: {:?}\nassert false;",
                            bytecode
                        );
                    }
                    Destroy => {}
                    TraceLocal(idx) => {
                        self.track_local(fun_target, *idx, srcs[0]);
                    }
                    TraceReturn(i) => {
                        self.track_return(fun_target, *i, srcs[0]);
                    }
                    TraceAbort => self.track_abort(fun_target, &str_local(srcs[0])),
                    TraceExp(node_id) => self.track_exp(fun_target, *node_id, srcs[0]),
                    EmitEvent => {
                        let msg = srcs[0];
                        let handle = srcs[1];
                        let translate_local = |idx: usize| {
                            let ty = fun_target.get_local_type(idx);
                            if ty.is_mutable_reference() {
                                let suffix = boogie_type_suffix(
                                    fun_target.global_env(),
                                    ty.skip_reference(),
                                );
                                format!("$Unbox{}($Dereference({}))", suffix, str_local(idx))
                            } else {
                                str_local(idx)
                            }
                        };
                        emit!(
                            self.writer,
                            "$es := ${}ExtendEventStore($es, ",
                            if srcs.len() > 2 { "Cond" } else { "" }
                        );
                        let suffix = boogie_type_suffix(
                            fun_target.global_env(),
                            fun_target.get_local_type(msg),
                        );
                        emit!(
                            self.writer,
                            "{}, $Box{}({})",
                            translate_local(handle),
                            suffix,
                            str_local(msg)
                        );
                        if srcs.len() > 2 {
                            emit!(self.writer, ", {}", str_local(srcs[2]));
                        }
                        emitln!(self.writer, ");");
                    }
                    EventStoreDiverge => {
                        emitln!(self.writer, "call $es := $EventStore__diverge($es);");
                    }
                }
                if let Some(AbortAction(target, code)) = aa {
                    emitln!(self.writer, "if ($abort_flag) {");
                    self.writer.indent();
                    let code_str = str_local(*code);
                    emitln!(self.writer, "{} := $abort_code;", code_str);
                    self.track_abort(fun_target, &code_str);
                    emitln!(self.writer, "goto L{};", target.as_usize());
                    self.writer.unindent();
                    emitln!(self.writer, "}");
                }
            }
            Abort(_, src) => {
                emitln!(self.writer, "$abort_code := {};", str_local(*src));
                emitln!(self.writer, "$abort_flag := true;");
                emitln!(self.writer, "return;")
            }
            Nop(..) => {}
        }
        emitln!(self.writer);
    }

    /// Track location for execution trace, avoiding to track the same line multiple times.
    fn track_loc(
        &self,
        _fun_target: &FunctionTarget<'_>,
        last_tracked_loc: &mut Option<(Loc, LineIndex)>,
        loc: &Loc,
    ) {
        if let Some(l) = self.module_env.env.get_location(loc) {
            if let Some((last_loc, last_line)) = last_tracked_loc {
                if *last_line == l.line {
                    // This line already tracked.
                    return;
                }
                *last_loc = loc.clone();
                *last_line = l.line;
            } else {
                *last_tracked_loc = Some((loc.clone(), l.line));
            }
            emitln!(
                self.writer,
                "assume {{:print \"$at{}\"}} true;",
                self.loc_str(&loc)
            );
        }
    }

    fn track_abort(&self, fun_target: &FunctionTarget<'_>, code_var: &str) {
        emitln!(self.writer, &boogie_debug_track_abort(fun_target, code_var));
    }

    /// Generates an update of the debug information about temporary.
    fn track_local(&self, fun_target: &FunctionTarget<'_>, origin_idx: TempIndex, idx: TempIndex) {
        emitln!(
            self.writer,
            &boogie_debug_track_local(fun_target, origin_idx, idx)
        );
    }

    /// Generates an update of the debug information about the return value at given location.
    fn track_return(&self, fun_target: &FunctionTarget<'_>, return_idx: usize, idx: TempIndex) {
        emitln!(
            self.writer,
            &boogie_debug_track_return(fun_target, return_idx, idx)
        );
    }

    fn track_exp(&self, _fun_target: &FunctionTarget<'_>, node_id: NodeId, tmp: TempIndex) {
        emitln!(
            self.writer,
            "assume {{:print \"$track_exp({}):\", $t{}}} true;",
            node_id.as_usize(),
            tmp,
        );
    }

    fn loc_str(&self, loc: &Loc) -> String {
        let file_idx = self.module_env.env.file_id_to_idx(loc.file_id());
        format!("({},{},{})", file_idx, loc.span().start(), loc.span().end())
    }

    /// Compute temporaries needed for the compilation of given function.
    fn compute_needed_temps(&self, fun_target: &FunctionTarget<'_>) -> BTreeMap<String, usize> {
        use Bytecode::*;
        use Operation::*;
        let env = self.module_env.env;
        let mut res: BTreeMap<String, usize> = BTreeMap::new();
        let mut need = |boogie_ty: String, n: usize| {
            let cnt = res.entry(boogie_ty).or_default();
            *cnt = (*cnt).max(n);
        };
        for bc in &fun_target.data.code {
            if let Call(_, _, oper, ..) = bc {
                match oper {
                    TraceExp(id) => need(boogie_local_type(env, &env.get_node_type(*id)), 1),
                    TraceReturn(idx) => need(
                        boogie_local_type(env, fun_target.get_return_type(*idx).skip_reference()),
                        1,
                    ),
                    TraceLocal(idx) => need(
                        boogie_local_type(env, fun_target.get_local_type(*idx).skip_reference()),
                        1,
                    ),
                    Function(mid, fid, _) => {
                        // We need as many temporaries as there are output parameters which
                        // are of generic type, to handle unboxing after calls to such
                        // generic functions.
                        let fun_env = env.get_module(*mid).into_function(*fid);
                        let num_of_generics = fun_env
                            .get_return_types()
                            .iter()
                            .filter(|ty| ty.skip_reference().is_type_parameter())
                            .count();
                        need(
                            boogie_local_type(env, &Type::TypeParameter(0)),
                            num_of_generics,
                        );
                    }
                    _ => {}
                }
            }
        }
        res
    }
}

fn struct_has_native_equality(struct_env: &StructEnv<'_>, options: &BoogieOptions) -> bool {
    if options.native_equality {
        // Everything has native equality
    }
    for field in struct_env.get_fields() {
        match field.get_type() {
            Type::Vector(..) => {
                return false;
            }
            Type::Struct(mid, sid, _) => {
                if !struct_has_native_equality(
                    &struct_env.module_env.env.get_struct_qid(mid.qualified(sid)),
                    options,
                ) {
                    return false;
                }
            }
            Type::TypeParameter(..) => {
                // Type parameter can be instantiated with something containing a vec
                return false;
            }
            _ => {}
        }
    }
    false
}
