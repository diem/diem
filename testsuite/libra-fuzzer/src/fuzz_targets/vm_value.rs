// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::FuzzTargetImpl;
use anyhow::{bail, Result};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use libra_proptest_helpers::ValueGenerator;
use std::io::Cursor;
use vm_runtime_types::{
    loaded_data::{struct_def::StructDef, types::Type},
    value::Value,
};

#[derive(Clone, Debug, Default)]
pub struct ValueTarget;

impl FuzzTargetImpl for ValueTarget {
    fn name(&self) -> &'static str {
        module_name!()
    }

    fn description(&self) -> &'static str {
        "VM values + types (custom deserializer)"
    }

    fn generate(&self, _idx: usize, gen: &mut ValueGenerator) -> Option<Vec<u8>> {
        let value = gen.generate(Value::struct_strategy());
        let struct_def = match value.to_type_FOR_TESTING() {
            Type::Struct(def) => def,
            _ => unreachable!("must have returned a StructDef"),
        };

        // Values as currently serialized are not self-describing, so store a serialized form of the
        // type along with the value as well.
        let struct_def_blob = lcs::to_bytes(&struct_def).unwrap();

        let value_blob = value.simple_serialize().expect("must serialize");
        let mut blob = vec![];
        // Prefix the struct def blob with its length.
        blob.write_u64::<BigEndian>(struct_def_blob.len() as u64)
            .expect("writing should work");
        blob.extend_from_slice(&struct_def_blob);
        blob.extend_from_slice(&value_blob);
        Some(blob)
    }

    fn fuzz(&self, data: &[u8]) {
        let _ = deserialize(data);
    }
}

fn deserialize(data: &[u8]) -> Result<()> {
    let mut data = Cursor::new(data);
    // Read the length of the struct def blob.
    let len = data.read_u64::<BigEndian>()? as usize;
    let position = data.position() as usize;
    let data = &data.into_inner()[position..];

    if data.len() < len {
        bail!("too little data");
    }
    let struct_def_data = &data[0..len];
    let value_data = &data[len..];

    let struct_def: StructDef = lcs::from_bytes(struct_def_data)?;
    let _ = Value::simple_deserialize(value_data, struct_def);
    Ok(())
}
