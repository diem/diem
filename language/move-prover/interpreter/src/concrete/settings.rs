// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#[derive(Default)]
pub struct InterpreterSettings {
    /// dump stepwise bytecode
    pub verbose_stepwise: bool,

    /// dump bytecode trace
    pub verbose_bytecode: bool,
}
