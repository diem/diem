// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::language_storage::{ModuleId, TypeTag};
use serde::{Deserialize, Serialize};

/// How to call a particular Move script (aka. an "ABI").
#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum ScriptABI {
    TransactionScript(TransactionScriptABI),
    ScriptFunction(ScriptFunctionABI),
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct ScriptFunctionABI {
    /// The public name of the script.
    name: String,
    /// The module name where the script lives.
    module_name: ModuleId,
    /// Some text comment.
    doc: String,
    /// The names of the type arguments.
    ty_args: Vec<TypeArgumentABI>,
    /// The description of regular arguments.
    args: Vec<ArgumentABI>,
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct TransactionScriptABI {
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

impl TransactionScriptABI {
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

impl ScriptFunctionABI {
    pub fn new(
        name: String,
        module_name: ModuleId,
        doc: String,
        ty_args: Vec<TypeArgumentABI>,
        args: Vec<ArgumentABI>,
    ) -> Self {
        Self {
            name,
            module_name,
            doc,
            ty_args,
            args,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn module_name(&self) -> &ModuleId {
        &self.module_name
    }

    pub fn doc(&self) -> &str {
        &self.doc
    }

    pub fn ty_args(&self) -> &[TypeArgumentABI] {
        &self.ty_args
    }

    pub fn args(&self) -> &[ArgumentABI] {
        &self.args
    }
}

impl ScriptABI {
    pub fn is_script_fun_abi(&self) -> bool {
        matches!(self, Self::ScriptFunction(_))
    }

    pub fn is_transaction_script_abi(&self) -> bool {
        matches!(self, Self::TransactionScript(_))
    }

    pub fn name(&self) -> &str {
        match self {
            Self::TransactionScript(abi) => abi.name(),
            Self::ScriptFunction(abi) => abi.name(),
        }
    }

    pub fn doc(&self) -> &str {
        match self {
            Self::TransactionScript(abi) => abi.doc(),
            Self::ScriptFunction(abi) => abi.doc(),
        }
    }

    pub fn ty_args(&self) -> &[TypeArgumentABI] {
        match self {
            Self::TransactionScript(abi) => abi.ty_args(),
            Self::ScriptFunction(abi) => abi.ty_args(),
        }
    }

    pub fn args(&self) -> &[ArgumentABI] {
        match self {
            Self::TransactionScript(abi) => abi.args(),
            Self::ScriptFunction(abi) => abi.args(),
        }
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
