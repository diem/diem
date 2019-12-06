// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::account_address::AccountAddress;
use crate::identifier::{IdentStr, Identifier};
use crate::language_storage::ModuleId;
use crate::transaction::parse_as_transaction_argument;
use crate::transaction::transaction_argument::TransactionArgument;
use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};

#[derive(Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum Action {
    // module, function
    Call(ModuleId, Identifier),
    Code(Vec<u8>),
}

impl Action {
    pub fn parse_call(call: &str) -> Result<Self> {
        let call_parts: Vec<&str> = call.trim().split('.').collect();
        ensure!(
            call_parts.len() == 3,
            "expect call format: address.ModuleName.FunctionName  but got: {}",
            call
        );
        let address = AccountAddress::from_hex_literal(call_parts[0])?;
        let module_name = IdentStr::new(call_parts[1])?;
        let function_name = IdentStr::new(call_parts[2])?;
        Ok(Action::Call(
            ModuleId::new(address, module_name.to_owned()),
            function_name.to_owned(),
        ))
    }
}

impl std::fmt::Debug for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Action::Call(module, function) => {
                write!(
                    f,
                    "call {}.{}.{}",
                    module.address(),
                    module.name(),
                    function
                )?;
            }
            Action::Code(code) => {
                write!(f, "code {}", &hex::encode(code))?;
            }
        }
        Ok(())
    }
}

impl std::fmt::Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Action::Call(module, function) => {
                write!(
                    f,
                    "call {}.{}.{}",
                    module.address(),
                    module.name(),
                    function
                )?;
            }
            Action::Code(code) => {
                write!(f, "code {}", &hex::encode(code))?;
            }
        }
        Ok(())
    }
}

#[derive(Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct ScriptAction {
    action: Action,
    args: Vec<TransactionArgument>,
}

impl ScriptAction {
    pub fn new(action: Action, args: Vec<TransactionArgument>) -> Self {
        Self { action, args }
    }

    pub fn new_call(
        module: ModuleId,
        function: Identifier,
        args: Vec<TransactionArgument>,
    ) -> Self {
        Self {
            action: Action::Call(module, function),
            args,
        }
    }

    pub fn new_code(code: Vec<u8>, args: Vec<TransactionArgument>) -> Self {
        Self {
            action: Action::Code(code),
            args,
        }
    }

    pub fn parse_call_with_args(action: &str) -> Result<Self> {
        let parts: Vec<&str> = action.trim().split(' ').collect();
        let call = Action::parse_call(parts[0])?;
        let args_result: Result<Vec<TransactionArgument>> = if parts.len() > 1 {
            parts[1..]
                .iter()
                .map(|str| parse_as_transaction_argument(str))
                .collect()
        } else {
            Ok(vec![])
        };
        Ok(Self::new(call, args_result?))
    }

    pub fn action(&self) -> &Action {
        &self.action
    }

    pub fn args(&self) -> &[TransactionArgument] {
        &self.args
    }

    pub fn into_inner(self) -> (Action, Vec<TransactionArgument>) {
        (self.action, self.args)
    }
}

impl std::fmt::Debug for ScriptAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.action)?;
        for arg in &self.args {
            write!(f, " ")?;
            write!(f, "{:?}", arg)?;
        }
        Ok(())
    }
}

impl std::fmt::Display for ScriptAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.action)?;
        for arg in &self.args {
            write!(f, " ")?;
            write!(f, "{:?}", arg)?;
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
                ScriptAction::new_call(
                    ModuleId::new(AccountAddress::default(), ident("M")),
                    ident("f"),
                    vec![],
                ),
                "0x0.M.f",
            ),
            (
                ScriptAction::new_call(
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
        assert_eq!(action, &ScriptAction::parse_call_with_args(action_str)?);
        Ok(())
    }
}
