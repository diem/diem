// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::sandbox::utils::on_disk_state_view::OnDiskStateView;
use anyhow::{bail, Result};
use move_binary_format::layout::StructLayoutBuilder;
use move_core_types::{
    identifier::Identifier,
    language_storage::{StructTag, TypeTag},
};
use std::path::Path;

pub fn generate_struct_layouts(
    path: &Path,
    struct_opt: &Option<String>,
    type_params_opt: &Option<Vec<TypeTag>>,
    state: &OnDiskStateView,
) -> Result<()> {
    if let Some(module_id) = state.get_module_id(path) {
        if let Some(struct_) = struct_opt {
            // Generate for one struct
            let type_params = type_params_opt.as_ref().unwrap().to_vec(); // always Some if struct_opt is
            let name = Identifier::new(struct_.as_str())?;
            let struct_tag = StructTag {
                address: *module_id.address(),
                module: module_id.name().to_owned(),
                name,
                type_params,
            };
            let layout = StructLayoutBuilder::build_with_fields(&struct_tag, state)?;
            // save to disk
            state.save_layout_yaml(struct_tag, &layout)?;
            println!("{}", layout);
        } else {
            unimplemented!("Generating layout for all structs in a module. Use the --module and --struct options")
        }
        Ok(())
    } else {
        bail!("Can't resolve module at {:?}", path)
    }
}
