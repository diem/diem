// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    sandbox::utils::{
        package::{MovePackage, SourceFilter},
        OnDiskStateView,
    },
    DEFAULT_BUILD_DIR, DEFAULT_PACKAGE_DIR, DEFAULT_STORAGE_DIR,
};
use anyhow::{bail, Result};
use include_dir::{include_dir, Dir};
use move_binary_format::file_format::CompiledModule;
use once_cell::sync::Lazy;
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    str::FromStr,
};

use super::ModuleIdWithNamedAddress;

/// Content for the move stdlib directory
const DIR_MOVE_STDLIB: Dir = include_dir!("../../move-stdlib/modules");
/// Content for the nursery directory
const DIR_MOVE_STDLIB_NURSERY: Dir = include_dir!("../../move-stdlib/nursery");
/// Content for diem framework directory
const DIR_DIEM_FRAMEWORK: Dir = include_dir!("../../diem-framework/modules");

/// Pre-defined stdlib package
static PACKAGE_MOVE_STDLIB: Lazy<MovePackage> = Lazy::new(|| {
    MovePackage::new(
        "stdlib".to_string(),
        vec![
            SourceFilter {
                source_dir: &DIR_MOVE_STDLIB,
                inclusion: None, // include everything
                exclusion: HashSet::new(),
            },
            SourceFilter {
                source_dir: &DIR_MOVE_STDLIB_NURSERY,
                inclusion: None, // include everything
                exclusion: HashSet::new(),
            },
        ],
        vec![],
        Some("Std".to_owned()),
    )
});

static PACKAGE_DIEM_FRAMEWORK: Lazy<MovePackage> = Lazy::new(|| {
    MovePackage::new(
        "diem".to_string(),
        vec![SourceFilter {
            source_dir: &DIR_DIEM_FRAMEWORK,
            inclusion: None, // include everything
            exclusion: HashSet::new(),
        }],
        vec![&PACKAGE_MOVE_STDLIB],
        Some("DiemFramework".to_owned()),
    )
});

/// The dependency interface exposed to CLI main
pub struct Mode(Vec<&'static MovePackage>);

/// Set of supported modes
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum ModeType {
    /// No dependencies
    Bare,
    /// Move stdlib dependencies only (e.g., Vector, Signer)
    Stdlib,
    /// Include stdlib and all Diem dependencies
    Diem,
}

impl Mode {
    pub fn new(t: ModeType) -> Self {
        match t {
            ModeType::Bare => Mode(vec![]),
            ModeType::Stdlib => Mode(vec![&*PACKAGE_MOVE_STDLIB]),
            ModeType::Diem => Mode(vec![&*PACKAGE_DIEM_FRAMEWORK]),
        }
    }

    pub fn prepare_default_state() -> Result<OnDiskStateView> {
        Self::new(ModeType::default()).prepare_state(DEFAULT_STORAGE_DIR, DEFAULT_BUILD_DIR)
    }

    /// Prepare an OnDiskStateView that is ready to use. Library modules will be preloaded into the
    /// storage if `load_libraries` is true.
    ///
    /// NOTE: this is the only way to get a state view in Move CLI, and thus, this function needs
    /// to be run before every command that needs a state view, i.e., `check`, `publish`, `run`,
    /// `view`, and `doctor`.
    pub fn prepare_state<P: Into<PathBuf>>(
        &self,
        build_dir: P,
        storage_dir: P,
    ) -> Result<OnDiskStateView> {
        let build_dir_pb = build_dir.into();
        let state = OnDiskStateView::create(&build_dir_pb, &storage_dir.into())?;
        let package_dir = Path::new(&build_dir_pb).join(DEFAULT_PACKAGE_DIR);
        self.prepare(&package_dir, false)?;

        // preload the storage with library modules (if such modules do not exist yet)
        let lib_modules = self.compiled_modules(&package_dir)?;
        let new_modules: Vec<_> = lib_modules
            .into_iter()
            .filter(|(_, m)| !state.has_module(&m.self_id()))
            .collect();

        let mut serialized_modules = vec![];
        for (id, module) in new_modules {
            let mut module_bytes = vec![];
            module.serialize(&mut module_bytes)?;
            serialized_modules.push((id, module_bytes));
        }
        state.save_modules(&serialized_modules)?;

        Ok(state)
    }

    pub fn prepare(&self, out_path: &Path, source_only: bool) -> Result<()> {
        for pkg in &self.0 {
            pkg.prepare(out_path, source_only)?;
        }
        Ok(())
    }

    pub fn source_files(&self, out_path: &Path) -> Result<Vec<String>> {
        let pkg_sources: Result<Vec<_>, _> = self
            .0
            .iter()
            .map(|pkg| pkg.source_files(out_path))
            .collect();
        Ok(pkg_sources?.into_iter().flatten().collect())
    }

    pub fn compiled_modules(
        &self,
        out_path: &Path,
    ) -> Result<Vec<(ModuleIdWithNamedAddress, CompiledModule)>> {
        let pkg_modules = self
            .0
            .iter()
            .map(|pkg| pkg.compiled_modules(out_path))
            .collect::<Result<Vec<Vec<_>>>>()?;
        Ok(pkg_modules.into_iter().flatten().collect())
    }
}

impl FromStr for ModeType {
    type Err = anyhow::Error;

    fn from_str(mode: &str) -> Result<Self> {
        Ok(match mode {
            "bare" => ModeType::Bare,
            "stdlib" => ModeType::Stdlib,
            "diem" => ModeType::Diem,
            _ => bail!("Invalid mode for dependency: {}", mode),
        })
    }
}

impl ToString for ModeType {
    fn to_string(&self) -> String {
        match self {
            ModeType::Bare => "bare",
            ModeType::Stdlib => "stdlib",
            ModeType::Diem => "diem",
        }
        .to_string()
    }
}

impl Default for ModeType {
    fn default() -> Self {
        ModeType::Stdlib
    }
}

impl Default for Mode {
    fn default() -> Self {
        Self::new(ModeType::default())
    }
}
