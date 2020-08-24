// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use disassembler::disassembler::Disassembler;
use move_core_types::{
    account_address::AccountAddress,
    identifier::Identifier,
    language_storage::{ModuleId, StructTag, TypeTag},
    parser,
    value::MoveTypeLayout,
    vm_status::StatusCode,
};
use move_vm_runtime::data_cache::RemoteCache;
use move_vm_types::values::Value;
use resource_viewer::{AnnotatedMoveStruct, MoveValueAnnotator};
use vm::{
    access::ModuleAccess,
    errors::*,
    file_format::{CompiledModule, FunctionDefinitionIndex},
};

use anyhow::{anyhow, bail, Result};
use std::{
    collections::HashMap,
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub struct OnDiskStateView {
    modules: HashMap<ModuleId, Vec<u8>>,
    resources: HashMap<(AccountAddress, StructTag), Vec<u8>>,
    move_data_dir: PathBuf,
}

impl OnDiskStateView {
    /// Create an `OnDiskStateView` that reads/writes resource data in `move_data_dir` and can
    /// execute code in `compiled_modules`.
    pub fn create(move_data_dir: PathBuf, compiled_modules: &[CompiledModule]) -> Result<Self> {
        if !move_data_dir.exists() || !move_data_dir.is_dir() {
            bail!(
                "Attempting to create OnDiskStateView from non-directory {:?}",
                move_data_dir
            )
        }

        let mut modules = HashMap::with_capacity(compiled_modules.len());
        for module in compiled_modules {
            let mut module_bytes = vec![];
            module.serialize(&mut module_bytes)?;
            modules.insert(module.self_id(), module_bytes);
        }
        let resources = HashMap::new();
        Ok(Self {
            modules,
            resources,
            move_data_dir,
        })
    }

    fn get_addr_path(&self, addr: &AccountAddress) -> PathBuf {
        let mut path = self.move_data_dir.clone();
        path.push(format!("0x{}", addr.to_string()));
        path
    }

    fn get_resource_path(&self, addr: AccountAddress, tag: StructTag) -> PathBuf {
        let mut path = self.get_addr_path(&addr);
        path.push(StructID(tag).to_string());
        path
    }

    fn get_module_path(&self, module_id: &ModuleId) -> PathBuf {
        let mut path = self.get_addr_path(module_id.address());
        path.push(module_id.name().to_string());
        path
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
            let mut bytes = vec![];
            let mut f = fs::File::open(path)?;
            f.read_to_end(&mut bytes)?;
            Some(bytes)
        } else {
            None
        })
    }

    pub fn view_resource(&self, resource_path: &Path) -> Result<Option<AnnotatedMoveStruct>> {
        if resource_path.is_dir() {
            bail!("Bad resource path {:?}. Needed file, found directory")
        }
        match resource_path.file_name() {
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
                    Some(resource_data) => Some(
                        MoveValueAnnotator::new_no_stdlib(self)
                            .view_resource(&id, &resource_data)?,
                    ),
                    None => None,
                }
            }),
        }
    }

    pub fn view_module(&self, module_path: &Path) -> Result<Option<String>> {
        type Loc = u64;
        if module_path.is_dir() {
            bail!("Bad module path {:?}. Needed file, found directory")
        }

        Ok(match Self::get_bytes(module_path)? {
            Some(module_bytes) => {
                // TODO: find or create source map and pass it to disassembler
                let d: Disassembler<Loc> = Disassembler::from_module(
                    CompiledModule::deserialize(&module_bytes)
                        .map_err(|e| anyhow!("Failure deserializing module: {:?}", e))?,
                    0,
                )?;
                Some(d.disassemble()?)
            }
            None => None,
        })
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

    /// Save `resource` on disk under the path `addr`/`tag`
    pub fn save_resource(
        &self,
        addr: AccountAddress,
        tag: StructTag,
        layout: MoveTypeLayout,
        resource: Value,
    ) -> Result<()> {
        let mut path = self.get_addr_path(&addr);
        if !path.exists() {
            fs::create_dir(path.clone())?
        }
        path.push(StructID(tag).to_string());
        let lcs = resource
            .simple_serialize(&layout)
            .ok_or_else(|| anyhow!("Failed to serialize resource"))?;
        let mut f = fs::File::create(path)?;
        Ok(f.write_all(&lcs)?)
    }

    /// Save `module` on disk under the path `module.address()`/`module.name()`
    fn save_module(&self, module_id: &ModuleId, module_bytes: &[u8]) -> Result<()> {
        let mut path = self.get_addr_path(module_id.address());
        if !path.exists() {
            fs::create_dir(path.clone())?
        }
        path.push(module_id.name().to_string());
        let mut f = fs::File::create(path)?;
        Ok(f.write_all(&module_bytes)?)
    }

    /// Save all the modules in the local cache
    pub fn save_modules(&self) -> Result<()> {
        for (id, bytes) in &self.modules {
            self.save_module(id, &bytes)?
        }
        Ok(())
    }
}

impl RemoteCache for OnDiskStateView {
    fn get_module(&self, module_id: &ModuleId) -> VMResult<Option<Vec<u8>>> {
        match self.modules.get(module_id) {
            None => self.get_module_bytes(module_id).map_err(|_| {
                PartialVMError::new(StatusCode::STORAGE_ERROR).finish(Location::Undefined)
            }),
            m => Ok(m.cloned()),
        }
    }

    fn get_resource(
        &self,
        address: &AccountAddress,
        struct_tag: &StructTag,
    ) -> PartialVMResult<Option<Vec<u8>>> {
        match self.resources.get(&(*address, struct_tag.clone())) {
            None => self
                .get_resource_bytes(*address, struct_tag.clone())
                .map_err(|_| PartialVMError::new(StatusCode::STORAGE_ERROR)),
            res => Ok(res.cloned()),
        }
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
        // TODO: TypeTag parser insists on leading 0x for StructTag's, so we insert one here. Would be nice to
        // expose a StructTag parser and get rid of the 0x here
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
