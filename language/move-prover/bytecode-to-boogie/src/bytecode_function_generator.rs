// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module generates the Boogie version of bytecode instructions in the format of Boogie
//! procedures.
use crate::translator::*;
use bytecode_verifier::VerifiedModule;
use itertools::Itertools;
use vm::access::ModuleAccess;

impl<'a> BoogieTranslator<'a> {
    pub fn emit_stratified_functions(&self) -> String {
        let mut res = String::new();
        res.push_str("\n\n// ** stratified functions\n\n");
        let mut update_value_str = String::new();
        for i in 0..=self.max_struct_depth {
            if i == self.max_struct_depth {
                res.push_str(
                    "procedure {:inline 1} ReadValueMax(p: Path, i: int, v: Value) returns (v': Value)\n{\n"
                );
                update_value_str.push_str(
                    "procedure {:inline 1} UpdateValueMax(p: Path, i: int, v: Value, new_v: Value) returns (v': Value)\n{\n"
                );
            } else {
                res.push_str(&format!(
                    "procedure {{:inline 1}} ReadValue{}(p: Path, i: int, v: Value) returns (v': Value)\n{{\n",
                    i,
                ));
                update_value_str.push_str(&format!(
                    "procedure {{:inline 1}} UpdateValue{}(p: Path, i: int, v: Value, new_v: Value) returns (v': Value)\n{{\n",
                    i,
                ));
            }

            res.push_str("    var e: Edge;\n");
            res.push_str("    if (i == size#Path(p)) {\n");
            res.push_str("        v' := v;\n");
            res.push_str("    } else {\n");

            update_value_str.push_str("    var e: Edge;\n");
            update_value_str.push_str("    if (i == size#Path(p)) {\n");
            update_value_str.push_str("        v' := new_v;\n");
            update_value_str.push_str("    } else {\n");

            if i == 0 {
                res.push_str("        assert false;\n");
                update_value_str.push_str("        assert false;\n");
            } else {
                res.push_str("        e := p#Path(p)[i];\n");
                res.push_str("        if (is#Struct(v)) { v' := smap(v)[f#Field(e)]; }\n");
                res.push_str("        if (is#Vector(v)) { v' := vmap(v)[i#Index(e)]; }\n");
                res.push_str(&format!(
                    "        call v' := ReadValue{}(p, i+1, v');\n",
                    i - 1
                ));
                update_value_str.push_str("        e := p#Path(p)[i];\n");
                update_value_str
                    .push_str("        if (is#Struct(v)) { v' := smap(v)[f#Field(e)]; }\n");
                update_value_str
                    .push_str("        if (is#Vector(v)) { v' := vmap(v)[i#Index(e)]; }\n");
                update_value_str.push_str(&format!(
                    "        call v' := UpdateValue{}(p, i+1, v', new_v);\n",
                    i - 1,
                ));
                update_value_str.push_str(
                    "        if (is#Struct(v)) { v' := mk_struct(smap(v)[f#Field(e) := v'], slen(v));}\n",
                );
                update_value_str.push_str(
                    "        if (is#Vector(v)) { v' := mk_vector(vmap(v)[i#Index(e) := v'], vlen(v));}\n");
            }
            res.push_str("    }\n}\n\n");
            update_value_str.push_str("    }\n}\n\n");
        }
        res.push_str(&update_value_str);
        res
    }

    pub fn emit_struct_specific_functions(
        &self,
        module: &VerifiedModule,
        def_idx: usize,
    ) -> String {
        let mut res = String::from("\n");
        let field_info = get_field_infos(module, &module.struct_defs()[def_idx]);
        let struct_handle_index = module.struct_defs()[def_idx].struct_handle;
        let struct_name = struct_name_from_handle_index(module, struct_handle_index);
        let struct_type_arity = struct_type_arity_from_handle_index(module, struct_handle_index);
        let mut type_args_str = String::new();
        let mut args_str = String::new();
        let mut typechecking_str = String::new();
        let mut fields_str = String::new();
        // pack
        for i in 0..struct_type_arity {
            if !type_args_str.is_empty() {
                type_args_str.push_str(", ");
            }
            type_args_str.push_str(&format!("tv{0}: TypeValue", i));
        }
        for (i, (field_name, field_type)) in field_info.iter().enumerate() {
            if !args_str.is_empty() {
                args_str.push_str(", ");
            }
            args_str.push_str(&format!("v{}: Value", i));
            typechecking_str.push_str(&format!(
                "    assume has_type({}, v{});\n",
                format_type_value(module, field_type),
                i
            ));
            fields_str.push_str(&format!("[{}_{} := v{}]", struct_name, field_name, i));
        }
        res.push_str(&format!(
            "procedure {{:inline 1}} Pack_{}({}) returns (v: Value)\n{{\n",
            struct_name,
            if type_args_str.is_empty() {
                args_str.clone()
            } else {
                format!("{}, {}", type_args_str, args_str)
            }
        ));
        res.push_str(&typechecking_str);
        res.push_str(&format!(
            "    v := Struct(ValueArray(DefaultIntMap{}, {}));\n",
            fields_str,
            field_info.len(),
        ));
        res.push_str(&format!(
            "    assume has_type({}_type_value({}), v);\n}}\n\n",
            struct_name,
            (0..struct_type_arity)
                .map(|i| format!("tv{}", i))
                .join(", ")
        ));

        // unpack
        res.push_str(&format!(
            "procedure {{:inline 1}} Unpack_{}(v: Value) returns ({})\n{{\n",
            struct_name, args_str
        ));
        res.push_str("    assert is#Struct(v);\n");
        for (i, (field_name, _)) in field_info.iter().enumerate() {
            res.push_str(&format!(
                "    v{} := smap(v)[{}_{}];\n",
                i, struct_name, field_name
            ));
        }
        res.push_str("}\n\n");

        res
    }
}
