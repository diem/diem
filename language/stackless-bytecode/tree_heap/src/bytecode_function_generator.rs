// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module generates the Boogie version of bytecode instructions in the format of Boogie
//! procedures.
use crate::translator::*;
use bytecode_verifier::VerifiedModule;
use vm::access::ModuleAccess;
impl BoogieTranslator {
    pub fn emit_stratified_functions(&self) -> String {
        let mut res = String::new();
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
                res.push_str("        v' := m#Map(v)[e];\n");
                res.push_str("        if (is#Vector(v)) { v' := v#Vector(v)[e]; }\n");
                res.push_str(&format!(
                    "        call v' := ReadValue{}(p, i+1, v');\n",
                    i - 1
                ));
                update_value_str.push_str("        e := p#Path(p)[i];\n");
                update_value_str.push_str("        v' := m#Map(v)[e];\n");
                update_value_str.push_str("        if (is#Vector(v)) { v' := v#Vector(v)[e]; }\n");
                update_value_str.push_str(&format!(
                    "        call v' := UpdateValue{}(p, i+1, v', new_v);\n",
                    i - 1,
                ));
                update_value_str
                    .push_str("        if (is#Map(v)) { v' := Map(m#Map(v)[e := v']);}\n");
                update_value_str.push_str("        if (is#Vector(v)) { v' := Vector(v#Vector(v)[e := v'], l#Vector(v));}\n");
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
        let field_info = get_field_info_from_def_index(module, def_idx);
        let struct_handle_index = module.struct_defs()[def_idx].struct_handle;
        let struct_name = struct_name_from_handle_index(module, struct_handle_index);
        let mut args_str = String::new();
        let mut typechecking_str = String::new();
        let mut fields_str = String::new();
        // pack
        for (i, (field_name, (_, value_cons))) in field_info.iter().enumerate() {
            if i > 0 {
                args_str.push_str(", ");
            }
            args_str.push_str(&format!("v{}: Value", i));
            typechecking_str.push_str(&format!("    assert is#{}(v{});\n", value_cons, i));
            fields_str.push_str(&format!(
                "[Field({}_{}) := v{}]",
                struct_name, field_name, i
            ));
        }
        res.push_str(&format!(
            "procedure {{:inline 1}} Pack_{}({}) returns (v: Value)\n{{\n",
            struct_name, args_str
        ));
        res.push_str(&typechecking_str);
        res.push_str(&format!("    v := Map(DefaultMap{});\n}}\n\n", fields_str));

        // unpack
        res.push_str(&format!(
            "procedure {{:inline 1}} Unpack_{}(v: Value) returns ({})\n{{\n",
            struct_name, args_str
        ));
        res.push_str("    assert is#Map(v);\n");
        for (i, (field_name, _)) in field_info.iter().enumerate() {
            res.push_str(&format!(
                "    v{} := m#Map(v)[Field({}_{})];\n",
                i, struct_name, field_name
            ));
        }
        res.push_str("}\n\n");

        // Eq
        let mut bool_res_str = String::new();
        let mut bool_assign_str = String::new();
        res.push_str(&format!(
            "procedure {{:inline 1}} Eq_{}(v1: Value, v2: Value) returns (res: Value)\n{{\n",
            struct_name,
        ));
        for (i, (field_name, (field_type, _))) in field_info.iter().enumerate() {
            res.push_str(&format!("    var b{}: Value;\n", i));
            bool_res_str.push_str(&format!(" && b#Boolean(b{})", i));
            bool_assign_str.push_str(&format!(
                "    call b{} := Eq_{}(m#Map(v1)[Field({}_{})], m#Map(v2)[Field({}_{})]);\n",
                i, field_type, struct_name, field_name, struct_name, field_name,
            ));
        }
        res.push_str("    assert is#Map(v1) && is#Map(v2);\n");
        res.push_str(&bool_assign_str);
        res.push_str(&format!(
            "    res := Boolean(true{});\n}}\n\n",
            bool_res_str
        ));

        // Neq
        res.push_str(&format!(
            "procedure {{:inline 1}} Neq_{}(v1: Value, v2: Value) returns (res: Value)\n{{\n",
            struct_name,
        ));
        res.push_str("    var res_val: Value;\n");
        res.push_str("    var res_bool: bool;\n");
        res.push_str("    assert is#Map(v1) && is#Map(v2);\n");
        res.push_str(&format!(
            "    call res_val := Eq_{}(v1, v2);\n",
            struct_name,
        ));
        res.push_str("    res := Boolean(!b#Boolean(res_val));\n}\n\n");
        res
    }
}
