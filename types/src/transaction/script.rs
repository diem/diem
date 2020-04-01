// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{language_storage::TypeTag, transaction::transaction_argument::TransactionArgument};
use serde::{Deserialize, Serialize};
use std::fmt;

#[allow(dead_code)]
pub const SCRIPT_HASH_LENGTH: usize = 32;

#[derive(Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct Script {
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
            .field("args", &self.args)
            .finish()
    }
}
