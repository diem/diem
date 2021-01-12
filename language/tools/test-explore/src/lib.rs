// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};
use std::fmt;

use diem_types::transaction::TransactionArgument;
use move_core_types::{account_address::AccountAddress, language_storage::TypeTag};

// script invocation package
#[derive(Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
struct ScriptExecPack {
    sender: AccountAddress,
    ty_args: Vec<TypeTag>,
    args: Vec<TransactionArgument>,
}

impl fmt::Display for ScriptExecPack {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} [{}] ({}))",
            self.sender,
            self.ty_args
                .iter()
                .map(|tag| tag.to_string())
                .collect::<Vec<_>>()
                .join(", "),
            self.args
                .iter()
                .map(|arg| format!("{:?}", arg))
                .collect::<Vec<_>>()
                .join(", "),
        )
    }
}

pub mod analyzer_e2e;
