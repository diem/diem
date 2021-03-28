// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module translates the bytecode of a module to Boogie code.

use std::collections::BTreeMap;

use itertools::Itertools;
#[allow(unused_imports)]
use log::{debug, info, log, warn, Level};

use bytecode::{
    function_target::FunctionTarget,
    function_target_pipeline::FunctionTargetsHolder,
    stackless_bytecode::{BorrowEdge, BorrowNode, Bytecode, Constant, Operation, StrongEdge},
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
        boogie_debug_track_return, boogie_field_name, boogie_function_name, boogie_local_type,
        boogie_modifies_memory_name, boogie_resource_memory_name, boogie_struct_name,
        boogie_type_value, boogie_type_value_array, boogie_type_values, boogie_well_formed_check,
    },
    spec_translator::SpecTranslator,
};
use boogie_backend_v2::options::BoogieOptions;
use bytecode::{
    function_target_pipeline::FunctionVariant,
    stackless_bytecode::{AbortAction, PropKind},
};
use codespan::LineIndex;
use move_model::{
    ast::TempIndex,
    model::{Loc, NodeId},
};

pub struct BoogieTranslator<'env> {
    env: &'env GlobalEnv,
    options: &'env BoogieOptions,
    writer: &'env CodeWriter,
    targets: &'env FunctionTargetsHolder,
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
            // Set the location to internal so we don't see locations of pack/unpack
            // in execution traces.
            self.writer
                .set_location(&self.module_env.env.internal_loc());
            self.translate_struct_type(&struct_env);
        }
    }

    /// Translates the given struct.
    fn translate_struct_type(&self, struct_env: &StructEnv<'_>) {
        // Emit TypeName
        let struct_name = boogie_struct_name(&struct_env);
        emitln!(self.writer, "const unique {}: $TypeName;", struct_name);

        // Emit FieldNames
        for (i, field_env) in struct_env.get_fields().enumerate() {
            let field_name = boogie_field_name(&field_env);
            emitln!(
                self.writer,
                "const {}: $FieldName;\naxiom {} == {};",
                field_name,
                field_name,
                i
            );
        }

        // Emit TypeValue constructor function.
        let type_params = struct_env
            .get_type_parameters()
            .iter()
            .enumerate()
            .map(|(i, _)| format!("$tv{}: $TypeValue", i))
            .join(", ");
        let type_args = struct_env
            .get_type_parameters()
            .iter()
            .enumerate()
            .map(|(i, _)| Type::TypeParameter(i as u16))
            .collect_vec();
        let type_args_array = boogie_type_value_array(struct_env.module_env.env, &type_args);
        let type_value = format!("$StructType({}, {})", struct_name, type_args_array);
        emitln!(
            self.writer,
            "function {}_type_value({}): $TypeValue {{\n    {}\n}}",
            struct_name,
            type_params,
            type_value
        );

        // Emit memory variable.
        if struct_env.is_resource() {
            let memory_name = boogie_resource_memory_name(
                struct_env.module_env.env,
                struct_env.get_qualified_id(),
                &None,
            );
            emitln!(self.writer, "var {}: $Memory;", memory_name);
        }

        // Emit type assumption function.
        self.spec_translator
            .translate_assume_well_formed(&struct_env);
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
                format!("{}t{}: {}", prefix, i, boogie_local_type(ty))
            }))
            .join(", ");
        let mut_ref_count = (0..fun_target.get_parameter_count())
            .filter(|idx| fun_target.get_local_type(*idx).is_mutable_reference())
            .count();
        let rets = fun_target
            .get_return_types()
            .iter()
            .enumerate()
            .map(|(i, ref s)| format!("$ret{}: {}", i, boogie_local_type(s)))
            // Add implicit return parameters for &mut
            .chain(
                (0..mut_ref_count)
                    .map(|i| format!("$ret{}: $Mutation", fun_target.get_return_count() + i)),
            )
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
                boogie_local_type(local_type),
                boogie_type_value(self.module_env.env, local_type)
            );
        }
        // Generate declarations for renamed parameters.
        let proxied_parameters = self.get_mutable_parameters(fun_target);
        for (idx, ty) in &proxied_parameters {
            emitln!(self.writer, "var $t{}: {};", idx, boogie_local_type(ty));
        }
        // Generate declarations for modifies condition.
        fun_target.get_modify_targets().keys().for_each(|ty| {
            emitln!(
                self.writer,
                "var {}: {}",
                boogie_modifies_memory_name(fun_target.global_env(), *ty),
                "[$TypeValueArray, int]bool;"
            );
        });

        // Declare temporaries for debug tracing.
        emitln!(self.writer, "var $trace_abort_temp: int;");
        emitln!(self.writer, "var $trace_local_temp: $Value;");

        // Generate memory snapshot variable declarations.
        let code = fun_target.get_bytecode();
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
            emitln!(self.writer, "var {}: $Memory;", name);
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
                        "{{:msg \"assert_failed{}: {}\"}} ",
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
                    let boogie_mem =
                        boogie_resource_memory_name(self.module_env.env, mid.qualified(sid), &None);
                    let boogie_type_args = boogie_type_value_array(self.module_env.env, type_args);
                    emit!(
                        self.writer,
                        "call {} := $Modifies({}, {}, ",
                        boogie_mem,
                        boogie_mem,
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
                "if (b#$Boolean({})) {{ goto L{}; }} else {{ goto L{}; }}",
                str_local(*idx),
                then_target.as_usize(),
                else_target.as_usize(),
            ),
            Assign(_, dest, src, _) => {
                if fun_target.get_local_type(*dest).is_reference() {
                    emitln!(
                        self.writer,
                        "call {} := $CopyOrMoveRef({});",
                        str_local(*dest),
                        str_local(*src)
                    );
                } else {
                    emitln!(
                        self.writer,
                        "call {} := $CopyOrMoveValue({});",
                        str_local(*dest),
                        str_local(*src)
                    );
                }
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
                    Constant::Bool(true) => "$Boolean(true)".to_string(),
                    Constant::Bool(false) => "$Boolean(false)".to_string(),
                    Constant::U8(num) => format!("$Integer({})", num),
                    Constant::U64(num) => format!("$Integer({})", num),
                    Constant::U128(num) => format!("$Integer({})", num),
                    Constant::Address(val) => format!("$Address({})", val),
                    Constant::ByteArray(val) => {
                        format!("$Vector({})", boogie_byte_blob(self.options, val))
                    }
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
                            GlobalRoot(struct_decl) => {
                                let memory = struct_decl.module_id.qualified(struct_decl.id);
                                let memory_name = boogie_resource_memory_name(
                                    fun_target.global_env(),
                                    memory,
                                    &None,
                                );
                                let func = match edge {
                                    BorrowEdge::Weak => "WritebackToGlobalWeak",
                                    BorrowEdge::Strong(StrongEdge::Direct) => {
                                        "WritebackToGlobalStrong"
                                    }
                                    _ => {
                                        panic!("Strong global writeback cannot have field")
                                    }
                                };
                                emitln!(
                                    self.writer,
                                    "call {} := ${}({}, {});",
                                    memory_name,
                                    func,
                                    memory_name,
                                    str_local(src),
                                );
                            }
                            LocalRoot(idx) => {
                                let func = match edge {
                                    BorrowEdge::Weak => "WritebackToValueWeak",
                                    BorrowEdge::Strong(StrongEdge::Direct) => {
                                        "WritebackToValueStrong"
                                    }
                                    _ => {
                                        panic!("Strong local writeback cannot have field")
                                    }
                                };
                                emitln!(
                                    self.writer,
                                    "call {} := ${}({}, {}, {});",
                                    str_local(*idx),
                                    func,
                                    str_local(src),
                                    idx,
                                    str_local(*idx)
                                );
                            }
                            Reference(idx) => {
                                let (func, thirdarg): (&str, String) = match edge {
                                    BorrowEdge::Weak => {
                                        ("WritebackToReferenceWeak", "".to_string())
                                    }
                                    BorrowEdge::Strong(StrongEdge::Direct) => {
                                        ("WritebackToReferenceStrongDirect", "".to_string())
                                    }
                                    BorrowEdge::Strong(StrongEdge::Field(field)) => {
                                        ("WritebackToReferenceStrongField", format!(", {}", field))
                                    }
                                    BorrowEdge::Strong(StrongEdge::FieldUnknown) => {
                                        ("WritebackToVec", "".to_string())
                                    }
                                };
                                emitln!(
                                    self.writer,
                                    "call {} := ${}({}, {}{});",
                                    str_local(*idx),
                                    func,
                                    str_local(src),
                                    str_local(*idx),
                                    thirdarg
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
                        emitln!(
                            self.writer,
                            "call {} := $BorrowLoc({}, {});",
                            str_local(dest),
                            src,
                            str_local(src)
                        );
                    }
                    ReadRef => {
                        let src = srcs[0];
                        let dest = dests[0];
                        emitln!(
                            self.writer,
                            "call {} := $ReadRef({});",
                            str_local(dest),
                            str_local(src)
                        );
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
                    }
                    Function(mid, fid, type_actuals) => {
                        let callee_env = self.module_env.env.get_module(*mid).into_function(*fid);

                        let args_str = std::iter::once(boogie_type_values(
                            fun_target.func_env.module_env.env,
                            type_actuals,
                        ))
                        .chain(srcs.iter().map(|arg_idx| str_local(*arg_idx)))
                        .filter(|s| !s.is_empty())
                        .join(", ");

                        let dest_str = dests
                            .iter()
                            // Add implict dest returns for &mut srcs:
                            //  f(x) --> x := f(x)  with t(x) = &mut_
                            .chain(srcs.iter().filter(|idx| {
                                fun_target.get_local_type(**idx).is_mutable_reference()
                            }))
                            .map(|idx| str_local(*idx))
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
                        }
                        // Clear the last track location after function call, as the call inserted
                        // location tracks before it returns.
                        *last_tracked_loc = None;
                    }
                    Pack(mid, sid, _type_actuals) => {
                        let struct_env = fun_target
                            .func_env
                            .module_env
                            .env
                            .get_module(*mid)
                            .into_struct(*sid);
                        let mut ctor_expr = "$MapConstValue($DefaultValue())".to_owned();
                        for (i, field_env) in struct_env.get_fields().enumerate() {
                            ctor_expr = format!(
                                "{}[{} := {}]",
                                ctor_expr,
                                boogie_field_name(&field_env),
                                str_local(srcs[i])
                            );
                        }
                        emitln!(
                            self.writer,
                            "{} := $Vector($ValueArray({}, {}));",
                            str_local(dests[0]),
                            ctor_expr,
                            struct_env.get_field_count()
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
                            emitln!(
                                self.writer,
                                "{} := $SelectField({}, {});",
                                str_local(dests[i]),
                                str_local(srcs[0]),
                                boogie_field_name(&field_env)
                            );
                            let type_check = boogie_well_formed_check(
                                self.module_env.env,
                                &str_local(dests[i]),
                                &field_env.get_type(),
                            );
                            emit!(self.writer, &type_check);
                        }
                    }
                    BorrowField(mid, sid, _, field_offset) => {
                        let src = srcs[0];
                        let dest = dests[0];
                        let struct_env = fun_target
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
                    }
                    GetField(mid, sid, _, field_offset) => {
                        let src = srcs[0];
                        let dest = dests[0];
                        let struct_env = fun_target
                            .func_env
                            .module_env
                            .env
                            .get_module(*mid)
                            .into_struct(*sid);
                        let field_env = &struct_env.get_field_by_offset(*field_offset);
                        let is_ref = fun_target.get_local_type(src).is_reference();
                        emitln!(
                            self.writer,
                            "call {} := {}({}, {});",
                            str_local(dest),
                            if is_ref {
                                "$GetFieldFromReference"
                            } else {
                                "$GetFieldFromValue"
                            },
                            str_local(src),
                            boogie_field_name(field_env)
                        );
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
                        let memory = mid.qualified(*sid);
                        let memory_name =
                            boogie_resource_memory_name(self.module_env.env, memory, &None);
                        emitln!(
                            self.writer,
                            "call {} := $BorrowGlobal({}, {}, {});",
                            str_local(dest),
                            memory_name,
                            addr_name,
                            type_args,
                        );
                    }
                    GetGlobal(mid, sid, type_actuals) => {
                        let addr = srcs[0];
                        let dest = dests[0];
                        let type_args = boogie_type_value_array(self.module_env.env, type_actuals);
                        let memory = mid.qualified(*sid);
                        let memory_name =
                            boogie_resource_memory_name(self.module_env.env, memory, &None);
                        emitln!(
                            self.writer,
                            "call {} := $GetGlobal({}, {}, {});",
                            str_local(dest),
                            memory_name,
                            str_local(addr),
                            type_args,
                        );
                    }
                    MoveTo(mid, sid, type_actuals) => {
                        let value = srcs[0];
                        let signer = srcs[1];
                        let type_args = boogie_type_value_array(self.module_env.env, type_actuals);
                        let signer_name = str_local(signer);
                        let memory = mid.qualified(*sid);
                        let memory_name =
                            boogie_resource_memory_name(self.module_env.env, memory, &None);
                        emitln!(
                            self.writer,
                            "call {} := $MoveTo({}, {}, {}, {});",
                            memory_name,
                            memory_name,
                            type_args,
                            str_local(value),
                            signer_name,
                        );
                    }
                    MoveFrom(mid, sid, type_actuals) => {
                        let src = srcs[0];
                        let dest = dests[0];
                        let type_args = boogie_type_value_array(self.module_env.env, type_actuals);
                        let src_name = str_local(src);
                        let memory = mid.qualified(*sid);
                        let memory_name =
                            boogie_resource_memory_name(self.module_env.env, memory, &None);
                        emitln!(
                            self.writer,
                            "call {}, {} := $MoveFrom({}, {}, {});",
                            memory_name,
                            str_local(dest),
                            memory_name,
                            src_name,
                            type_args,
                        );
                    }
                    HavocVal => {
                        let temp_str = str_local(srcs[0]);
                        emitln!(self.writer, "havoc {};", temp_str);
                        // Insert a WellFormed check
                        let ty = fun_target.get_local_type(srcs[0]);
                        let check = boogie_well_formed_check(self.module_env.env, &temp_str, ty);
                        if !check.is_empty() {
                            emitln!(self.writer, &check);
                        }
                    }
                    HavocRef(havoc_all) => {
                        let havoc_procedure = if *havoc_all {
                            "HavocMutationAll"
                        } else {
                            "HavocMutation"
                        };
                        let temp_str = str_local(srcs[0]);
                        emitln!(
                            self.writer,
                            "call {} := ${}({});",
                            temp_str,
                            havoc_procedure,
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
                        emitln!(self.writer, "assume false;");
                        // a return statement terminates any execution trace that reaches it
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
                    Eq => {
                        let dest = dests[0];
                        let op1 = srcs[0];
                        let op2 = srcs[1];
                        emitln!(
                            self.writer,
                            "{} := $Boolean($IsEqual({}, {}));",
                            str_local(dest),
                            str_local(op1),
                            str_local(op2)
                        );
                    }
                    Neq => {
                        let dest = dests[0];
                        let op1 = srcs[0];
                        let op2 = srcs[1];
                        emitln!(
                            self.writer,
                            "{} := $Boolean(!$IsEqual({}, {}));",
                            str_local(dest),
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
                            if fun_target.get_local_type(idx).is_mutable_reference() {
                                format!("$Dereference({})", str_local(idx))
                            } else {
                                str_local(idx)
                            }
                        };
                        emit!(
                            self.writer,
                            "$es := ${}ExtendEventStore($es, ",
                            if srcs.len() > 2 { "Cond" } else { "" }
                        );
                        emit!(
                            self.writer,
                            "$SelectField({}, $Event_EventHandle_guid), {}",
                            translate_local(handle),
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
                    emitln!(self.writer, "{} := $Integer($abort_code);", code_str);
                    self.track_abort(fun_target, &code_str);
                    emitln!(self.writer, "goto L{};", target.as_usize());
                    self.writer.unindent();
                    emitln!(self.writer, "}");
                }
            }
            Abort(_, src) => {
                emitln!(
                    self.writer,
                    "$abort_code := i#$Integer({});",
                    str_local(*src)
                );
                emitln!(self.writer, "$abort_flag := true;");
                for (i, ty) in fun_target.get_return_types().iter().enumerate() {
                    let ret_str = format!("$ret{}", i);
                    if ty.is_reference() {
                        emitln!(self.writer, "{} := $DefaultMutation;", &ret_str);
                    } else {
                        emitln!(self.writer, "{} := $DefaultValue();", &ret_str);
                    }
                }
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
        // In order to determine whether we need to dereference, use the type of the temporary
        // which actually holds the value, not the original temp we are tracing.
        let ty = fun_target.get_local_type(idx);
        let mut value = format!("$t{}", idx);
        if ty.is_reference() {
            value = format!("$Dereference({})", value);
        }
        let track = boogie_debug_track_local(fun_target, origin_idx, &value);
        emitln!(self.writer, &track);
    }

    /// Generates an update of the debug information about the return value at given location.
    fn track_return(&self, fun_target: &FunctionTarget<'_>, return_idx: usize, idx: TempIndex) {
        let ty = fun_target.get_local_type(idx);
        let mut value = format!("$t{}", idx);
        if ty.is_reference() {
            value = format!("$Dereference({})", value);
        }
        emitln!(
            self.writer,
            &boogie_debug_track_return(fun_target, return_idx, &value)
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
}
