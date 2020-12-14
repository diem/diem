// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use serde_reflection::{ContainerFormat, Error, Format, FormatHolder, Result};

/// Verify that a Serde format is compatible with BCS and follows best practices.
pub fn lint_bcs_format(format: &ContainerFormat) -> Result<()> {
    if is_empty_container(format) {
        return Err(Error::Custom("Please avoid 0-sized containers".into()));
    }
    format.visit(&mut |f| {
        use Format::*;
        match f {
            F32 | F64 | Char => Err(Error::Custom(format!("BCS does not support type {:?}", f))),
            Seq(inner) => match inner.as_ref() {
                U8 => Err(Error::Custom(
                    "Please use `#[serde(with = \"serde_bytes\")` on `Vec<u8>` objects.".into(),
                )),
                e if is_empty(e) => Err(Error::Custom(format!(
                    "Sequences with empty content are not allowed: {:?}",
                    f
                ))),
                _ => Ok(()),
            },
            Map { key, value } => {
                if is_empty(key) && is_empty(value) {
                    Err(Error::Custom(format!(
                        "Maps with empty keys and values are not allowed: {:?}",
                        f
                    )))
                } else {
                    Ok(())
                }
            }
            _ => Ok(()),
        }
    })
}

fn is_empty(format: &Format) -> bool {
    use Format::*;
    match format {
        Unit => true,
        TupleArray { content, size } => *size == 0 || is_empty(content.as_ref()),
        Tuple(formats) => formats.iter().all(is_empty),
        _ => false,
    }
}

fn is_empty_container(format: &ContainerFormat) -> bool {
    use ContainerFormat::*;
    match format {
        UnitStruct => true,
        NewTypeStruct(inner) => is_empty(inner.as_ref()),
        TupleStruct(formats) => formats.iter().all(is_empty),
        Struct(formats) => formats.iter().map(|named| &named.value).all(is_empty),
        Enum(_) => false,
    }
}
