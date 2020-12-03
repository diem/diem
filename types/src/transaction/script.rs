// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::transaction::transaction_argument::TransactionArgument;
use move_core_types::language_storage::TypeTag;
use serde::{Deserialize, Serialize};
use std::fmt;

#[allow(dead_code)]
pub const SCRIPT_HASH_LENGTH: usize = 32;

/// Call a Move script.
#[derive(Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct Script {
    #[serde(with = "serde_bytes")]
    code: Vec<u8>,
    ty_args: Vec<TypeTag>,
    args: Vec<TransactionArgument>,
}

impl Script {
    pub fn new(code: Vec<u8>, ty_args: Vec<TypeTag>, args: Vec<TransactionArgument>) -> Self {
        Script {
            code,
            ty_args,
            args,
        }
    }

    pub fn code(&self) -> &[u8] {
        &self.code
    }

    pub fn ty_args(&self) -> &[TypeTag] {
        &self.ty_args
    }

    pub fn args(&self) -> &[TransactionArgument] {
        &self.args
    }

    pub fn into_inner(self) -> (Vec<u8>, Vec<TransactionArgument>) {
        (self.code, self.args)
    }
}

impl fmt::Debug for Script {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Script")
            .field("code", &hex::encode(&self.code))
            .field("ty_args", &self.ty_args)
            .field("args", &self.args)
            .finish()
    }
}

/// How to call a particular Move script (aka. an "ABI").
#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct ScriptABI {
    /// The public name of the script.
    name: String,
    /// Some text comment.
    doc: String,
    /// The `code` value to set in the `Script` object.
    #[serde(with = "serde_bytes")]
    code: Vec<u8>,
    /// The names of the type arguments.
    ty_args: Vec<TypeArgumentABI>,
    /// The description of regular arguments.
    args: Vec<ArgumentABI>,
}

/// The description of a (regular) argument in a script.
#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct ArgumentABI {
    /// The name of the argument.
    name: String,
    /// The expected type.
    /// In Move scripts, this does contain generics type parameters.
    type_tag: TypeTag,
}

/// The description of a type argument in a script.
#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct TypeArgumentABI {
    /// The name of the argument.
    name: String,
}

impl ScriptABI {
    pub fn new(
        name: String,
        doc: String,
        code: Vec<u8>,
        ty_args: Vec<TypeArgumentABI>,
        args: Vec<ArgumentABI>,
    ) -> Self {
        Self {
            name,
            doc,
            code,
            ty_args,
            args,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn doc(&self) -> &str {
        &self.doc
    }

    pub fn code(&self) -> &[u8] {
        &self.code
    }

    pub fn ty_args(&self) -> &[TypeArgumentABI] {
        &self.ty_args
    }

    pub fn args(&self) -> &[ArgumentABI] {
        &self.args
    }
}

impl ArgumentABI {
    pub fn new(name: String, type_tag: TypeTag) -> Self {
        Self { name, type_tag }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn type_tag(&self) -> &TypeTag {
        &self.type_tag
    }
}

impl TypeArgumentABI {
    pub fn new(name: String) -> Self {
        Self { name }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}
