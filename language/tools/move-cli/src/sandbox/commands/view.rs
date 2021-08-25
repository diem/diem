// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::sandbox::utils::{
    contains_module, is_bytecode_file, on_disk_state_view::OnDiskStateView,
};

use anyhow::{bail, Result};
use std::path::Path;
/// Print a module or resource stored in `file`
pub fn view(state: &OnDiskStateView, path: &Path) -> Result<()> {
    if state.is_resource_path(path) {
        match state.view_resource(path)? {
            Some(resource) => println!("{}", resource),
            None => println!("Resource not found."),
        }
    } else if state.is_event_path(path) {
        let events = state.view_events(path)?;
        if events.is_empty() {
            println!("Events not found.")
        } else {
            for event in events {
                println!("{}", event)
            }
        }
    } else if is_bytecode_file(path) {
        let bytecode_opt = if contains_module(path) {
            OnDiskStateView::view_module(path)?
        } else {
            // bytecode extension, but not a module--assume it's a script
            OnDiskStateView::view_script(path)?
        };
        match bytecode_opt {
            Some(bytecode) => println!("{}", bytecode),
            None => println!("Bytecode not found."),
        }
    } else {
        bail!("`move view <file>` must point to a valid file under storage")
    }
    Ok(())
}
