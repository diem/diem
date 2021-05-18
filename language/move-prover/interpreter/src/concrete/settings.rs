// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#[derive(Default, Clone)]
pub struct InterpreterSettings {
    /// dump stepwise bytecode
    pub verbose_stepwise: bool,
    /// dump bytecode trace
    pub verbose_bytecode: bool,
    /// dump expression trace
    pub verbose_expression: bool,
}

impl InterpreterSettings {
    pub fn verbose_default() -> Self {
        Self {
            verbose_stepwise: true,
            verbose_bytecode: true,
            verbose_expression: true,
        }
    }
}
