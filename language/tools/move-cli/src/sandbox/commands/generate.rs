// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::sandbox::utils::on_disk_state_view::OnDiskStateView;
use anyhow::{bail, Result};
use move_binary_format::layout::SerdeLayoutBuilder;
use move_core_types::{
    identifier::Identifier,
    language_storage::{StructTag, TypeTag},
};
use std::path::Path;

pub fn generate_struct_layouts(
    path: &Path,
    struct_opt: &Option<String>,
    type_params_opt: &Option<Vec<TypeTag>>,
    shallow: bool,
    state: &OnDiskStateView,
) -> Result<()> {
    if let Some(module_id) = state.get_module_id(path) {
        if let Some(struct_) = struct_opt {
            // Generate for one struct
            let type_params = type_params_opt.as_ref().cloned().unwrap_or_default();
            let name = Identifier::new(struct_.as_str())?;
            let struct_tag = StructTag {
                address: *module_id.address(),
                module: module_id.name().to_owned(),
                name,
                type_params,
            };
            let mut layout_builder = if shallow {
                SerdeLayoutBuilder::new_shallow(state)
            } else {
                SerdeLayoutBuilder::new(state)
            };
            layout_builder.build_struct_layout(&struct_tag)?;
            let layout = serde_yaml::to_string(layout_builder.registry())?;
            state.save_struct_layouts(&layout)?;
            println!("{}", layout);
        } else {
            unimplemented!("Generating layout for all structs in a module. Use the --module and --struct options")
        }
        Ok(())
    } else {
        bail!("Can't resolve module at {:?}", path)
    }
}
