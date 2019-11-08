// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::account_address::AccountAddress;
use crate::identifier::{IdentStr, Identifier};
use crate::language_storage::ModuleId;
use crate::transaction::parse_as_transaction_argument;
use crate::transaction::transaction_argument::TransactionArgument;
use failure::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Debug)]
pub struct ScriptAction {
    module: ModuleId,
    function: Identifier,
    args: Vec<TransactionArgument>,
}

impl ScriptAction {
    pub fn new(module: ModuleId, function: Identifier, args: Vec<TransactionArgument>) -> Self {
        Self {
            module,
            function,
            args,
        }
    }

    pub fn parse(action: &str) -> Result<Self> {
        let parts: Vec<&str> = action.trim().split(' ').collect();
        let call_parts: Vec<&str> = parts[0].trim().split('.').collect();
        ensure!(
            call_parts.len() == 3,
            "expect action format: address.ModuleName.FunctionName arg1 arg2 ..  but got: {}",
            action
        );
        let address = AccountAddress::from_hex_literal(call_parts[0])?;
        let module_name = IdentStr::new(call_parts[1])?;
        let function_name = IdentStr::new(call_parts[2])?;
        let args_result: Result<Vec<TransactionArgument>> = if parts.len() > 1 {
            parts[1..]
                .iter()
                .map(|str| parse_as_transaction_argument(str))
                .collect()
        } else {
            Ok(vec![])
        };
        Ok(Self {
            module: ModuleId::new(address, module_name.to_owned()),
            function: function_name.to_owned(),
            args: args_result?,
        })
    }

    pub fn module(&self) -> &ModuleId {
        &self.module
    }

    pub fn function(&self) -> &IdentStr {
        &self.function
    }

    pub fn args(&self) -> &[TransactionArgument] {
        &self.args
    }

    pub fn into_inner(self) -> (ModuleId, Identifier, Vec<TransactionArgument>) {
        (self.module, self.function, self.args)
    }
}

impl std::fmt::Display for ScriptAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}.{}.{}",
            self.module.address(),
            self.module.name(),
            self.function
        )?;
        for arg in &self.args {
            write!(f, " ")?;
            write!(f, "{}", arg)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test_script_action {
    use super::*;
    use crate::byte_array::ByteArray;

    fn ident(name: impl Into<Box<str>>) -> Identifier {
        Identifier::new(name).unwrap()
    }

    #[test]
    fn test_action_parse() {
        let test_pairs: Vec<(ScriptAction, &str)> = vec![
            (
                ScriptAction::new(
                    ModuleId::new(AccountAddress::default(), ident("M")),
                    ident("f"),
                    vec![],
                ),
                "0x0.M.f",
            ),
            (
                ScriptAction::new(
                    ModuleId::new(AccountAddress::default(), ident("M")),
                    ident("f"),
                    vec![
                        TransactionArgument::U64(6),
                        TransactionArgument::Address(
                            AccountAddress::from_hex_literal("0x1").unwrap(),
                        ),
                        TransactionArgument::ByteArray(ByteArray::new(
                            hex::decode("abcd").unwrap(),
                        )),
                        TransactionArgument::Bool(false),
                    ],
                ),
                "0x0.M.f 6 0x1 b\"abcd\" false",
            ),
        ];
        test_pairs
            .iter()
            .for_each(|(action, action_str)| do_test(action, action_str).unwrap());
    }

    fn do_test(action: &ScriptAction, action_str: &str) -> Result<()> {
        assert_eq!(action, &ScriptAction::parse(action_str)?);
        Ok(())
    }
}
