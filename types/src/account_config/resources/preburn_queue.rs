// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    access_path::AccessPath,
    account_config::{
        constants::{xus_tag, CORE_CODE_ADDRESS, DIEM_MODULE_IDENTIFIER},
        resources::PreburnWithMetadataResource,
    },
};
use move_core_types::{
    ident_str,
    identifier::IdentStr,
    language_storage::{StructTag, TypeTag},
    move_resource::{MoveResource, MoveStructType},
};
#[cfg(any(test, feature = "fuzzing"))]
use proptest_derive::Arbitrary;
use serde::{Deserialize, Serialize};

/// The preburn balance held under an account.
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
pub struct PreburnQueueResource {
    preburns: Vec<PreburnWithMetadataResource>,
}

impl PreburnQueueResource {
    pub fn preburns(&self) -> &[PreburnWithMetadataResource] {
        &self.preburns
    }

    // TODO/XXX: remove this once the MoveResource trait allows type arguments to `struct_tag`.
    pub fn struct_tag_for_currency(currency_typetag: TypeTag) -> StructTag {
        StructTag {
            address: CORE_CODE_ADDRESS,
            name: PreburnQueueResource::struct_identifier(),
            module: PreburnQueueResource::module_identifier(),
            type_params: vec![currency_typetag],
        }
    }

    // TODO: remove this once the MoveResource trait allows type arguments to `resource_path`.
    pub fn access_path_for(currency_typetag: TypeTag) -> Vec<u8> {
        AccessPath::resource_access_vec(PreburnQueueResource::struct_tag_for_currency(
            currency_typetag,
        ))
    }
}

impl MoveStructType for PreburnQueueResource {
    const MODULE_NAME: &'static IdentStr = DIEM_MODULE_IDENTIFIER;
    const STRUCT_NAME: &'static IdentStr = ident_str!("PreburnQueue");

    fn type_params() -> Vec<TypeTag> {
        vec![xus_tag()]
    }
}

impl MoveResource for PreburnQueueResource {}
