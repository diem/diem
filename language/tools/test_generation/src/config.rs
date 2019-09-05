// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

/// This defines how tolerant the generator will be about deviating from
/// the starting stack height. The higher the tolerance, the more likely
/// it is for invalid programs to be generated within a given target number
/// of instructions.
/// Default is 0.9 for generating ~1000 instruction sequences.
pub const MUTATION_TOLERANCE: f32 = 0.9;

/// This defines the maximum number of blocks that will be generated for
/// a function body's CFG. During generation, a random number of blocks from
/// 1 to this constant will be created.
pub const MAX_CFG_BLOCKS: u16 = 10;

/// This defines the number of programs that will be generated and then
/// run on the bytecode verifier and transaction execution logic
pub const NUM_ITERATIONS: u64 = 100;

/// Whether preconditions will be negated to generate invalid programs
/// in order to test error paths.
pub const NEGATE_PRECONDITIONS: bool = false;

/// The probability that preconditions will be negated for a pariticular
/// bytecode instruction.
pub const NEGATION_PROBABILITY: f64 = 0.1;

/// Whether generation of instructions that require borrow checking will
/// be allowed. (Note that if `NEGATE_PRECONDITIONS` is true then these
/// instructions can still come up).
pub const ALLOW_MEMORY_UNSAFE: bool = false;

/// Whether the generated programs should be run on the VM
pub const RUN_ON_VM: bool = true;

/// Whether generated modules will be executed even if they fail the
/// the bytecode verifier.
pub const EXECUTE_UNVERIFIED_MODULE: bool = true;

/// Whether gas will be metered when running generated programs. The default
/// is `true` to bound the execution time.
pub const GAS_METERING: bool = true;
