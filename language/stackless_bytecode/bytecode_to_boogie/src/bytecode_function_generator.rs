//! This module generates the Boogie version of bytecode instructions in the format of Boogie
//! procedures.
use crate::translator::*;
use vm::file_format::StructHandleIndex;

impl<'a> BoogieTranslator<'a> {
    pub fn emit_stratified_functions(&self) -> String {
        let mut res = String::new();
        let mut update_value_str = String::new();
        for i in 0..=self.max_struct_depth {
            if i == self.max_struct_depth {
                res.push_str(
                    "procedure {:inline 1} IsPrefixMax(dstPath: Path, srcPath: Path) returns (isPrefix: bool)\n{\n"
                );
                update_value_str.push_str(
                    "procedure {:inline 1} UpdateValueMax(srcPath: Path, srcValue: Value, dstPath: Path, dstValue: Value) returns (dstValue': Value)\n{\n"
                );
            } else {
                res.push_str(&format!(
                    "procedure {{:inline 1}} IsPrefix{}(dstPath: Path, srcPath: Path) returns (isPrefix: bool)\n{{\n",
                    i,
                ));
                update_value_str.push_str(&format!(
                    "procedure {{:inline 1}} UpdateValue{}(srcPath: Path, srcValue: Value, dstPath: Path, dstValue: Value) returns (dstValue': Value)\n{{\n",
                    i,
                ));
            }

            res.push_str("    if (srcPath == dstPath) {\n");
            res.push_str("        isPrefix := true;\n");
            res.push_str("    } else if (srcPath == Nil()) {\n");
            res.push_str("        isPrefix := false;\n");
            res.push_str("    } else {\n");

            update_value_str.push_str("    var e: Edge;\n");
            update_value_str.push_str("    var v': Value;\n");
            update_value_str.push_str("    if (srcPath == dstPath) {\n");
            update_value_str.push_str("        dstValue' := srcValue;\n");
            update_value_str.push_str("    } else {\n");

            if i == 0 {
                res.push_str("        assert false;\n");
                update_value_str.push_str("        assume false;\n");
            } else {
                res.push_str(&format!(
                    "        call isPrefix := IsPrefix{}(dstPath, p#Cons(srcPath));\n",
                    i - 1,
                ));
                update_value_str.push_str(&format!(
                    "        call v' := UpdateValue{}(srcPath, srcValue, Cons(dstPath, e), m#Map(dstValue)[e]);\n",
                    i - 1,
                ));
                update_value_str.push_str("        dstValue' := Map(m#Map(dstValue)[e := v']);\n");
            }
            res.push_str("    }\n}\n\n");
            update_value_str.push_str("    }\n}\n\n");
        }
        res.push_str(&update_value_str);
        res
    }

    pub fn emit_struct_specific_functions(&self, idx: StructHandleIndex) -> String {
        let mut res = String::from("\n");
        let field_info = self.get_field_info_from_struct_handle_index(idx);
        let struct_name = self.struct_name_from_handle_index(idx);
        let mut args_str = String::new();
        let mut fields_str = String::new();
        // pack
        for (i, (field_name, _)) in field_info.iter().enumerate() {
            if i > 0 {
                args_str.push_str(", ");
            }
            args_str.push_str(&format!("v{}: Value", i));
            fields_str.push_str(&format!(
                "[Field({}_{}) := v{}]",
                struct_name, field_name, i
            ));
        }
        res.push_str(&format!(
            "procedure {{:inline 1}} Pack_{}({}) returns (v: Value)\n{{\n",
            struct_name, args_str
        ));
        res.push_str(&format!("    v := Map(DefaultMap{});\n}}\n\n", fields_str));

        // unpack
        res.push_str(&format!(
            "procedure {{:inline 1}} Unpack_{}(v: Value) returns ({})\n{{\n",
            struct_name, args_str
        ));
        for (i, (field_name, _)) in field_info.iter().enumerate() {
            res.push_str(&format!(
                "    v{} := m#Map(v)[Field({}_{})];\n",
                i, struct_name, field_name
            ));
        }
        res.push_str("}\n\n");

        // TODO: Eq

        // TODO: Neq
        res
    }
}
