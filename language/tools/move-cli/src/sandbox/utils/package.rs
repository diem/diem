// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::ModuleIdWithNamedAddress;
use anyhow::{anyhow, Result};
use include_dir::Dir;
use move_binary_format::file_format::CompiledModule;
use move_command_line_common::files::{
    extension_equals, find_filenames, path_to_string, MOVE_COMPILED_EXTENSION, MOVE_EXTENSION,
};
use move_lang::{
    compiled_unit::CompiledUnit,
    shared::{AddressBytes, Flags},
    Compiler,
};
use move_symbol_pool::Symbol;
use once_cell::sync::Lazy;
use std::{
    collections::{BTreeMap, HashSet},
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

/// Directory name for the package source files under package/<name>
const PKG_SOURCE_DIR: &str = "source_files";
/// Directory name for the package binary files under package/<name>
const PKG_BINARY_DIR: &str = "compiled";

pub struct SourceFilter<'a> {
    /// The embedded directory
    pub source_dir: &'a Dir<'a>,
    /// Source files to be included, if set to None, include everything
    pub inclusion: Option<HashSet<&'a str>>,
    /// Source files to be excluded, to exclude nothing, set it to empty
    pub exclusion: HashSet<&'a str>,
}

impl<'a> SourceFilter<'a> {
    fn should_include_file(&self, filename: &str) -> bool {
        !self.exclusion.contains(filename)
            && self
                .inclusion
                .as_ref()
                .map_or(true, |set| set.contains(filename))
    }

    fn prepare_source_files_recursive(&self, dir: &'a Dir<'a>, output_dir: &Path) -> Result<()> {
        fs::create_dir_all(output_dir.join(dir.path()))?;

        for subdir in dir.dirs() {
            self.prepare_source_files_recursive(subdir, output_dir)?;
        }

        for file in dir.files() {
            let path = file.path();
            if path
                .extension()
                .map_or(false, |ext| ext.to_str().unwrap() == MOVE_EXTENSION)
                && self.should_include_file(path.to_str().unwrap())
            {
                let file_path = output_dir.join(path);
                let mut fp = File::create(file_path)?;
                fp.write_all(file.contents())?;
            }
        }

        Ok(())
    }

    fn prepare_source_files(&self, output_dir: &Path) -> Result<()> {
        self.prepare_source_files_recursive(self.source_dir, output_dir)
    }
}

pub struct MovePackage {
    /// Name of the package
    name: String,
    /// The directory containing all the .move source files
    sources: Vec<SourceFilter<'static>>,
    /// Dependencies
    deps: Vec<&'static Lazy<MovePackage>>,
    /// Named address values to be be used in this package.
    named_addresses: BTreeMap<String, AddressBytes>,
    /// Hack to support named addresses. Works now since our current packages only have one address
    // TODO properly support named addresses, will require migrating to new/planned build system
    named_address_hack: Option<String>,
}

impl MovePackage {
    pub fn new(
        name: String,
        sources: Vec<SourceFilter<'static>>,
        deps: Vec<&'static Lazy<MovePackage>>,
        named_addresses: BTreeMap<String, AddressBytes>,
        named_address: Option<String>,
    ) -> Self {
        MovePackage {
            name,
            sources,
            deps,
            named_addresses,
            named_address_hack: named_address,
        }
    }

    fn get_package_dir(&self, out_path: &Path) -> PathBuf {
        out_path.join(&self.name)
    }

    fn get_source_dir(&self, out_path: &Path) -> PathBuf {
        self.get_package_dir(out_path).join(PKG_SOURCE_DIR)
    }

    fn get_binary_dir(&self, out_path: &Path) -> PathBuf {
        self.get_package_dir(out_path).join(PKG_BINARY_DIR)
    }

    /// Prepare the package, lay down the source files and compile the modules
    pub(crate) fn prepare(&self, out_path: &Path, source_only: bool) -> Result<Vec<String>> {
        // bottom-up by preparing the dependencies first
        let mut src_dirs = vec![];
        for dep in self.deps.iter() {
            src_dirs.extend(dep.prepare(out_path, source_only)?);
        }

        // package directory layouts
        let pkg_path = self.get_package_dir(out_path);
        fs::create_dir_all(&pkg_path)?;
        let pkg_src_path = self.get_source_dir(out_path);
        let pkg_bin_path = self.get_binary_dir(out_path);

        // if we have processed the package, shortcut the execution
        // otherwise, prepare the output directory and its contents
        if !pkg_src_path.exists() {
            // splash the source files
            for entry in self.sources.iter() {
                entry.prepare_source_files(&pkg_src_path)?;
            }
        }
        if !source_only && !pkg_bin_path.exists() {
            fs::create_dir_all(&pkg_bin_path)?;

            // compile the source files
            let (_files, compiled_units) =
                Compiler::new(&[path_to_string(&pkg_src_path)?], &src_dirs)
                    .set_flags(Flags::empty().set_sources_shadow_deps(false))
                    .set_named_address_values(self.named_addresses.clone())
                    .build_and_report()?;

            // save modules and ignore scripts
            for unit in compiled_units {
                match unit {
                    CompiledUnit::Module { ident, module, .. } => {
                        let mut data = vec![];
                        module.serialize(&mut data)?;
                        let file_path = pkg_bin_path
                            .join(ident.module_name.0.value.as_str())
                            .with_extension(MOVE_COMPILED_EXTENSION);
                        let mut fp = File::create(file_path)?;
                        fp.write_all(&data)?;
                    }
                    CompiledUnit::Script { loc, .. } => eprintln!(
                        "Warning: Found a script in given dependencies. \
                            The script will be ignored: {}",
                        loc.file()
                    ),
                }
            }
        }

        // done
        src_dirs.push(pkg_src_path.into_os_string().into_string().unwrap());
        Ok(src_dirs)
    }

    pub(crate) fn source_files(&self, out_path: &Path) -> Result<Vec<String>> {
        let mut src_dirs = vec![];
        for dep in self.deps.iter() {
            src_dirs.extend(dep.source_files(out_path)?);
        }
        src_dirs.push(path_to_string(&self.get_source_dir(out_path))?);
        Ok(src_dirs)
    }

    pub(crate) fn named_addresses(&self) -> &BTreeMap<String, AddressBytes> {
        &self.named_addresses
    }

    pub(crate) fn compiled_modules(
        &self,
        out_path: &Path,
    ) -> Result<Vec<(ModuleIdWithNamedAddress, CompiledModule)>> {
        let mut modules = vec![];
        let named_address = self
            .named_address_hack
            .as_ref()
            .map(|n| Symbol::from(n.as_str()));
        for dep in self.deps.iter() {
            modules.extend(dep.compiled_modules(out_path)?);
        }
        for entry in find_filenames(&[path_to_string(&self.get_binary_dir(out_path))?], |path| {
            extension_equals(path, MOVE_COMPILED_EXTENSION)
        })? {
            let module = CompiledModule::deserialize(&fs::read(Path::new(&entry)).unwrap())
                .map_err(|e| anyhow!("Failure deserializing module {}: {:?}", entry, e))?;
            modules.push(((module.self_id(), named_address), module));
        }
        Ok(modules)
    }
}
