// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    compilation::package_layout::CompiledPackageLayout,
    resolution::resolution_graph::{Renaming, ResolvedGraph, ResolvedPackage, ResolvedTable},
    source_package::parsed_manifest::{FileName, NamedAddress, PackageName},
    BuildConfig,
};
use abigen::{Abigen, AbigenOptions};
use anyhow::Result;
use bytecode_source_map::utils::source_map_from_file;
use colored::Colorize;
use docgen::{Docgen, DocgenOptions};
use move_binary_format::file_format::{CompiledModule, CompiledScript};
use move_command_line_common::files::{
    extension_equals, find_filenames, find_move_filenames, MOVE_COMPILED_EXTENSION,
    SOURCE_MAP_EXTENSION,
};
use move_core_types::language_storage::ModuleId;
use move_lang::{
    compiled_unit::{
        AnnotatedCompiledUnit, CompiledUnit, NamedCompiledModule, NamedCompiledScript,
    },
    shared::{AddressBytes, Flags},
    Compiler,
};
use move_model::{model::GlobalEnv, options::ModelBuilderOptions, run_model_builder_with_options};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    io::Write,
    path::{Path, PathBuf},
};

/// Module resolution data
pub type ModuleResolutionMetadata = BTreeMap<ModuleId, NamedAddress>;

/// Represents meta information about a package and the information it was compiled with. Shared
/// across both the `CompiledPackage` and `OnDiskCompiledPackage` structs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompiledPackageInfo {
    /// The name of the compiled package
    pub package_name: PackageName,
    /// The instantiations for all named addresses that were used for compilation
    pub address_alias_instantiation: ResolvedTable,
    /// The module resolution {<addr>::<module_name> |-> <named_address>} mapping used for
    /// compiling against bytecode using named addresses and to allow address renaming.
    pub module_resolution_metadata: ModuleResolutionMetadata,
    /// The hash of the source directory at the time of compilation. `None` if the source for this
    /// package is not available/this package was not compiled.
    pub source_digest: Option<String>,
    /// The build flags that were used when compiling this package.
    pub build_flags: BuildConfig,
}

/// Represents a compiled package in memory.
#[derive(Debug, Clone)]
pub struct CompiledPackage {
    /// Meta information about the compilation of this `CompiledPackage`
    pub compiled_package_info: CompiledPackageInfo,
    /// The source files in this package that were used for generation.
    pub sources: Vec<String>,
    /// The output compiled bytecode (both module, and scripts)
    pub compiled_units: Vec<CompiledUnit>,
    /// Packages that this package depends on. Non-transitive dependencies.
    pub dependencies: Vec<CompiledPackage>,

    // Optional artifacts from compilation
    //
    /// filename -> doctext
    pub compiled_docs: Option<Vec<(String, String)>>,
    /// filename -> json bytes for ScriptABI. Can then be used to generate transaction builders in
    /// various languages.
    pub compiled_abis: Option<Vec<(String, Vec<u8>)>>,
}

/// Represents a compiled package that has been saved to disk. This holds only the minimal metadata
/// needed to reconstruct a `CompiledPackage` package from it and to determine whether or not a
/// recompilation of the package needs to be performed or not.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnDiskCompiledPackage {
    /// Path to the root of the package and its data on disk.
    pub root_path: PathBuf,
    /// Information about the package and the specific compilation that was done.
    pub compiled_package_info: CompiledPackageInfo,
    /// Paths to dependencies for this package.
    pub dependencies: Vec<PathBuf>,
}

impl OnDiskCompiledPackage {
    pub fn from_path<P: AsRef<Path>>(p: P) -> Result<Self> {
        let buf = std::fs::read(p)?;
        let on_disk_package = serde_yaml::from_slice::<Self>(&buf)?;
        Ok(on_disk_package)
    }

    pub fn into_compiled_package(&self) -> Result<CompiledPackage> {
        let sources = find_move_filenames(
            &[self
                .root_path
                .join(CompiledPackageLayout::Sources.path())
                .to_string_lossy()
                .to_string()],
            false,
        )?;
        let compiled_units = self.get_compiled_units_paths()?;
        let source_maps = find_filenames(
            &[self
                .root_path
                .join(CompiledPackageLayout::SourceMaps.path())
                .to_string_lossy()
                .to_string()],
            |path| extension_equals(path, SOURCE_MAP_EXTENSION),
        )?;
        assert!(
            compiled_units.len() == source_maps.len(),
            "compiled units and source maps differ"
        );
        let compiled_units = compiled_units
            .iter()
            .zip(source_maps.iter())
            .map(|(bytecode_path, source_map_path)| {
                let bytecode_bytes = std::fs::read(bytecode_path.as_str())?;
                let source_map = source_map_from_file(Path::new(source_map_path))?;
                match CompiledModule::deserialize(&bytecode_bytes) {
                    Ok(module) => {
                        let (address_bytes, module_name) = {
                            let id = module.self_id();
                            let addr_bytes = AddressBytes::new(id.address().to_u8());
                            let module_name = FileName::from(id.name().as_str());
                            (addr_bytes, module_name)
                        };
                        Ok(CompiledUnit::Module(NamedCompiledModule {
                            address_bytes,
                            name: module_name,
                            module,
                            source_map,
                        }))
                    }
                    Err(_) => {
                        let script = CompiledScript::deserialize(&bytecode_bytes)?;
                        let name = FileName::from(
                            Path::new(bytecode_path.as_str())
                                .file_stem()
                                .unwrap()
                                .to_string_lossy()
                                .to_string(),
                        );
                        Ok(CompiledUnit::Script(NamedCompiledScript {
                            name,
                            script,
                            source_map,
                        }))
                    }
                }
            })
            .collect::<Result<Vec<_>>>()?;

        let mut dependencies = Vec::new();
        let self_path = Path::new(self.compiled_package_info.package_name.as_str());
        for dep_path in &self.dependencies {
            if dep_path == self_path {
                continue;
            }
            dependencies.push(
                Self::from_path(dep_path.join(CompiledPackageLayout::BuildInfo.path()))?
                    .into_compiled_package()?,
            )
        }

        let docs_path = self
            .root_path
            .join(CompiledPackageLayout::CompiledDocs.path());
        let compiled_docs = if docs_path.is_dir() {
            Some(
                find_filenames(&[docs_path.to_string_lossy().to_string()], |path| {
                    extension_equals(path, "md")
                })?
                .into_iter()
                .map(|path| {
                    let contents = std::fs::read_to_string(&path).unwrap();
                    (path, contents)
                })
                .collect(),
            )
        } else {
            None
        };

        let abi_path = self
            .root_path
            .join(CompiledPackageLayout::CompiledABIs.path());
        let compiled_abis = if abi_path.is_dir() {
            Some(
                find_filenames(&[abi_path.to_string_lossy().to_string()], |path| {
                    extension_equals(path, "abi")
                })?
                .into_iter()
                .map(|path| {
                    let contents = std::fs::read(&path).unwrap();
                    (path, contents)
                })
                .collect(),
            )
        } else {
            None
        };

        Ok(CompiledPackage {
            compiled_package_info: self.compiled_package_info.clone(),
            sources,
            compiled_units,
            dependencies,
            compiled_docs,
            compiled_abis,
        })
    }

    /// Save `bytes` under `path_under` relative to the package on disk
    pub(crate) fn save_under(
        &self,
        dir_or_file: &Path,
        path_under: Option<PathBuf>,
        bytes: &[u8],
    ) -> Result<()> {
        let mut path_to_save = self.root_path.join(dir_or_file);
        if let Some(under_path) = path_under {
            let parent = under_path.parent().unwrap();
            path_to_save.push(parent);
            std::fs::create_dir_all(&path_to_save)?;
            path_to_save.push(Path::new(under_path.file_name().unwrap()));
        }
        std::fs::write(path_to_save, bytes).map_err(|err| err.into())
    }

    pub(crate) fn has_source_changed_since_last_compile(
        &self,
        resolved_package: &ResolvedPackage,
    ) -> bool {
        match &self.compiled_package_info.source_digest {
            // Don't have source available to us
            None => false,
            Some(digest) => digest != &resolved_package.source_digest,
        }
    }
    pub(crate) fn are_build_flags_different(&self, build_config: &BuildConfig) -> bool {
        build_config != &self.compiled_package_info.build_flags
    }

    fn get_compiled_units_paths(&self) -> Result<Vec<String>> {
        let mut compiled_unit_paths = vec![self
            .root_path
            .join(CompiledPackageLayout::CompiledModules.path())
            .to_string_lossy()
            .to_string()];
        let compiled_scripts_path = self
            .root_path
            .join(CompiledPackageLayout::CompiledScripts.path());
        if compiled_scripts_path.exists() {
            compiled_unit_paths.push(compiled_scripts_path.to_string_lossy().to_string());
        }
        find_filenames(&compiled_unit_paths, |path| {
            extension_equals(path, MOVE_COMPILED_EXTENSION)
        })
    }
}

impl CompiledPackage {
    /// Returns all compiled modules for this package in transitive dependencies
    pub fn transitive_compiled_modules(&self) -> Vec<CompiledUnit> {
        self.transitive_dependencies()
            .into_iter()
            .flat_map(|compiled_package| compiled_package.compiled_units.clone())
            .collect()
    }

    /// Returns `CompiledPackage`s in dependency order
    pub fn transitive_dependencies(&self) -> Vec<&CompiledPackage> {
        self.dependencies
            .iter()
            .flat_map(|dep| {
                let mut dep_deps = dep.transitive_dependencies();
                dep_deps.push(dep);
                dep_deps
            })
            .collect()
    }

    pub(crate) fn build<W: Write>(
        w: &mut W,
        project_root: &Path,
        resolved_package: ResolvedPackage,
        dependencies: Vec<CompiledPackage>,
        resolution_graph: &ResolvedGraph,
    ) -> Result<CompiledPackage> {
        writeln!(
            w,
            "{} {} [{:?}]",
            "BUILDING".bold().green(),
            resolved_package.source_package.package.name,
            resolved_package.package_path
        )?;

        let mut module_resolution_metadata = BTreeMap::new();

        // NB: This renaming needs to be applied in the topological order of dependencies
        for package in &dependencies {
            package.apply_renaming(&mut module_resolution_metadata, &resolved_package.renaming)
        }

        let build_root_path = project_root.join(CompiledPackageLayout::Root.path());
        let path = build_root_path
            .join(resolved_package.source_package.package.name.as_str())
            .join(CompiledPackageLayout::BuildInfo.path());

        // Compare the digest of the package being compiled against the digest of the package at the time
        // of the last compilation to determine if we can reuse the already-compiled package or not.
        if let Ok(package) = OnDiskCompiledPackage::from_path(path) {
            if !package.has_source_changed_since_last_compile(&resolved_package)
                && !package.are_build_flags_different(&resolution_graph.build_options)
            {
                // Need to dive deeper to make sure that instantiations haven't changed since that
                // can be changed by other packages above us in the dependency graph possibly
                if package.compiled_package_info.address_alias_instantiation
                    == resolved_package.resolution_table
                {
                    return package.into_compiled_package();
                }
            }
        }

        let dep_paths = dependencies
            .iter()
            .map(|compiled_package| {
                build_root_path
                    .join(
                        compiled_package
                            .compiled_package_info
                            .package_name
                            .to_string(),
                    )
                    .join(CompiledPackageLayout::CompiledModules.path())
                    .to_string_lossy()
                    .to_string()
            })
            .collect::<Vec<_>>();

        let tmp_interface_dir = tempfile::tempdir()?;
        let in_scope_named_addrs = resolved_package
            .resolution_table
            .iter()
            .map(|(ident, addr)| (ident.to_string(), AddressBytes::new(addr.to_u8())))
            .collect::<BTreeMap<_, _>>();
        let sources: Vec<_> = resolved_package
            .get_sources(&resolution_graph.build_options)?
            .into_iter()
            .map(|symbol| symbol.as_str().to_string())
            .collect();
        let flags = if resolution_graph.build_options.test_mode {
            Flags::testing()
        } else {
            Flags::empty()
        };

        let (_, compiled_units) = Compiler::new(&sources, &dep_paths)
            .set_compiled_module_named_address_mapping(
                module_resolution_metadata
                    .clone()
                    .into_iter()
                    .map(|(x, ident)| (x, ident.to_string()))
                    .collect::<BTreeMap<_, _>>(),
            )
            .set_named_address_values(in_scope_named_addrs.clone())
            .set_interface_files_dir(tmp_interface_dir.path().to_string_lossy().to_string())
            .set_flags(flags)
            .build_and_report()?;

        let (compiled_units, resolutions): (Vec<_>, Vec<_>) = compiled_units
            .into_iter()
            .map(|annot_unit| match &annot_unit {
                AnnotatedCompiledUnit::Module(annot_module) => {
                    // Only return resolutions for modules that have named addresses
                    let resolution = match annot_module.module_id() {
                        (Some(str_name), module_id) => Some((module_id, str_name.value)),
                        _ => None,
                    };
                    (annot_unit.into_compiled_unit(), resolution)
                }
                AnnotatedCompiledUnit::Script(_) => (annot_unit.into_compiled_unit(), None),
            })
            .unzip();

        for (mod_id, name) in resolutions.into_iter().flatten() {
            module_resolution_metadata.insert(mod_id, name);
        }

        let mut compiled_docs = None;
        let mut compiled_abis = None;
        if resolution_graph.build_options.generate_docs
            || resolution_graph.build_options.generate_abis
        {
            let model = run_model_builder_with_options(
                &sources,
                &[tmp_interface_dir.path().to_string_lossy().to_string()],
                ModelBuilderOptions::default(),
                in_scope_named_addrs,
            )?;

            if resolution_graph.build_options.generate_docs {
                compiled_docs = Some(Self::build_docs(&model, &build_root_path, &dependencies));
            }

            if resolution_graph.build_options.generate_abis {
                compiled_abis = Some(Self::build_abis(&model, &compiled_units));
            }
        };

        let compiled_package = CompiledPackage {
            compiled_package_info: CompiledPackageInfo {
                package_name: resolved_package.source_package.package.name,
                address_alias_instantiation: resolved_package.resolution_table,
                module_resolution_metadata,
                source_digest: Some(resolved_package.source_digest),
                build_flags: resolution_graph.build_options.clone(),
            },
            sources,
            compiled_units,
            compiled_docs,
            compiled_abis,
            dependencies,
        };

        compiled_package.save_to_disk(build_root_path)?;

        Ok(compiled_package)
    }

    pub(crate) fn save_to_disk(&self, under_path: PathBuf) -> Result<OnDiskCompiledPackage> {
        let on_disk_package = OnDiskCompiledPackage {
            root_path: under_path.join(&self.compiled_package_info.package_name.to_string()),
            compiled_package_info: self.compiled_package_info.clone(),
            dependencies: self
                .dependencies
                .iter()
                .map(|dep| under_path.join(dep.compiled_package_info.package_name.as_str()))
                .collect(),
        };

        std::fs::create_dir_all(&on_disk_package.root_path)?;

        for source_path in &self.sources {
            on_disk_package.save_under(
                CompiledPackageLayout::Sources.path(),
                Some(PathBuf::from(Path::new(&source_path).file_name().unwrap())),
                std::fs::read_to_string(source_path)?.as_bytes(),
            )?;
        }

        for compiled_unit in &self.compiled_units {
            let under_path = match &compiled_unit {
                CompiledUnit::Script(_) => CompiledPackageLayout::CompiledScripts.path(),
                CompiledUnit::Module(_) => CompiledPackageLayout::CompiledModules.path(),
            };
            let path = match &compiled_unit {
                CompiledUnit::Script(named) => named.name.as_str(),
                CompiledUnit::Module(named) => named.name.as_str(),
            };
            on_disk_package.save_under(
                under_path,
                Some(Path::new(path).with_extension(MOVE_COMPILED_EXTENSION)),
                compiled_unit.serialize().as_slice(),
            )?;

            on_disk_package.save_under(
                CompiledPackageLayout::SourceMaps.path(),
                Some(Path::new(path).with_extension(SOURCE_MAP_EXTENSION)),
                compiled_unit.serialize_source_map().as_slice(),
            )?;
        }

        if let Some(docs) = &self.compiled_docs {
            for (doc_filename, doc_contents) in docs {
                on_disk_package.save_under(
                    CompiledPackageLayout::CompiledDocs.path(),
                    Some(Path::new(&doc_filename).with_extension("md")),
                    doc_contents.clone().as_bytes(),
                )?;
            }
        }

        if let Some(abis) = &self.compiled_abis {
            for (filename, abi_bytes) in abis {
                on_disk_package.save_under(
                    CompiledPackageLayout::CompiledABIs.path(),
                    Some(Path::new(&filename).with_extension("abi")),
                    abi_bytes,
                )?;
            }
        }

        on_disk_package.save_under(
            CompiledPackageLayout::BuildInfo.path(),
            None,
            serde_yaml::to_string(&on_disk_package)?.as_bytes(),
        )?;

        Ok(on_disk_package)
    }

    fn apply_renaming(
        &self,
        module_resolution: &mut ModuleResolutionMetadata,
        renaming: &Renaming,
    ) {
        let package_renamings = renaming
            .iter()
            .filter_map(|(rename_to, (package_name, from_name))| {
                if package_name == &self.compiled_package_info.package_name {
                    Some((from_name, *rename_to))
                } else {
                    None
                }
            })
            .collect::<BTreeMap<_, _>>();

        for (module_id, ident) in self
            .compiled_package_info
            .module_resolution_metadata
            .clone()
            .into_iter()
        {
            match package_renamings.get(&ident) {
                Some(rename) => module_resolution.insert(module_id, *rename),
                None => module_resolution.insert(module_id, ident),
            };
        }
    }

    fn build_abis(model: &GlobalEnv, compiled_units: &[CompiledUnit]) -> Vec<(String, Vec<u8>)> {
        let bytecode_map: BTreeMap<_, _> = compiled_units
            .iter()
            .map(|unit| match &unit {
                CompiledUnit::Script(script) => (script.name.to_string(), unit.serialize()),
                CompiledUnit::Module(module) => (module.name.to_string(), unit.serialize()),
            })
            .collect();
        let abi_options = AbigenOptions {
            in_memory_bytes: Some(bytecode_map),
            output_directory: "".to_string(),
            ..AbigenOptions::default()
        };
        let mut abigen = Abigen::new(model, &abi_options);
        abigen.gen();
        abigen.into_result()
    }

    fn build_docs(
        model: &GlobalEnv,
        build_root_path: &Path,
        deps: &[CompiledPackage],
    ) -> Vec<(String, String)> {
        let doc_options = DocgenOptions {
            doc_path: deps
                .iter()
                .map(|dep| {
                    build_root_path
                        .join(dep.compiled_package_info.package_name.as_str())
                        .join(CompiledPackageLayout::CompiledDocs.path())
                        .to_string_lossy()
                        .to_string()
                })
                .collect(),
            output_directory: "".to_string(),
            ..DocgenOptions::default()
        };
        let docgen = Docgen::new(model, &doc_options);
        docgen.gen()
    }
}
