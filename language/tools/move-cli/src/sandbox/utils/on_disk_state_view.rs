// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{BCS_EXTENSION, DEFAULT_BUILD_DIR, DEFAULT_STORAGE_DIR};
use anyhow::{anyhow, bail, Result};
use disassembler::disassembler::Disassembler;
use move_binary_format::{
    access::ModuleAccess,
    binary_views::BinaryIndexedView,
    file_format::{CompiledModule, CompiledScript, FunctionDefinitionIndex},
};
use move_command_line_common::files::MOVE_COMPILED_EXTENSION;
use move_core_types::{
    account_address::AccountAddress,
    identifier::Identifier,
    language_storage::{ModuleId, StructTag, TypeTag},
    parser,
    resolver::{ModuleResolver, ResourceResolver},
};
use move_lang::{shared::AddressBytes, MOVE_COMPILED_INTERFACES_DIR};
use resource_viewer::{AnnotatedMoveStruct, AnnotatedMoveValue, MoveValueAnnotator};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    convert::{TryFrom, TryInto},
    fs,
    path::{Path, PathBuf},
};

type Event = (Vec<u8>, u64, TypeTag, Vec<u8>);

/// subdirectory of `DEFAULT_STORAGE_DIR`/<addr> where resources are stored
pub const RESOURCES_DIR: &str = "resources";
/// subdirectory of `DEFAULT_STORAGE_DIR`/<addr> where modules are stored
pub const MODULES_DIR: &str = "modules";
/// subdirectory of `DEFAULT_STORAGE_DIR`/<addr> where events are stored
pub const EVENTS_DIR: &str = "events";

pub type ModuleIdWithNamedAddress = (ModuleId, Option<String>);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct InterfaceFilesMetadata {
    named_address_mapping: BTreeMap<ModuleId, String>,
    named_address_values: BTreeMap<String, String>,
}

#[derive(Debug)]
pub struct OnDiskStateView {
    build_dir: PathBuf,
    storage_dir: PathBuf,
}

impl OnDiskStateView {
    /// Create an `OnDiskStateView` that reads/writes resource data and modules in `storage_dir`.
    pub fn create<P: Into<PathBuf>>(build_dir: P, storage_dir: P) -> Result<Self> {
        let build_dir = build_dir.into();
        if !build_dir.exists() {
            fs::create_dir_all(&build_dir)?;
        }

        let storage_dir = storage_dir.into();
        if !storage_dir.exists() {
            fs::create_dir_all(&storage_dir)?;
        }

        Ok(Self {
            build_dir,
            // it is important to canonicalize the path here because `is_data_path()` relies on the
            // fact that storage_dir is canonicalized.
            storage_dir: storage_dir.canonicalize()?,
        })
    }

    pub fn interface_files_dir(&self) -> Result<String> {
        let path = self.build_dir.join(MOVE_COMPILED_INTERFACES_DIR);
        if !path.exists() {
            fs::create_dir_all(&path)?;
        }
        Ok(path.into_os_string().into_string().unwrap())
    }

    pub(crate) fn interface_files_metadata_file(&self) -> PathBuf {
        // File containing interface file metadata, specifically named address mapping
        const INTERFACE_FILES_METADATA: &str = "metadata";
        let mut path = self.build_dir.join(MOVE_COMPILED_INTERFACES_DIR);
        path = path.join(INTERFACE_FILES_METADATA);
        path.set_extension("yaml");
        path
    }

    pub(crate) fn read_interface_files_metadata(&self) -> Result<InterfaceFilesMetadata> {
        let bytes_opt = Self::get_bytes(&self.interface_files_metadata_file())?;
        Ok(match bytes_opt {
            None => InterfaceFilesMetadata {
                named_address_mapping: BTreeMap::new(),
                named_address_values: BTreeMap::new(),
            },
            Some(bytes) => serde_yaml::from_slice::<InterfaceFilesMetadata>(&bytes)?,
        })
    }

    pub(crate) fn get_named_addresses(
        &self,
        additional_named_address_values: BTreeMap<String, AddressBytes>,
    ) -> Result<BTreeMap<String, AddressBytes>> {
        let mut save_named_addrs: BTreeMap<_, _> = self
            .read_interface_files_metadata()?
            .named_address_values
            .iter()
            .map(|(name, addr_str)| (name.clone(), AddressBytes::parse_str(addr_str).unwrap()))
            .collect();
        save_named_addrs.extend(additional_named_address_values);
        Ok(save_named_addrs)
    }

    fn update_interface_files_metadata(
        &self,
        additional_named_address_mapping: BTreeMap<ModuleId, Option<String>>,
        additional_named_address_values: BTreeMap<String, AddressBytes>,
    ) -> Result<()> {
        let InterfaceFilesMetadata {
            mut named_address_mapping,
            mut named_address_values,
        } = self.read_interface_files_metadata()?;
        for (id, address_name_opt) in additional_named_address_mapping {
            match address_name_opt {
                None => {
                    named_address_mapping.remove(&id);
                }
                Some(address_name) => {
                    named_address_mapping.insert(id, address_name);
                }
            }
        }
        named_address_values.extend(
            additional_named_address_values
                .into_iter()
                .map(|(name, addr)| (name, format!("0x{:#X}", addr))),
        );
        self.write_interface_files_metadata(InterfaceFilesMetadata {
            named_address_mapping,
            named_address_values,
        })
    }

    fn write_interface_files_metadata(&self, metadata: InterfaceFilesMetadata) -> Result<()> {
        let yaml_string = serde_yaml::to_string(&metadata).unwrap();
        let path = self.interface_files_metadata_file();
        if !path.exists() {
            fs::create_dir_all(path.parent().unwrap())?;
        }
        Ok(fs::write(path, yaml_string.as_bytes())?)
    }

    pub fn build_dir(&self) -> &PathBuf {
        &self.build_dir
    }

    fn is_data_path(&self, p: &Path, parent_dir: &str) -> bool {
        if !p.exists() {
            return false;
        }
        let p = p.canonicalize().unwrap();
        p.starts_with(&self.storage_dir)
            && match p.parent() {
                Some(parent) => parent.ends_with(parent_dir),
                None => false,
            }
    }

    pub fn is_resource_path(&self, p: &Path) -> bool {
        self.is_data_path(p, RESOURCES_DIR)
    }

    pub fn is_event_path(&self, p: &Path) -> bool {
        self.is_data_path(p, EVENTS_DIR)
    }

    pub fn is_module_path(&self, p: &Path) -> bool {
        self.is_data_path(p, MODULES_DIR)
    }

    fn get_addr_path(&self, addr: &AccountAddress) -> PathBuf {
        let mut path = self.storage_dir.clone();
        path.push(format!("0x{}", addr));
        path
    }

    fn get_resource_path(&self, addr: AccountAddress, tag: StructTag) -> PathBuf {
        let mut path = self.get_addr_path(&addr);
        path.push(RESOURCES_DIR);
        path.push(StructID(tag).to_string());
        path.with_extension(BCS_EXTENSION)
    }

    // Events are stored under address/handle creation number
    fn get_event_path(&self, key: &[u8]) -> PathBuf {
        // TODO: this is a hacky way to get the account address and creation number from the event key.
        // The root problem here is that the move-cli is using the Diem-specific event format.
        // We will deal this later when we make events more generic in the Move VM.
        let account_addr = AccountAddress::try_from(&key[8..])
            .expect("failed to get account address from event key");
        let creation_number = u64::from_le_bytes(key[..8].try_into().unwrap());
        let mut path = self.get_addr_path(&account_addr);
        path.push(EVENTS_DIR);
        path.push(creation_number.to_string());
        path.with_extension(BCS_EXTENSION)
    }

    fn get_module_path(&self, module_id: &ModuleId) -> PathBuf {
        let mut path = self.get_addr_path(module_id.address());
        path.push(MODULES_DIR);
        path.push(module_id.name().to_string());
        path.with_extension(MOVE_COMPILED_EXTENSION)
    }

    /// Read the resource bytes stored on-disk at `addr`/`tag`
    pub fn get_resource_bytes(
        &self,
        addr: AccountAddress,
        tag: StructTag,
    ) -> Result<Option<Vec<u8>>> {
        Self::get_bytes(&self.get_resource_path(addr, tag))
    }

    /// Read the resource bytes stored on-disk at `addr`/`tag`
    fn get_module_bytes(&self, module_id: &ModuleId) -> Result<Option<Vec<u8>>> {
        Self::get_bytes(&self.get_module_path(module_id))
    }

    /// Check if a module at `addr`/`module_id` exists
    pub fn has_module(&self, module_id: &ModuleId) -> bool {
        self.get_module_path(module_id).exists()
    }

    /// Deserialize and return the module stored on-disk at `addr`/`module_id`
    pub fn get_compiled_module(&self, module_id: &ModuleId) -> Result<CompiledModule> {
        CompiledModule::deserialize(
            &self
                .get_module_bytes(module_id)?
                .ok_or_else(|| anyhow!("Can't find {:?} on disk", module_id))?,
        )
        .map_err(|e| anyhow!("Failure deserializing module {:?}: {:?}", module_id, e))
    }

    /// Return the name of the function at `idx` in `module_id`
    pub fn resolve_function(&self, module_id: &ModuleId, idx: u16) -> Result<Identifier> {
        let m = self.get_compiled_module(module_id)?;
        Ok(m.identifier_at(
            m.function_handle_at(m.function_def_at(FunctionDefinitionIndex(idx)).function)
                .name,
        )
        .to_owned())
    }

    fn get_bytes(path: &Path) -> Result<Option<Vec<u8>>> {
        Ok(if path.exists() {
            Some(fs::read(path)?)
        } else {
            None
        })
    }

    /// Returns a deserialized representation of the resource value stored at `resource_path`.
    /// Returns Err if the path does not hold a resource value or the resource cannot be deserialized
    pub fn view_resource(&self, resource_path: &Path) -> Result<Option<AnnotatedMoveStruct>> {
        if resource_path.is_dir() {
            bail!("Bad resource path {:?}. Needed file, found directory")
        }
        match resource_path.file_stem() {
            None => bail!(
                "Bad resource path {:?}; last component must be a file",
                resource_path
            ),
            Some(name) => Ok({
                let id = match parser::parse_type_tag(&name.to_string_lossy())? {
                    TypeTag::Struct(s) => s,
                    t => bail!("Expected to parse struct tag, but got {}", t),
                };
                match Self::get_bytes(resource_path)? {
                    Some(resource_data) => {
                        Some(MoveValueAnnotator::new(self).view_resource(&id, &resource_data)?)
                    }
                    None => None,
                }
            }),
        }
    }

    fn get_events(&self, events_path: &Path) -> Result<Vec<Event>> {
        Ok(if events_path.exists() {
            match Self::get_bytes(events_path)? {
                Some(events_data) => bcs::from_bytes::<Vec<Event>>(&events_data)?,
                None => vec![],
            }
        } else {
            vec![]
        })
    }

    pub fn view_events(&self, events_path: &Path) -> Result<Vec<AnnotatedMoveValue>> {
        let annotator = MoveValueAnnotator::new(self);
        self.get_events(events_path)?
            .iter()
            .map(|(_, _, event_type, event_data)| annotator.view_value(event_type, event_data))
            .collect()
    }

    fn view_bytecode(path: &Path, is_module: bool) -> Result<Option<String>> {
        type Loc = u64;
        if path.is_dir() {
            bail!("Bad bytecode path {:?}. Needed file, found directory")
        }

        Ok(match Self::get_bytes(path)? {
            Some(bytes) => {
                let module: CompiledModule;
                let script: CompiledScript;
                let view = if is_module {
                    module = CompiledModule::deserialize(&bytes)
                        .map_err(|e| anyhow!("Failure deserializing module: {:?}", e))?;
                    BinaryIndexedView::Module(&module)
                } else {
                    script = CompiledScript::deserialize(&bytes)
                        .map_err(|e| anyhow!("Failure deserializing script: {:?}", e))?;
                    BinaryIndexedView::Script(&script)
                };
                // TODO: find or create source map and pass it to disassembler
                let d: Disassembler<Loc> = Disassembler::from_view(view, 0)?;
                Some(d.disassemble()?)
            }
            None => None,
        })
    }

    pub fn view_module(module_path: &Path) -> Result<Option<String>> {
        Self::view_bytecode(module_path, true)
    }

    pub fn view_script(script_path: &Path) -> Result<Option<String>> {
        Self::view_bytecode(script_path, false)
    }

    /// Delete resource stored on disk at the path `addr`/`tag`
    pub fn delete_resource(&self, addr: AccountAddress, tag: StructTag) -> Result<()> {
        let path = self.get_resource_path(addr, tag);
        fs::remove_file(path)?;

        // delete addr directory if this address is now empty
        let addr_path = self.get_addr_path(&addr);
        if addr_path.read_dir()?.next().is_none() {
            fs::remove_dir(addr_path)?
        }
        Ok(())
    }

    pub fn save_resource(
        &self,
        addr: AccountAddress,
        tag: StructTag,
        bcs_bytes: &[u8],
    ) -> Result<()> {
        let path = self.get_resource_path(addr, tag);
        if !path.exists() {
            fs::create_dir_all(path.parent().unwrap())?;
        }
        Ok(fs::write(path, bcs_bytes)?)
    }

    pub fn save_event(
        &self,
        event_key: &[u8],
        event_sequence_number: u64,
        event_type: TypeTag,
        event_data: Vec<u8>,
    ) -> Result<()> {
        // save event data in handle_address/EVENTS_DIR/handle_number
        let path = self.get_event_path(event_key);
        if !path.exists() {
            fs::create_dir_all(path.parent().unwrap())?;
        }
        // grab the old event log (if any) and append this event to it
        let mut event_log = self.get_events(&path)?;
        event_log.push((
            event_key.to_vec(),
            event_sequence_number,
            event_type,
            event_data,
        ));
        Ok(fs::write(path, &bcs::to_bytes(&event_log)?)?)
    }

    /// Save `module` on disk under the path `module.address()`/`module.name()`
    pub fn save_module(&self, module_id: &ModuleId, module_bytes: &[u8]) -> Result<()> {
        let path = self.get_module_path(module_id);
        if !path.exists() {
            fs::create_dir_all(path.parent().unwrap())?
        }
        Ok(fs::write(path, &module_bytes)?)
    }

    // keep the mv_interfaces generated in the build_dir in-sync with the modules on storage. The
    // mv_interfaces will be used for compilation and the modules will be used for linking.
    fn sync_interface_files(
        &self,
        named_address_mapping_changes: BTreeMap<ModuleId, Option<String>>,
        named_address_values: BTreeMap<String, AddressBytes>,
    ) -> Result<()> {
        self.update_interface_files_metadata(named_address_mapping_changes, named_address_values)?;
        move_lang::generate_interface_files(
            &[self
                .storage_dir
                .clone()
                .into_os_string()
                .into_string()
                .unwrap()],
            Some(
                self.build_dir
                    .clone()
                    .into_os_string()
                    .into_string()
                    .unwrap(),
            ),
            &self.read_interface_files_metadata()?.named_address_mapping,
            false,
        )?;
        Ok(())
    }

    /// Save all the modules in the local cache, re-generate mv_interfaces if required.
    pub fn save_modules<'a>(
        &self,
        modules: impl IntoIterator<Item = &'a (ModuleIdWithNamedAddress, Vec<u8>)>,
        named_address_values: BTreeMap<String, AddressBytes>,
    ) -> Result<()> {
        let mut named_address_mapping_changes = BTreeMap::new();
        let mut is_empty = true;
        for ((module_id, address_name_opt), module_bytes) in modules {
            self.save_module(module_id, module_bytes)?;
            named_address_mapping_changes.insert(module_id.clone(), address_name_opt.clone());
            is_empty = false;
        }

        // sync with build_dir for updates of mv_interfaces if new modules are added
        if !is_empty {
            self.sync_interface_files(named_address_mapping_changes, named_address_values)?;
        }

        Ok(())
    }

    pub fn delete_module(&self, id: &ModuleId) -> Result<()> {
        let path = self.get_module_path(id);
        fs::remove_file(path)?;

        // delete addr directory if this address is now empty
        let addr_path = self.get_addr_path(id.address());
        if addr_path.read_dir()?.next().is_none() {
            fs::remove_dir(addr_path)?
        }
        Ok(())
    }

    fn iter_paths<F>(&self, f: F) -> impl Iterator<Item = PathBuf>
    where
        F: FnOnce(&Path) -> bool + Copy,
    {
        walkdir::WalkDir::new(&self.storage_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .map(|e| e.path().to_path_buf())
            .filter(move |path| f(path))
    }

    pub fn resource_paths(&self) -> impl Iterator<Item = PathBuf> + '_ {
        self.iter_paths(move |p| self.is_resource_path(p))
    }

    pub fn module_paths(&self) -> impl Iterator<Item = PathBuf> + '_ {
        self.iter_paths(move |p| self.is_module_path(p))
    }

    pub fn event_paths(&self) -> impl Iterator<Item = PathBuf> + '_ {
        self.iter_paths(move |p| self.is_event_path(p))
    }

    /// Build all modules in the self.storage_dir.
    /// Returns an Err if a module does not deserialize.
    pub fn get_all_modules(&self) -> Result<Vec<CompiledModule>> {
        self.module_paths()
            .map(|path| {
                CompiledModule::deserialize(&Self::get_bytes(&path)?.unwrap())
                    .map_err(|e| anyhow!("Failed to deserialized module: {:?}", e))
            })
            .collect::<Result<Vec<CompiledModule>>>()
    }
}

impl ModuleResolver for OnDiskStateView {
    type Error = anyhow::Error;
    fn get_module(&self, module_id: &ModuleId) -> Result<Option<Vec<u8>>, Self::Error> {
        self.get_module_bytes(module_id)
    }
}

impl ResourceResolver for OnDiskStateView {
    type Error = anyhow::Error;

    fn get_resource(
        &self,
        address: &AccountAddress,
        struct_tag: &StructTag,
    ) -> Result<Option<Vec<u8>>, Self::Error> {
        self.get_resource_bytes(*address, struct_tag.clone())
    }
}

impl Default for OnDiskStateView {
    fn default() -> Self {
        OnDiskStateView::create(Path::new(DEFAULT_BUILD_DIR), Path::new(DEFAULT_STORAGE_DIR))
            .expect("Failure creating OnDiskStateView")
    }
}

// wrappers of TypeTag, StructTag, Vec<TypeTag> to allow us to implement the FromStr/ToString traits
#[derive(Debug)]
struct TypeID(TypeTag);
#[derive(Debug)]
struct StructID(StructTag);
#[derive(Debug)]
struct Generics(Vec<TypeTag>);

impl ToString for TypeID {
    fn to_string(&self) -> String {
        match &self.0 {
            TypeTag::Struct(s) => StructID(s.clone()).to_string(),
            TypeTag::Vector(t) => format!("vector<{}>", TypeID(*t.clone()).to_string()),
            t => t.to_string(),
        }
    }
}

impl ToString for StructID {
    fn to_string(&self) -> String {
        let tag = &self.0;
        // TODO: TypeTag parser insists on leading 0x for StructTag's, so we insert one here.
        // Would be nice to expose a StructTag parser and get rid of the 0x here
        format!(
            "0x{}::{}::{}{}",
            tag.address,
            tag.module,
            tag.name,
            Generics(tag.type_params.clone()).to_string()
        )
    }
}

impl ToString for Generics {
    fn to_string(&self) -> String {
        if self.0.is_empty() {
            "".to_string()
        } else {
            let generics: Vec<String> = self
                .0
                .iter()
                .map(|t| TypeID(t.clone()).to_string())
                .collect();
            format!("<{}>", generics.join(","))
        }
    }
}
