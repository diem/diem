// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    source_package::{
        layout::SourcePackageLayout,
        manifest_parser::{parse_move_manifest_string, parse_source_manifest},
        parsed_manifest::{Dependency, NamedAddress, PackageName, SourceManifest, SubstOrRename},
    },
    BuildConfig,
};
use anyhow::{bail, Context, Result};
use move_command_line_common::files::find_move_filenames;
use move_core_types::account_address::AccountAddress;
use petgraph::{algo, graphmap::DiGraphMap};
use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
    rc::Rc,
};

pub type ResolvedTable = ResolutionTable<AccountAddress>;
pub type ResolvedPackage = ResolutionPackage<AccountAddress>;
pub type ResolvedGraph = ResolutionGraph<AccountAddress>;

// rename_to => (from_package name, from_address_name)
pub type Renaming = BTreeMap<NamedAddress, (PackageName, NamedAddress)>;
pub type GraphIndex = PackageName;

type ResolutionTable<T> = BTreeMap<NamedAddress, T>;
type ResolvingTable = ResolutionTable<ResolvingNamedAddress>;
type ResolvingGraph = ResolutionGraph<ResolvingNamedAddress>;
type ResolvingPackage = ResolutionPackage<ResolvingNamedAddress>;

#[derive(Debug, Clone)]
pub struct ResolvingNamedAddress {
    value: Rc<RefCell<Option<AccountAddress>>>,
}

/// A `ResolutionGraph` comes in two flavors:
/// 1. a `ResolutionGraph` during resolution (some named addresses may yet be instantiated)
/// 2. a `ResolvedGraph` which is a graph after resolution in which all named addresses have been
///    assigned a value.
///
/// Named addresses can be assigned values in a couple different ways:
/// 1. They can be assigned a value in the declaring package. In this case the value of that
///    named address will always be that value.
/// 2. Can be left unassigned in the declaring package. In this case it can receive its value
///    through unification across the package graph.
///
/// Named addresses can also be renamed in a package and will be re-exported under thes new names in this case.
#[derive(Debug, Clone)]
pub struct ResolutionGraph<T> {
    /// Build options
    pub build_options: BuildConfig,
    /// Root package
    pub root_package: SourceManifest,
    /// Dependency graph
    pub graph: DiGraphMap<PackageName, ()>,
    /// A mapping of package name to its resolution
    pub package_table: BTreeMap<PackageName, ResolutionPackage<T>>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ResolutionPackage<T> {
    /// Pointer into the `ResolutionGraph.graph`
    pub resolution_graph_index: GraphIndex,
    /// source manifest for this package
    pub source_package: SourceManifest,
    /// Where this package is located on the filesystem
    pub package_path: PathBuf,
    /// The renaming of addresses performed by this package
    pub renaming: Renaming,
    /// The mapping of addresses for this package (and that are in scope for it)
    pub resolution_table: ResolutionTable<T>,
    /// The digest of the contents of all files under the package root
    pub source_digest: String,
}

impl ResolvingGraph {
    pub fn new(
        root_package: SourceManifest,
        root_package_path: PathBuf,
        build_options: BuildConfig,
    ) -> Result<ResolvingGraph> {
        let mut resolution_graph = Self {
            build_options,
            root_package: root_package.clone(),
            graph: DiGraphMap::new(),
            package_table: BTreeMap::new(),
        };

        resolution_graph
            .build_resolution_graph(root_package.clone(), root_package_path, true)
            .with_context(|| {
                format!(
                    "Unable to resolve packages for package '{}'",
                    root_package.package.name
                )
            })?;
        Ok(resolution_graph)
    }

    pub fn resolve(self) -> Result<ResolvedGraph> {
        let ResolvingGraph {
            build_options,
            root_package,
            graph,
            package_table,
        } = self;

        let mut unresolved_addresses = Vec::new();

        let resolved_package_table = package_table
            .into_iter()
            .map(|(name, package)| {
                let ResolutionPackage {
                    resolution_graph_index,
                    source_package,
                    package_path,
                    renaming,
                    resolution_table,
                    source_digest,
                } = package;

                let resolved_table = resolution_table
                    .into_iter()
                    .filter_map(|(addr_name, instantiation_opt)| {
                        match *instantiation_opt.value.borrow() {
                            None => {
                                unresolved_addresses.push(format!(
                                    "Named address '{}' in package '{}'",
                                    addr_name, name
                                ));
                                None
                            }
                            Some(addr) => Some((addr_name, addr)),
                        }
                    })
                    .collect::<BTreeMap<_, _>>();
                (
                    name,
                    ResolvedPackage {
                        resolution_graph_index,
                        source_package,
                        package_path,
                        renaming,
                        resolution_table: resolved_table,
                        source_digest,
                    },
                )
            })
            .collect::<BTreeMap<_, _>>();

        if !unresolved_addresses.is_empty() {
            bail!(
                "Unresolved addresses found: [\n{}\n]",
                unresolved_addresses.join("\n")
            )
        }

        Ok(ResolvedGraph {
            build_options,
            root_package,
            graph,
            package_table: resolved_package_table,
        })
    }

    fn build_resolution_graph(
        &mut self,
        package: SourceManifest,
        package_path: PathBuf,
        is_root_package: bool,
    ) -> Result<()> {
        let package_name = package.package.name.clone();
        let package_node_id = match self.package_table.get(&package_name) {
            None => self.get_or_add_node(package_name.clone())?,
            // Same package and we've already resolved it: OK, return early
            Some(other) if other.source_package == package => return Ok(()),
            // Different packages, with same name: Not OK
            Some(other) => {
                bail!(
                    "Conflicting dependencies found: package '{}' conflicts with '{}'",
                    other.source_package.package.name,
                    package.package.name,
                )
            }
        };

        let mut renaming = BTreeMap::new();
        let mut resolution_table = BTreeMap::new();

        // include dev dependencies if in dev mode
        let additional_deps = if !self.build_options.dev_mode {
            package.dev_dependencies.clone()
        } else {
            BTreeMap::new()
        };

        for (dep_name, dep) in package
            .dependencies
            .clone()
            .into_iter()
            .chain(additional_deps.into_iter())
        {
            let dep_node_id = self.get_or_add_node(dep_name.clone()).with_context(|| {
                format!(
                    "Cycle between packages {} and {} found",
                    package_name, dep_name
                )
            })?;
            self.graph.add_edge(package_node_id, dep_node_id, ());

            let (dep_renaming, dep_resolution_table) = self
                .process_dependency(dep_name.clone(), dep, package_path.clone())
                .with_context(|| {
                    format!(
                        "While resolving dependency '{}' in package '{}'",
                        dep_name, package_name
                    )
                })?;

            ResolutionPackage::extend_renaming(&mut renaming, &dep_name, dep_renaming.clone())
                .with_context(|| {
                    format!(
                        "While resolving address renames in dependency '{}' in package '{}'",
                        dep_name, package_name
                    )
                })?;

            ResolutionPackage::extend_resolution_table(
                &mut resolution_table,
                &dep_name,
                dep_resolution_table,
                dep_renaming,
            )
            .with_context(|| {
                format!(
                    "Resolving named addresses for dependency '{}' in package '{}'",
                    dep_name, package_name
                )
            })?;
        }

        self.unify_addresses_in_package(&package, &mut resolution_table, is_root_package)?;

        let package_path = package_path.canonicalize()?;
        let source_digest = checksumdir::checksumdir(&package_path.to_string_lossy().to_string())?;

        let resolved_package = ResolutionPackage {
            resolution_graph_index: package_node_id,
            source_package: package,
            package_path,
            renaming,
            resolution_table,
            source_digest,
        };

        self.package_table.insert(package_name, resolved_package);
        Ok(())
    }

    fn unify_addresses_in_package(
        &mut self,
        package: &SourceManifest,
        resolution_table: &mut ResolvingTable,
        is_root_package: bool,
    ) -> Result<()> {
        let package_name = &package.package.name;
        for (name, addr_opt) in package
            .addresses
            .clone()
            .unwrap_or_else(BTreeMap::new)
            .into_iter()
        {
            match resolution_table.get(&name) {
                Some(other) => {
                    other.unify(addr_opt).with_context(|| {
                        format!(
                            "Unable to resolve named address '{}' in\
                                package '{}' when resolving dependencies",
                            name, package_name
                        )
                    })?;
                }
                None => {
                    resolution_table.insert(name, ResolvingNamedAddress::new(addr_opt));
                }
            }
        }

        if self.build_options.dev_mode && is_root_package {
            for (name, addr) in package
                .dev_address_assignments
                .clone()
                .unwrap_or_else(BTreeMap::new)
                .into_iter()
            {
                match resolution_table.get(&name) {
                    Some(other) => {
                        other.unify(Some(addr)).with_context(|| {
                            format!(
                                "Unable to resolve named address '{}' in\
                                    package '{}' when resolving dependencies in dev mode",
                                name, package_name
                            )
                        })?;
                    }
                    None => {
                        bail!(
                            "Found unbound dev address assignment '{} = 0x{}' in root package '{}'. \
                             Dev addresses cannot introduce new named addresses",
                            name,
                            addr.short_str_lossless(),
                            package_name
                        );
                    }
                }
            }
        }
        Ok(())
    }

    // Process a dependency. `dep_name_in_pkg` is the name assigned to the dependent package `dep`
    // in the source manifest, and we check that this name matches the name of the dependency it is
    // assigned to.
    fn process_dependency(
        &mut self,
        dep_name_in_pkg: PackageName,
        dep: Dependency,
        root_path: PathBuf,
    ) -> Result<(Renaming, ResolvingTable)> {
        let (dep_package, dep_package_dir) =
            Self::parse_package_manifest(&dep, &dep_name_in_pkg, root_path)
                .with_context(|| format!("While processing dependency '{}'", dep_name_in_pkg))?;
        self.build_resolution_graph(dep_package.clone(), dep_package_dir, false)
            .with_context(|| {
                format!("Unable to resolve package dependency '{}'", dep_name_in_pkg)
            })?;

        if dep_name_in_pkg != dep_package.package.name {
            bail!("Name of dependency declared in package '{}' does not match dependency's package name '{}'",
                dep_name_in_pkg,
                dep_package.package.name
            );
        }

        let resolving_dep = &self.package_table[&dep_name_in_pkg];
        let mut renaming = BTreeMap::new();
        let mut resolution_table = resolving_dep.resolution_table.clone();

        // check that address being renamed exists in the dep that is being renamed/imported
        if let Some(dep_subst) = dep.subst {
            for (name, rename_from_or_assign) in dep_subst.into_iter() {
                match rename_from_or_assign {
                    SubstOrRename::RenameFrom(ident) => {
                        // Make sure dep has the address that we're importing
                        if !resolving_dep.resolution_table.contains_key(&ident) {
                            bail!(
                                "Tried to rename named address {0} from package '{1}'.\
                                However, {1} does not contain that address",
                                ident,
                                dep_name_in_pkg
                            );
                        }

                        // Apply the substitution, NB that the refcell for the address's value is kept!
                        if let Some(other_val) = resolution_table.remove(&ident) {
                            resolution_table.insert(name.clone(), other_val);
                        }

                        if renaming
                            .insert(name.clone(), (dep_name_in_pkg.clone(), ident))
                            .is_some()
                        {
                            bail!("Duplicate renaming of named address '{0}' found for dependency {1}",
                                name,
                                dep_name_in_pkg,
                            );
                        }
                    }
                    SubstOrRename::Assign(value) => {
                        resolution_table
                            .get(&name)
                            .map(|named_addr| named_addr.unify(Some(value)))
                            .transpose()
                            .with_context(|| {
                                format!(
                                    "Unable to assign value to named address {} in dependency {}",
                                    name, dep_name_in_pkg
                                )
                            })?;
                    }
                }
            }
        }

        Ok((renaming, resolution_table))
    }

    fn get_or_add_node(&mut self, package_name: PackageName) -> Result<GraphIndex> {
        if self.graph.contains_node(package_name) {
            // If we encounter a node that we've already added we should check for cycles
            if algo::is_cyclic_directed(&self.graph) {
                // get the first cycle. Exists because we found a cycle above.
                let mut cycle = algo::kosaraju_scc(&self.graph)[0]
                    .windows(2)
                    .map(|pair| {
                        let end = pair[1];
                        end.as_str().to_string()
                    })
                    .collect::<Vec<_>>();
                // Add offending node as start and end
                cycle.insert(0, package_name.as_str().to_string());
                cycle.push(package_name.as_str().to_string());
                bail!("Found cycle between packages: {}", cycle.join(" -> "));
            }
            Ok(package_name)
        } else {
            Ok(self.graph.add_node(package_name))
        }
    }

    fn parse_package_manifest(
        dep: &Dependency,
        dep_name: &PackageName,
        mut root_path: PathBuf,
    ) -> Result<(SourceManifest, PathBuf)> {
        root_path.push(&dep.local);
        match std::fs::read_to_string(&root_path.join(SourcePackageLayout::Manifest.path())) {
            Ok(contents) => {
                let source_package: SourceManifest =
                    parse_move_manifest_string(contents).and_then(parse_source_manifest)?;
                Ok((source_package, root_path))
            }
            Err(_) => Err(anyhow::format_err!(
                "Unable to find package manifest for '{}' at {:?}",
                dep_name,
                SourcePackageLayout::Manifest.path().join(root_path),
            )),
        }
    }
}

impl ResolvingPackage {
    // Extend and check for duplicate names in rename_to
    fn extend_renaming(
        renaming: &mut Renaming,
        dep_name: &PackageName,
        dep_renaming: Renaming,
    ) -> Result<()> {
        for (rename_to, rename_from) in dep_renaming.into_iter() {
            // We cannot rename multiple named addresses to the same name.
            if renaming.insert(rename_to.clone(), rename_from).is_some() {
                bail!(
                    "Duplicate renaming of named address '{}' found in dependency '{}'",
                    rename_to,
                    dep_name
                );
            }
        }
        Ok(())
    }

    // The resolution table contains the transitive closure of addresses that are known in that
    // package. Extends the package's resolution table and checks for duplicate renamings that
    // conflict during this process.
    fn extend_resolution_table(
        resolution_table: &mut ResolvingTable,
        dep_name: &PackageName,
        dep_resolution_table: ResolvingTable,
        dep_renaming: Renaming,
    ) -> Result<()> {
        let renames = dep_renaming
            .into_iter()
            .map(|(rename_to, (_, rename_from))| (rename_from, rename_to))
            .collect::<BTreeMap<_, _>>();

        for (addr_name, addr_value) in dep_resolution_table.into_iter() {
            let addr_name = if let Some(rename_into) = renames.get(&addr_name) {
                rename_into.clone()
            } else {
                addr_name
            };
            if let Some(other) = resolution_table.insert(addr_name.clone(), addr_value.clone()) {
                // They need to be the same refcell so resolve to the same location if there are any
                // possible reassignments
                if other.value != addr_value.value {
                    bail!(
                        "Named address '{}' in dependency '{}' is already set to '{}' but was then reassigned to '{}'",
                        &addr_name,
                        dep_name,
                        match other.value.take() {
                            None => "unassigned".to_string(),
                            Some(addr) => format!("0x{}", addr.short_str_lossless()),
                        },
                        match addr_value.value.take() {
                            None => "unassigned".to_string(),
                            Some(addr) => format!("0x{}", addr.short_str_lossless()),
                        }
                    );
                }
            }
        }

        Ok(())
    }
}

impl ResolvingNamedAddress {
    pub fn new(address_opt: Option<AccountAddress>) -> Self {
        Self {
            value: Rc::new(RefCell::new(address_opt)),
        }
    }

    pub fn unify(&self, address_opt: Option<AccountAddress>) -> Result<()> {
        match address_opt {
            None => Ok(()),
            Some(addr_val) => match self.value.take() {
                None => {
                    self.value.replace(Some(addr_val));
                    Ok(())
                }
                Some(current_value) => {
                    if current_value != addr_val {
                        bail!("Attempted to assign a different value '0x{}' to an a already-assigned named address '0x{}'",
                                addr_val.short_str_lossless(), current_value.short_str_lossless()
                            )
                    } else {
                        self.value.replace(Some(current_value));
                        Ok(())
                    }
                }
            },
        }
    }
}

impl ResolvedGraph {
    pub fn get_package(&self, package_ident: &PackageName) -> &ResolvedPackage {
        self.package_table.get(package_ident).unwrap()
    }
}

impl ResolvedPackage {
    pub fn get_sources(&self, config: &BuildConfig) -> Result<Vec<String>> {
        let mut places_to_look = Vec::new();
        let mut add_path = |layout_path: SourcePackageLayout| {
            let path = self.package_path.join(layout_path.path());
            if layout_path.is_optional() && !path.exists() {
                return;
            }
            places_to_look.push(path.to_string_lossy().to_string())
        };

        add_path(SourcePackageLayout::Sources);
        add_path(SourcePackageLayout::Scripts);

        if config.dev_mode {
            add_path(SourcePackageLayout::Examples);
            add_path(SourcePackageLayout::Tests);
        }

        find_move_filenames(&places_to_look, false)
    }

    /// Returns the transitive dependencies of this package in dependency order
    pub fn transitive_dependencies(&self, resolved_graph: &ResolvedGraph) -> Vec<PackageName> {
        let mut seen = BTreeSet::new();
        let resolve_package = |(package_name, _): (&PackageName, _)| {
            let mut package_deps = resolved_graph
                .package_table
                .get(package_name)
                .unwrap()
                .transitive_dependencies(resolved_graph);
            package_deps.push(package_name.clone());
            package_deps
        };

        let transitive_deps: Vec<_> = if resolved_graph.build_options.dev_mode {
            self.source_package
                .dependencies
                .iter()
                .chain(self.source_package.dev_dependencies.iter())
                .flat_map(resolve_package)
                .collect()
        } else {
            self.source_package
                .dependencies
                .iter()
                .flat_map(resolve_package)
                .collect()
        };

        transitive_deps
            .into_iter()
            .filter(|ident| {
                if !seen.contains(ident) {
                    seen.insert(*ident);
                    true
                } else {
                    false
                }
            })
            .collect()
    }
}
