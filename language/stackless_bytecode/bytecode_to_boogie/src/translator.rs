//! This module translates the bytecode of a module to Boogie code.

use bytecode_verifier::VerifiedModule;
use stackless_bytecode_generator::{
    stackless_bytecode::StacklessBytecode::{self, *},
    stackless_bytecode_generator::{StacklessFunction, StacklessModuleGenerator},
};
use std::collections::{BTreeMap, BTreeSet};
use vm::{
    access::ModuleAccess,
    file_format::{
        FieldDefinitionIndex, FunctionDefinition, FunctionHandleIndex, SignatureToken,
        StructDefinitionIndex, StructHandleIndex,
    },
    views::{
        FieldDefinitionView, FunctionHandleView, SignatureTokenView, StructDefinitionView,
        StructHandleView, ViewInternals,
    },
};

pub struct BoogieTranslator<'a> {
    pub module: &'a VerifiedModule,
    pub stackless_bytecode: Vec<StacklessFunction>,
    pub handle_to_def: BTreeMap<StructHandleIndex, usize>,
    pub max_struct_depth: usize,
    pub all_type_strs: BTreeSet<String>,
}

impl<'a> BoogieTranslator<'a> {
    pub fn new(module: &'a VerifiedModule) -> Self {
        let stackless_bytecode = StacklessModuleGenerator::new(module.as_inner()).generate_module();
        let mut handle_to_def = BTreeMap::new();
        for (idx, struct_def) in module.struct_defs().iter().enumerate() {
            handle_to_def.insert(struct_def.struct_handle, idx);
        }
        Self {
            module,
            stackless_bytecode,
            handle_to_def,
            max_struct_depth: 0,
            all_type_strs: BTreeSet::new(),
        }
    }

    pub fn translate(&mut self) -> String {
        let mut res = String::from("\n\n// everything below is auto generated\n\n");

        // generate names and struct specific functions for all structs
        for (i, _) in self.module.struct_handles().iter().enumerate() {
            let struct_handle_index = StructHandleIndex::new(i as u16);
            let struct_name = self.struct_name_from_handle_index(struct_handle_index);
            res.push_str(&format!("const unique {}: TypeName;\n", struct_name));
            let field_info = self.get_field_info_from_struct_handle_index(struct_handle_index);
            for (field_name, _) in field_info {
                res.push_str(&format!(
                    "const unique {}_{}: FieldName;\n",
                    struct_name, field_name
                ));
            }
            self.all_type_strs.insert(struct_name);
            res.push_str(&self.emit_struct_specific_functions(struct_handle_index));

            // calculate the max depth of a struct
            self.max_struct_depth = std::cmp::max(
                self.max_struct_depth,
                self.get_struct_depth(&SignatureToken::Struct(struct_handle_index, vec![])),
            );
        }

        // calculate maximum number of locals and generate this many local names
        let max_local_num = self
            .stackless_bytecode
            .iter()
            .map(|c| c.code.len())
            .fold(0, std::cmp::max);
        for i in 0..max_local_num {
            res.push_str(&format!("const unique t{}_LocalName: LocalName;\n", i,));
        }
        res.push_str("\n");

        // generate IsPrefix and UpdateValue to the max depth
        res.push_str(&self.emit_stratified_functions());

        // actual translation of stackless bytecode
        for (idx, function_def) in self.module.function_defs().iter().enumerate() {
            res.push_str(&self.translate_function(
                idx,
                function_def,
                &self.stackless_bytecode[idx],
            ));
        }
        res
    }

    fn get_struct_depth(&self, sig: &SignatureToken) -> usize {
        if let SignatureToken::Struct(idx, _) = sig {
            let mut max_field_depth = 0;
            let def_idx = StructDefinitionIndex::new(*self.handle_to_def.get(&idx).unwrap() as u16);
            let struct_definition = self.module.struct_def_at(def_idx);
            let struct_definition_view = StructDefinitionView::new(self.module, struct_definition);
            for field_definition_view in struct_definition_view.fields().unwrap() {
                let field_depth = self
                    .get_struct_depth(field_definition_view.type_signature().token().as_inner());
                max_field_depth = std::cmp::max(max_field_depth, field_depth);
            }
            max_field_depth + 1
        } else {
            0
        }
    }

    pub fn translate_function(
        &self,
        idx: usize,
        function_def: &'a FunctionDefinition,
        code: &StacklessFunction,
    ) -> String {
        // potential optimization: keep track of all the structs that get modified globally and add
        // parameters and return values only for those resource stores

        // identify all the branching targets so we can insert labels in front of them
        let mut res = String::new();
        let mut branching_targets: BTreeSet<usize> = BTreeSet::new();
        for bytecode in code.code.iter() {
            match bytecode {
                Branch(target) | BrTrue(target, _) | BrFalse(target, _) => {
                    branching_targets.insert(*target as usize);
                }
                _ => {}
            }
        }

        // generate function signature
        let fun_name = self.function_name_from_definition_index(idx);
        let function_handle = self.module.function_handle_at(function_def.function);
        let function_signature = self.module.function_signature_at(function_handle.signature);
        let num_args = function_signature.arg_types.len();
        let mut args = String::new();
        let mut rets = String::new();
        for (i, arg_type) in function_signature.arg_types.iter().enumerate() {
            args.push_str(&format!(
                ", t{}: {}",
                i,
                self.format_value_or_ref(&arg_type)
            ));
        }
        for (i, return_type) in function_signature.return_types.iter().enumerate() {
            if i > 0 {
                rets.push_str(", ");
            }
            rets.push_str(&format!(
                "ret{}: {}",
                i,
                self.format_value_or_ref(&return_type)
            ));
        }
        for (i, type_str) in self.all_type_strs.iter().enumerate() {
            args.push_str(&format!(", rs_{}: ResourceStore", type_str));
            if !rets.is_empty() || i > 0 {
                rets.push_str(", ");
            }
            rets.push_str(&format!("rs_{}': ResourceStore", type_str));
        }
        res.push_str(&format!(
            "procedure {} (c: CreationTime{}) returns ({}) {{\n",
            fun_name, args, rets
        ));

        res.push_str("    // declare local variables\n".into());

        let mut ref_vars = BTreeSet::new(); // set of locals that are references
        let mut val_vars = BTreeSet::new(); // set of locals that are not
        for (i, local_type) in code.local_types.iter().enumerate() {
            if i < num_args {
                continue;
            }
            if SignatureTokenView::new(self.module, local_type).is_reference() {
                ref_vars.insert(i);
            } else {
                val_vars.insert(i);
            }
            res.push_str(&format!(
                "    var t{}: {}; // {}\n",
                i,
                self.format_value_or_ref(&local_type),
                self.format_type(&local_type)
            ));
        }

        res.push_str("\n    // declare a new creation time for calls inside this functions;\n");
        res.push_str("    var c': CreationTime;\n    assume c' > c;\n");
        for type_str in self.all_type_strs.iter() {
            res.push_str(&format!("    rs_{}' := rs_{};\n", type_str, type_str));
        }
        res.push_str("\n    // bytecode translation starts here\n".into());
        for (offset, bytecode) in code.code.iter().enumerate() {
            // uncomment to print out bytecode for debugging purpose
            // println!("{:?}", bytecode);

            // insert labels for branching targets
            if branching_targets.contains(&offset) {
                res.push_str(&format!("Label_{}:\n", offset));
            }
            res.push_str(&self.translate_bytecode(bytecode, idx));
            if let WriteRef(dest, src) = bytecode {
                // update everything that might be related to the updated reference
                for s in &ref_vars {
                    if s == dest {
                        continue;
                    }
                    res.push_str(&format!(
                        "    call t{} := DeepUpdateReference(t{}, t{});\n",
                        s, dest, s
                    ));
                }
                for t in &self.all_type_strs {
                    res.push_str(&format!(
                        "    call rs_{}' := DeepUpdateGlobal({}, t{}, rs_{}');\n",
                        t, t, dest, t
                    ));
                }
                for s in &val_vars {
                    if s == src {
                        continue;
                    }
                    res.push_str(&format!(
                        "    call t{} := DeepUpdateLocal(c, t{}_LocalName, t{}, t{});\n",
                        s, s, dest, s
                    ));
                }
                res.push_str("\n");
            }
        }
        res.push_str("}\n".into());
        res
    }

    pub fn translate_bytecode(&self, bytecode: &StacklessBytecode, func_idx: usize) -> String {
        let mut res = String::new();
        let stmts = match bytecode {
            Branch(target) => vec![format!("goto Label_{};", target)],
            BrFalse(target, idx) => vec![format!(
                "if (!b#Boolean(t{})) {{ goto Label_{}; }}",
                idx, target
            )],
            MoveLoc(dest, src) => {
                if self.is_local_ref(*dest, func_idx) {
                    vec![format!("call t{} := CopyOrMoveRef(t{});", dest, src)]
                } else {
                    vec![format!("call t{} := CopyOrMoveValue(t{});", dest, src)]
                }
            }
            CopyLoc(dest, src) => {
                if self.is_local_ref(*dest, func_idx) {
                    vec![format!("call t{} := CopyOrMoveRef(t{});", dest, src)]
                } else {
                    vec![format!("call t{} := CopyOrMoveValue(t{});", dest, src)]
                }
            }
            StLoc(dest, src) => {
                if self.is_local_ref(*dest as usize, func_idx) {
                    // TODO: release the scoop
                    vec![format!("call t{} := CopyOrMoveRef(t{});", dest, src)]
                } else {
                    vec![format!("call t{} := CopyOrMoveValue(t{});", dest, src)]
                }
            }
            BorrowLoc(dest, src) => vec![format!(
                "call t{} := BorrowLoc(c, t{}_LocalName, t{});",
                dest, src, src
            )],
            ReadRef(dest, src) => vec![format!("call t{} := ReadRef(t{});", dest, src)],
            WriteRef(dest, src) => {
                vec![format!("call t{} := WriteRef(t{}, t{});", dest, dest, src)]
            }
            ReleaseRef(_) => vec!["// noop for release?".to_string()],
            FreezeRef(dest, src) => vec![format!("call t{} := FreezeRef(t{});", dest, src)],
            Call(dests, callee_index, args) => {
                let callee_name = self.function_name_from_handle_index(*callee_index);
                let mut dest_str = String::new();
                for (i, dest) in dests.iter().enumerate() {
                    if i > 0 {
                        dest_str.push_str(", ");
                    }
                    dest_str.push_str(&format!("t{}", dest));
                }
                let mut args_str = String::new();
                for arg in args.iter() {
                    args_str.push_str(&format!(", t{}", arg));
                }

                vec![format!(
                    "call {} := {}(c'{});",
                    dest_str, callee_name, args_str
                )]
            }
            Pack(dest, struct_def_index, fields) => {
                let struct_str = self.struct_name_from_definition_index(*struct_def_index);
                let mut fields_str = String::new();
                for (idx, field_temp) in fields.iter().enumerate() {
                    if idx > 0 {
                        fields_str.push_str(", ");
                    }
                    fields_str.push_str(&format!("t{}", field_temp));
                }
                vec![format!(
                    "call t{} := Pack_{}({});",
                    dest, struct_str, fields_str
                )]
            }
            Unpack(dests, struct_def_index, src) => {
                let struct_str = self.struct_name_from_definition_index(*struct_def_index);
                let mut dests_str = String::new();
                for (idx, dest) in dests.iter().enumerate() {
                    if idx > 0 {
                        dests_str.push_str(", ");
                    }
                    dests_str.push_str(&format!("t{}", dest));
                }
                vec![format!(
                    "call {} := Unpack_{}(t{});",
                    dests_str, struct_str, src
                )]
            }
            BorrowField(dest, src, field_def_index) => {
                let field_name = self.field_name_from_index(*field_def_index);
                vec![format!(
                    "call t{} := BorrowField(t{}, {});",
                    dest, src, field_name
                )]
            }
            Exists(dest, addr, struct_def_index) => {
                let struct_str = self.struct_name_from_definition_index(*struct_def_index);
                vec![format!(
                    "call t{} := Exists(t{}, rs_{}');",
                    dest, addr, struct_str
                )]
            }
            BorrowGlobal(dest, addr, struct_def_index) => {
                let struct_str = self.struct_name_from_definition_index(*struct_def_index);
                vec![format!(
                    "call t{} := BorrowGlobal(t{}, {}, rs_{}');",
                    dest, addr, struct_str, struct_str,
                )]
            }
            MoveToSender(src, struct_def_index) => {
                let struct_str = self.struct_name_from_definition_index(*struct_def_index);
                vec![format!(
                    "call rs_{}' := MoveToSender(rs_{}', t{});",
                    struct_str, struct_str, src,
                )]
            }
            MoveFrom(dest, src, struct_def_index) => {
                let struct_str = self.struct_name_from_definition_index(*struct_def_index);
                vec![format!(
                    "call t{}, rs_{}' := MoveFrom(t{}, rs_{}');",
                    dest, struct_str, src, struct_str,
                )]
            }
            Ret(rets) => {
                let mut ret_assignments = vec![];
                for (i, r) in rets.iter().enumerate() {
                    ret_assignments.push(format!("ret{} := t{};", i, r));
                }
                ret_assignments.push("return;".to_string());
                ret_assignments
            }
            LdTrue(idx) => vec![format!("call t{} := LdTrue();", idx)],
            LdFalse(idx) => vec![format!("call t{} := LdFalse();", idx)],
            LdConst(idx, num) => vec![format!("call t{} := LdConst({});", idx, num)],
            Not(dest, operand) => vec![format!("call t{} := Not(t{});", dest, operand)],
            Add(dest, op1, op2) => vec![format!("call t{} := Add(t{}, t{});", dest, op1, op2)],
            Sub(dest, op1, op2) => vec![format!("call t{} := Sub(t{}, t{});", dest, op1, op2)],
            Mul(dest, op1, op2) => vec![format!("call t{} := Mul(t{}, t{});", dest, op1, op2)],
            Div(dest, op1, op2) => vec![format!("call t{} := Div(t{}, t{});", dest, op1, op2)],
            Mod(dest, op1, op2) => vec![format!("call t{} := Mod(t{}, t{});", dest, op1, op2)],
            Lt(dest, op1, op2) => vec![format!("call t{} := Lt(t{}, t{});", dest, op1, op2)],
            Gt(dest, op1, op2) => vec![format!("call t{} := Gt(t{}, t{});", dest, op1, op2)],
            Le(dest, op1, op2) => vec![format!("call t{} := Le(t{}, t{});", dest, op1, op2)],
            Ge(dest, op1, op2) => vec![format!("call t{} := Ge(t{}, t{});", dest, op1, op2)],
            Or(dest, op1, op2) => vec![format!("call t{} := Or(t{}, t{});", dest, op1, op2)],
            And(dest, op1, op2) => vec![format!("call t{} := And(t{}, t{});", dest, op1, op2)],
            Eq(dest, op1, op2) => {
                let operand_type = self.get_local_type(*op1, func_idx);
                vec![format!(
                    "call t{} := Eq_{}(t{}, t{});",
                    dest, operand_type, op1, op2
                )]
            }
            Neq(dest, op1, op2) => {
                let operand_type = self.get_local_type(*op1, func_idx);
                vec![format!(
                    "call t{} := Neq_{}(t{}, t{});",
                    dest, operand_type, op1, op2
                )]
            }
            BitOr(_, _, _) | BitAnd(_, _, _) | Xor(_, _, _) => {
                vec!["// bit operation not supported".into()]
            }
            Abort(_) => vec!["// abort not supported".into()],
            _ => vec!["// unimplemented instruction".into()],
        };
        for code in stmts {
            res.push_str(&format!("    {}\n", code));
        }
        res.push('\n');
        res
    }

    /*
        utility functions below
    */
    pub fn struct_name_from_definition_index(&self, idx: StructDefinitionIndex) -> String {
        let struct_handle = self.module.struct_def_at(idx).struct_handle;
        self.struct_name_from_handle_index(struct_handle)
    }

    pub fn struct_name_from_handle_index(&self, idx: StructHandleIndex) -> String {
        let struct_handle = self.module.struct_handle_at(idx);
        let struct_handle_view = StructHandleView::new(self.module, struct_handle);
        let module_name = self
            .module
            .string_at(struct_handle_view.module_handle().name);
        let struct_name = struct_handle_view.name();
        format!("{}_{}", module_name, struct_name)
    }

    pub fn field_name_from_index(&self, idx: FieldDefinitionIndex) -> String {
        let field_definition = self.module.field_def_at(idx);
        let struct_handle_index = field_definition.struct_;
        let struct_name = self.struct_name_from_handle_index(struct_handle_index);
        let field_name = FieldDefinitionView::new(self.module, field_definition).name();
        format!("{}_{}", struct_name, field_name)
    }

    fn function_name_from_definition_index(&self, idx: usize) -> String {
        let function_handle_index = self.module.function_defs()[idx].function;
        self.function_name_from_handle_index(function_handle_index)
    }

    fn function_name_from_handle_index(&self, idx: FunctionHandleIndex) -> String {
        let function_handle = self.module.function_handle_at(idx);
        let module_handle_index = function_handle.module;
        let module_name = self
            .module
            .string_at(self.module.module_handle_at(module_handle_index).name);
        let function_handle_view = FunctionHandleView::new(self.module, function_handle);
        let function_name = function_handle_view.name();
        format!("{}_{}", module_name, function_name)
    }

    pub fn get_local_type(&self, local_idx: usize, func_idx: usize) -> String {
        self.format_type(&self.stackless_bytecode[func_idx].local_types[local_idx])
    }

    pub fn format_type_index(&self, sig: &SignatureToken) -> String {
        format!("{}_index", self.format_type(sig))
    }

    pub fn format_type(&self, sig: &SignatureToken) -> String {
        match sig {
            SignatureToken::Bool => "bool".into(),
            SignatureToken::U64 => "int".into(),
            SignatureToken::String => "string".into(),
            SignatureToken::ByteArray => "bytearray".into(),
            SignatureToken::Address => "address".into(),
            SignatureToken::Struct(idx, _) => self.struct_name_from_handle_index(*idx),
            SignatureToken::Reference(t) | SignatureToken::MutableReference(t) => {
                format!("{}_ref", self.format_type(&*t))
            }
            SignatureToken::TypeParameter(_) => "unsupported".into(),
        }
    }

    pub fn is_local_ref(&self, local_idx: usize, func_idx: usize) -> bool {
        let sig = &self.stackless_bytecode[func_idx].local_types[local_idx];
        match sig {
            SignatureToken::MutableReference(_) | SignatureToken::Reference(_) => true,
            _ => false,
        }
    }

    pub fn format_value_or_ref(&self, sig: &SignatureToken) -> String {
        match sig {
            SignatureToken::Reference(_) | SignatureToken::MutableReference(_) => "Reference",
            _ => "Value",
        }
        .into()
    }

    pub fn get_field_info_from_struct_handle_index(
        &self,
        idx: StructHandleIndex,
    ) -> BTreeMap<String, String> {
        let mut name_to_type = BTreeMap::new();
        let def_idx = StructDefinitionIndex::new(*self.handle_to_def.get(&idx).unwrap() as u16);
        let struct_definition = self.module.struct_def_at(def_idx);
        let struct_definition_view = StructDefinitionView::new(self.module, struct_definition);
        for field_definition_view in struct_definition_view.fields().unwrap() {
            let field_name = field_definition_view.name().to_string();
            let type_str =
                self.format_type(field_definition_view.type_signature().token().as_inner());
            name_to_type.insert(field_name, type_str);
        }
        name_to_type
    }
}
