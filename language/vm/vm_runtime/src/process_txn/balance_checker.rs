use std::{
    cell::RefCell,
    collections::HashMap,
    fmt::{Display, Formatter},
};

use libra_types::{
    access_path::AccessPath,
    account_config::coin_struct_tag,
    language_storage::{ModuleId, StructTag},
    vm_error::{StatusCode, VMStatus},
    write_set::{WriteOp, WriteSet},
};
use logger::prelude::*;
use vm::{
    access::ModuleAccess,
    errors::VMResult,
    file_format::{SignatureToken, StructFieldInformation},
    gas_schedule::{GasAlgebra, GasUnits},
    views::StructHandleView,
};
use vm_runtime_types::{
    loaded_data::struct_def::StructDef,
    value::{Struct, Value},
};

use crate::{
    code_cache::module_cache::ModuleCache, data_cache::RemoteCache, gas_meter::GasMeter,
    loaded_data::loaded_module::LoadedModule,
};

struct StructFieldScanner<'alloc, 'txn> {
    module_cache: &'txn dyn ModuleCache<'alloc>,
}

impl<'alloc, 'txn> StructFieldScanner<'alloc, 'txn> {
    pub fn new(module_cache: &'txn dyn ModuleCache<'alloc>) -> Self {
        Self { module_cache }
    }

    fn module_cache(&self) -> &dyn ModuleCache<'alloc> {
        return self.module_cache;
    }

    pub fn scan(
        &self,
        tag: &StructTag,
        value: &Struct,
        predicate: &dyn Fn(&StructTag, &Struct) -> bool,
    ) -> VMResult<Vec<(StructTag, Struct)>> {
        let mut resources = vec![];
        if predicate(tag, value) {
            resources.push((tag.clone(), value.clone()));
        } else {
            self.scan_struct_field(tag, value, &mut resources, predicate)?;
        }
        Ok(resources)
    }

    fn load_module_by_tag(&self, tag: &StructTag) -> VMResult<&LoadedModule> {
        self.module_cache
            .get_loaded_module(&ModuleId::new(tag.address, tag.module.clone()))?
            .ok_or(
                VMStatus::new(StatusCode::MISSING_DATA)
                    .with_message(format!("load module by tag: {:?} fail.", tag)),
            )
    }

    fn scan_struct_field(
        &self,
        tag: &StructTag,
        value: &Struct,
        resources: &mut Vec<(StructTag, Struct)>,
        predicate: &dyn Fn(&StructTag, &Struct) -> bool,
    ) -> VMResult<()> {
        let module = self.load_module_by_tag(tag)?;
        let struct_def_idx = module
            .struct_defs_table
            .get(&tag.name)
            .ok_or(VMStatus::new(StatusCode::LINKER_ERROR))?;
        let struct_def = module.struct_def_at(*struct_def_idx);
        //let struct_handle = module.struct_handle_at(struct_def.struct_handle);
        match &struct_def.field_information {
            StructFieldInformation::Declared {
                field_count,
                fields,
            } => {
                for (idx, field) in module
                    .field_def_range(*field_count, *fields)
                    .iter()
                    .enumerate()
                {
                    let field_signature = &module.type_signature_at(field.signature).0;
                    let field_value = value.get_field_value(idx)?;
                    match field_signature {
                        SignatureToken::Struct(idx, _) => {
                            let struct_handle = module.struct_handle_at(*idx);
                            let struct_name = module.identifier_at(struct_handle.name);
                            let field_type_module_id =
                                StructHandleView::new(module, struct_handle).module_id();
                            let field_module = self
                                .module_cache
                                .get_loaded_module(&field_type_module_id)?
                                .ok_or(VMStatus::new(StatusCode::MISSING_DATA).with_message(
                                    format!("get module by id: {:?} fail.", field_type_module_id),
                                ))?;
                            let struct_value: Struct = match field_value.into() {
                                Some(s) => s,
                                None => {
                                    //TODO(jole) support native struct, such as Vector.
                                    return Ok(());
                                }
                            };

                            let field_struct_tag = StructTag {
                                name: struct_name.to_owned(),
                                address: *field_module.address(),
                                module: field_module.name().to_owned(),
                                type_params: vec![],
                            };
                            if predicate(&field_struct_tag, &struct_value) {
                                resources.push((field_struct_tag, struct_value))
                            } else {
                                self.scan_struct_field(
                                    &field_struct_tag,
                                    &struct_value,
                                    resources,
                                    predicate,
                                )?;
                            }
                        }
                        _ => {
                            //continue
                        }
                    }
                }
            }
            StructFieldInformation::Native => {
                //TODO impl native support
            }
        }

        Ok(())
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
struct AssetCollector {
    assets: RefCell<HashMap<StructTag, u64>>,
}

impl AssetCollector {
    pub fn new() -> Self {
        Self {
            assets: RefCell::new(HashMap::new()),
        }
    }

    pub fn incr(&self, tag: &StructTag, incr: u64) {
        let mut asserts = self.assets.borrow_mut();
        if asserts.contains_key(tag) {
            let old_balance = asserts.get_mut(tag).unwrap();
            let new_balance = *old_balance + incr;
            asserts.insert(tag.clone(), new_balance);
        } else {
            asserts.insert(tag.clone(), incr);
        }
    }

    pub fn is_asset(tag: &StructTag) -> bool {
        //TODO(jole) implement asset type tag or register.
        return tag == &coin_struct_tag();
    }

    pub fn asset_balance(value: &Struct) -> VMResult<u64> {
        let balance = value
            .get_field_value(0)?
            .value_as()
            .expect("asset first field must be balance.");
        Ok(balance)
    }

    pub fn collect(
        &self,
        scanner: &StructFieldScanner,
        tag: &StructTag,
        resource: &Struct,
    ) -> VMResult<()> {
        let assets = scanner.scan(tag, resource, &|tag, _resource| -> bool {
            Self::is_asset(tag)
        })?;
        for (tag, value) in assets {
            self.incr(&tag, Self::asset_balance(&value)?)
        }
        Ok(())
    }
}

impl Display for AssetCollector {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let assets = self.assets.borrow();
        for (k, v) in assets.iter() {
            writeln!(f, "{}: {}", k.module.as_str(), v)?;
        }
        Ok(())
    }
}

pub struct BalanceChecker<'alloc, 'txn> {
    data_cache: &'txn dyn RemoteCache,
    scanner: StructFieldScanner<'alloc, 'txn>,
}

impl<'alloc, 'txn> BalanceChecker<'alloc, 'txn> {
    pub fn new(
        data_cache: &'txn dyn RemoteCache,
        module_cache: &'txn dyn ModuleCache<'alloc>,
    ) -> Self {
        Self {
            data_cache,
            scanner: StructFieldScanner::new(module_cache),
        }
    }

    fn get_resource(&self, ap: &AccessPath) -> VMResult<Option<(StructTag, Struct)>> {
        let tag = ap
            .resource_tag()
            .expect("this access_path must contains resource tag.");
        let value = self.data_cache.get(ap).and_then(|value| match value {
            Some(value) => Ok(Some(self.deserialize_struct(&tag, value.as_slice())?)),
            None => Ok(None),
        })?;
        Ok(value.map(|struct_value| (tag, struct_value)))
    }

    fn deserialize_struct(&self, tag: &StructTag, value: &[u8]) -> VMResult<Struct> {
        let def = self.resolve(&tag)?;
        let struct_value = Value::simple_deserialize(value, def)?
            .value_as()
            .expect("value must be struct");
        Ok(struct_value)
    }

    fn resolve(&self, tag: &StructTag) -> VMResult<StructDef> {
        let mut gas = GasMeter::new(GasUnits::new(1));
        gas.disable_metering();
        let module = self.scanner.load_module_by_tag(tag)?;
        let struct_def_idx = module
            .struct_defs_table
            .get(&tag.name)
            .ok_or(VMStatus::new(StatusCode::LINKER_ERROR))?;
        self.scanner
            .module_cache()
            .resolve_struct_def(module, *struct_def_idx, &gas)?
            .ok_or(
                VMStatus::new(StatusCode::MISSING_DATA)
                    .with_message(format!("resolve StructDef by tag: {:?} fail.", tag)),
            )
    }

    pub fn check_balance(&self, write_set: &WriteSet) -> Result<(), VMStatus> {
        if write_set.len() == 0 {
            return Ok(());
        }
        debug!("check_balance write_set: {}", write_set.len());
        let mut old_resources = vec![];
        let mut new_resources = vec![];
        for (access_path, op) in write_set {
            match op {
                WriteOp::Deletion => {
                    if !access_path.is_code() {
                        // if old resource not exist, it possible create at offchain.
                        if let Some(old_resource) = self.get_resource(access_path)? {
                            old_resources.push(old_resource);
                        } else {
                            debug!("Can not find old resource by access_path: {}", access_path);
                        }
                    }
                }
                WriteOp::Value(value) => {
                    if !access_path.is_code() {
                        let tag = access_path
                            .resource_tag()
                            .expect("this access_path must contains resource tag.");
                        let struct_value = self.deserialize_struct(&tag, value.as_slice())?;
                        new_resources.push((tag, struct_value));
                        if let Some(old_resource) = self.get_resource(access_path)? {
                            old_resources.push(old_resource);
                        }
                    }
                }
            }
        }
        let old_assets = AssetCollector::new();
        let new_assets = AssetCollector::new();
        for (tag, value) in old_resources {
            old_assets.collect(&self.scanner, &tag, &value)?;
        }
        for (tag, value) in new_resources {
            new_assets.collect(&self.scanner, &tag, &value)?;
        }
        debug!("old assets: {}", old_assets);
        debug!("new assets: {}", new_assets);
        if old_assets != new_assets {
            let mut err = VMStatus::new(StatusCode::INVALID_DATA);
            err.set_message(format!(
                "old assets {:?} and new assets {:?} is not equals.",
                old_assets, new_assets
            ));
            return Err(err);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_balance_check() {}
}
