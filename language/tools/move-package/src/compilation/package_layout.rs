// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::path::Path;

#[derive(Debug, Clone)]
pub enum CompiledPackageLayout {
    BuildInfo,
    Root,
    Sources,
    SourceMaps,
    CompiledModules,
    CompiledScripts,
    CompiledDocs,
    CompiledABIs,
}

impl CompiledPackageLayout {
    pub fn path(&self) -> &Path {
        let path = match self {
            Self::BuildInfo => "BuildInfo.yaml",
            Self::Root => "build",
            Self::Sources => "sources",
            Self::SourceMaps => "source_maps",
            Self::CompiledModules => "bytecode_modules",
            Self::CompiledScripts => "bytecode_scripts",
            Self::CompiledDocs => "docs",
            Self::CompiledABIs => "abis",
        };
        Path::new(path)
    }
}
