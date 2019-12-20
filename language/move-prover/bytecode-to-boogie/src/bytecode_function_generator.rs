// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module generates the Boogie version of bytecode instructions in the format of Boogie
//! procedures.
use crate::translator::*;
use bytecode_verifier::VerifiedModule;
use vm::access::ModuleAccess;

impl<'a> BoogieTranslator<'a> {
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
        let mut fields_str = String::from("EmptyValueArray");
        // pack
        for i in 0..struct_type_arity {
            if !type_args_str.is_empty() {
                type_args_str.push_str(", ");
            }
            type_args_str.push_str(&format!("tv{0}: TypeValue", i));
        }
        for (i, (_, field_type)) in field_info.iter().enumerate() {
            if !args_str.is_empty() {
                args_str.push_str(", ");
            }
            args_str.push_str(&format!("v{}: Value", i));
            typechecking_str.push_str(&format!(
                "    {}",
                &format_type_checking(module, format!("v{}", i), field_type)
            ));
            fields_str = format!("ExtendValueArray({}, v{})", fields_str, i);
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
        res.push_str(&format!("    v := Vector({});\n", fields_str));
        res.push_str("\n}\n\n");

        // unpack
        res.push_str(&format!(
            "procedure {{:inline 1}} Unpack_{}(v: Value) returns ({})\n{{\n",
            struct_name, args_str
        ));
        res.push_str("    assume is#Vector(v);\n");
        for (i, (field_name, _)) in field_info.iter().enumerate() {
            res.push_str(&format!(
                "    v{} := SelectField(v, {}_{});\n",
                i, struct_name, field_name
            ));
        }
        res.push_str("}\n\n");

        res
    }
}
