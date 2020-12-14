// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::FuzzTargetImpl;
use anyhow::{bail, format_err, Result};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use diem_proptest_helpers::ValueGenerator;
use move_core_types::value::{MoveKindInfo, MoveTypeLayout};
use move_vm_types::values::{prop::layout_kinfo_and_value_strategy, Value};
use std::io::Cursor;

#[derive(Clone, Debug, Default)]
pub struct ValueTarget;

impl FuzzTargetImpl for ValueTarget {
    fn description(&self) -> &'static str {
        "VM values + types (custom deserializer)"
    }

    fn generate(&self, _idx: usize, gen: &mut ValueGenerator) -> Option<Vec<u8>> {
        let (layout, kinfo, value) = gen.generate(layout_kinfo_and_value_strategy());

        // Values as currently serialized are not self-describing, so store a serialized form of the
        // layout + kind info along with the value as well.
        let layout_blob = bcs::to_bytes(&layout).unwrap();
        let kinfo_blob = bcs::to_bytes(&kinfo).unwrap();
        let value_blob = value.simple_serialize(&layout).expect("must serialize");

        let mut blob = vec![];
        // Prefix the layout blob with its length.
        blob.write_u64::<BigEndian>(layout_blob.len() as u64)
            .expect("writing should work");
        blob.write_u64::<BigEndian>(kinfo_blob.len() as u64)
            .expect("writing should work");
        blob.extend_from_slice(&layout_blob);
        blob.extend_from_slice(&kinfo_blob);
        blob.extend_from_slice(&value_blob);
        Some(blob)
    }

    fn fuzz(&self, data: &[u8]) {
        let _ = deserialize(data);
    }
}

fn is_valid_layout(layout: &MoveTypeLayout) -> bool {
    use MoveTypeLayout as L;

    match layout {
        L::Bool | L::U8 | L::U64 | L::U128 | L::Address | L::Signer => true,

        L::Vector(layout) => is_valid_layout(layout),

        L::Struct(struct_layout) => {
            if struct_layout.fields().is_empty() {
                return false;
            }
            struct_layout.fields().iter().all(is_valid_layout)
        }
    }
}

fn is_valid_kinfo(kinfo: &MoveKindInfo) -> bool {
    use MoveKindInfo as K;

    match kinfo {
        K::Base(_) => true,
        K::Vector(_, inner) => is_valid_kinfo(inner),
        K::Struct(_, field_info) => !field_info.is_empty() && field_info.iter().all(is_valid_kinfo),
    }
}

fn deserialize(data: &[u8]) -> Result<()> {
    let mut data = Cursor::new(data);
    // Read the length of the layout blob.
    let layout_len = data.read_u64::<BigEndian>()? as usize;
    let kinfo_len = data.read_u64::<BigEndian>()? as usize;
    let position = data.position() as usize;
    let data = &data.into_inner()[position..];

    let n = layout_len
        .checked_add(kinfo_len)
        .ok_or_else(|| format_err!("bad sizes"))?;
    if data.len() < n {
        bail!("too little data");
    }
    let layout_data = &data[..layout_len];
    let kinfo_data = &data[layout_len..n];
    let value_data = &data[n..];

    let layout: MoveTypeLayout = bcs::from_bytes(layout_data)?;
    let kinfo: MoveKindInfo = bcs::from_bytes(kinfo_data)?;

    // The fuzzer may alter the raw bytes, resulting in invalid layouts that will not
    // pass the bytecode verifier. We need to filter these out as they can show up as
    // false positives.
    if !is_valid_layout(&layout) {
        bail!("bad layout")
    }

    if !is_valid_kinfo(&kinfo) {
        bail!("bad kinfo")
    }

    let _ = Value::simple_deserialize(value_data, &kinfo, &layout);
    Ok(())
}
