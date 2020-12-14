// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{anyhow, bail, Result};
use include_dir::{include_dir, Dir};
use once_cell::sync::Lazy;
use std::{
    collections::HashSet,
    fs::{self, File},
    io::Write,
    path::Path,
};

use move_lang::{
    compiled_unit::CompiledUnit, extension_equals, find_filenames, move_compile_and_report,
    path_to_string,
};
use std::path::PathBuf;
use stdlib::{COMPILED_EXTENSION, MOVE_EXTENSION};
use vm::file_format::CompiledModule;

/// Directory name for the package source files under package/<name>
const PKG_SOURCE_DIR: &str = "source_files";
/// Directory name for the package binary files under package/<name>
const PKG_BINARY_DIR: &str = "compiled";

/// Content for the stdlib directory
const DIR_STDLIB: Dir = include_dir!("../../stdlib/modules");
/// Content for the nursery directory
const DIR_NURSERY: Dir = include_dir!("../../stdlib/nursery");

/// The portion in the stdlib directory that are considered stdlib
const STDLIB_MODULE_FILES: [&str; 8] = [
    "Errors.move",
    "Event.move",
    "FixedPoint32.move",
    "Hash.move",
    "BCS.move",
    "Option.move",
    "Vector.move",
    "Signer.move",
];

/// Pre-defined stdlib package
static PACKAGE_STDLIB: Lazy<MovePackage> = Lazy::new(|| MovePackage {
    name: "stdlib",
    sources: vec![
        SourceFilter {
            source_dir: &DIR_STDLIB,
            inclusion: Some(STDLIB_MODULE_FILES.iter().copied().collect()),
            exclusion: HashSet::new(),
        },
        SourceFilter {
            source_dir: &DIR_NURSERY,
            inclusion: None, // include everything
            exclusion: HashSet::new(),
        },
    ],
    deps: vec![],
});

static PACKAGE_DIEM: Lazy<MovePackage> = Lazy::new(|| MovePackage {
    name: "diem",
    sources: vec![SourceFilter {
        source_dir: &DIR_STDLIB,
        inclusion: None, // include everything
        exclusion: STDLIB_MODULE_FILES.iter().copied().collect(),
    }],
    deps: vec![&PACKAGE_STDLIB],
});

struct SourceFilter<'a> {
    /// The embedded directory
    source_dir: &'a Dir<'a>,
    /// Source files to be included, if set to None, include everything
    inclusion: Option<HashSet<&'a str>>,
    /// Source files to be excluded, to exclude nothing, set it to empty
    exclusion: HashSet<&'a str>,
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

struct MovePackage {
    /// Name of the package
    name: &'static str,
    /// The directory containing all the .move source files
    sources: Vec<SourceFilter<'static>>,
    /// Dependencies
    deps: Vec<&'static Lazy<MovePackage>>,
}

impl MovePackage {
    fn get_package_dir(&self, out_path: &Path) -> PathBuf {
        out_path.join(self.name)
    }

    fn get_source_dir(&self, out_path: &Path) -> PathBuf {
        self.get_package_dir(out_path).join(PKG_SOURCE_DIR)
    }

    fn get_binary_dir(&self, out_path: &Path) -> PathBuf {
        self.get_package_dir(out_path).join(PKG_BINARY_DIR)
    }

    /// Prepare the package, lay down the source files and compile the modules
    fn prepare(&self, out_path: &Path, source_only: bool) -> Result<Vec<String>> {
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
                move_compile_and_report(&[path_to_string(&pkg_src_path)?], &src_dirs, None, None)?;

            // save modules and ignore scripts
            for unit in compiled_units {
                match unit {
                    CompiledUnit::Module { ident, module, .. } => {
                        let mut data = vec![];
                        module.serialize(&mut data)?;
                        let file_path = pkg_bin_path
                            .join(ident.0.value.name.0.value)
                            .with_extension(COMPILED_EXTENSION);
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

    fn source_files(&self, out_path: &Path) -> Result<Vec<String>> {
        let mut src_dirs = vec![];
        for dep in self.deps.iter() {
            src_dirs.extend(dep.source_files(out_path)?);
        }
        src_dirs.push(path_to_string(&self.get_source_dir(out_path))?);
        Ok(src_dirs)
    }

    fn compiled_modules(&self, out_path: &Path) -> Result<Vec<CompiledModule>> {
        let mut modules = vec![];
        for dep in self.deps.iter() {
            modules.extend(dep.compiled_modules(out_path)?);
        }
        for entry in find_filenames(&[path_to_string(&self.get_binary_dir(out_path))?], |path| {
            extension_equals(path, COMPILED_EXTENSION)
        })? {
            modules.push(
                CompiledModule::deserialize(&fs::read(Path::new(&entry)).unwrap())
                    .map_err(|e| anyhow!("Failure deserializing module {}: {:?}", entry, e))?,
            );
        }
        Ok(modules)
    }
}

/// The dependency interface exposed to CLI main
pub struct Mode(Vec<&'static MovePackage>);

impl Mode {
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

    pub fn compiled_modules(&self, out_path: &Path) -> Result<Vec<CompiledModule>> {
        let pkg_modules: Result<Vec<_>, _> = self
            .0
            .iter()
            .map(|pkg| pkg.compiled_modules(out_path))
            .collect();
        Ok(pkg_modules?.into_iter().flatten().collect())
    }
}

pub fn parse_mode_from_string(mode: &str) -> Result<Mode> {
    match mode {
        "bare" => Ok(Mode(vec![])),
        "stdlib" => Ok(Mode(vec![&*PACKAGE_STDLIB])),
        "diem" => Ok(Mode(vec![&*PACKAGE_DIEM])),
        _ => bail!("Invalid mode for dependency: {}", mode),
    }
}
