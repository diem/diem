// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Gas metering logic for the Move VM.

//***************************************************************************
// Gas Metering Logic
//***************************************************************************

#[macro_export]
macro_rules! gas {
    (instr: $context:ident, $self:ident, $opcode:path, $mem_size:expr) => {
        $context.deduct_gas(
            $self
                .gas_schedule
                .instruction_cost($opcode as u8)
                .total()
                .mul($mem_size),
        )
    };
    (const_instr: $context:ident, $self:ident, $opcode:path) => {
        $context.deduct_gas($self.gas_schedule.instruction_cost($opcode as u8).total())
    };
    (consume: $context:ident, $expr:expr) => {
        $context.deduct_gas($expr)
    };
}
