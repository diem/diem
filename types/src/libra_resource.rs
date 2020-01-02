// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::language_storage::StructTag;
use anyhow::Result;
use serde::Deserialize;

pub trait LibraResource {
    fn struct_tag() -> StructTag;
}

pub fn make_resource<'a, T>(value: &'a [u8]) -> Result<T>
where
    T: LibraResource + Deserialize<'a>,
{
    lcs::from_bytes(value).map_err(|e| Into::into(e))
}
