// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This is a temporary shim to allow us to get the names of new scripts until `public(script)
//! fun`'s are in. Once those are in this should be removed.

use crate::legacy::transaction_scripts::LegacyStdlibScript;
use anyhow::{anyhow, Error, Result};
use diem_crypto::HashValue;
use diem_types::transaction::ScriptABI;
use include_dir::{include_dir, Dir};
use std::{convert::TryFrom, fmt, path::PathBuf};

// This includes the script ABIs as binaries. We must use this hack to work around
// a problem with Docker, which does not copy over the Move source files that would be be used to
// produce these binaries at runtime.
const TXN_SCRIPTS_ABI_DIR: Dir = include_dir!("transaction_scripts/abi");

/// All of the Move transaction scripts that can be executed on the Diem blockchain
#[derive(Clone, Copy, Eq, PartialEq)]
enum TmpStdlibScript {
    BurnWithAmount,
    CancelBurnWithAmount,
}

impl TmpStdlibScript {
    /// Return a vector containing all of the standard library scripts (i.e., all inhabitants of the
    /// StdlibScript enum)
    pub(crate) fn all() -> Vec<Self> {
        use TmpStdlibScript::*;
        vec![BurnWithAmount, CancelBurnWithAmount]
    }

    /// Return a lowercase-underscore style name for this script
    pub(crate) fn name(self) -> String {
        self.to_string()
    }

    /// Return the Move bytecode that was produced by compiling this script.
    pub(crate) fn compiled_bytes(self) -> CompiledBytes {
        CompiledBytes(self.abi().code().to_vec())
    }

    /// Return the ABI of the script (including the bytecode).
    pub(crate) fn abi(self) -> ScriptABI {
        let mut path = PathBuf::from(self.name());
        path.set_extension("abi");
        let content = TXN_SCRIPTS_ABI_DIR
            .get_file(path.clone())
            .unwrap_or_else(|| panic!("File {:?} does not exist", path))
            .contents();
        bcs::from_bytes(content)
            .unwrap_or_else(|err| panic!("Failed to deserialize ABI file {:?}: {}", path, err))
    }

    /// Return the sha3-256 hash of the compiled script bytes.
    pub(crate) fn hash(self) -> HashValue {
        self.compiled_bytes().hash()
    }
}

/// Bytes produced by compiling a Move source language script into Move bytecode
#[derive(Clone)]
struct CompiledBytes(Vec<u8>);

impl CompiledBytes {
    /// Return the sha3-256 hash of the script bytes
    pub(crate) fn hash(&self) -> HashValue {
        Self::hash_bytes(&self.0)
    }

    /// Return the sha3-256 hash of the script bytes
    fn hash_bytes(bytes: &[u8]) -> HashValue {
        HashValue::sha3_256_of(bytes)
    }
}

impl TryFrom<&[u8]> for TmpStdlibScript {
    type Error = Error;

    /// Return `Some(<script_name>)` if  `code_bytes` is the bytecode of one of the standard library
    /// scripts, None otherwise.
    fn try_from(code_bytes: &[u8]) -> Result<Self> {
        let hash = CompiledBytes::hash_bytes(code_bytes);
        Self::all()
            .iter()
            .find(|script| script.hash() == hash)
            .cloned()
            .ok_or_else(|| anyhow!("Could not create standard library script from bytes"))
    }
}

impl fmt::Display for TmpStdlibScript {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use TmpStdlibScript::*;
        write!(
            f,
            "{}",
            match self {
                BurnWithAmount => "burn_with_amount",
                CancelBurnWithAmount => "cancel_burn_with_amount",
            }
        )
    }
}

// This is the only public interface of this file that should be used.
pub fn name_for_script(bytes: &[u8]) -> Result<String> {
    if let Ok(script) = LegacyStdlibScript::try_from(bytes) {
        Ok(format!("{}", script))
    } else {
        TmpStdlibScript::try_from(bytes).map(|script| format!("{}", script))
    }
}
