// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use structopt::StructOpt;

/// This defines how tolerant the generator will be about deviating from
/// the starting stack height.
/// Default is `0.9`
pub const MUTATION_TOLERANCE: f32 = 0.9;

/// This defines the maximum number of blocks that will be generated for
/// a function body's CFG. During generation, a random number of blocks from
/// 1 to this constant will be created.
/// Default is `10`
pub const MAX_CFG_BLOCKS: u16 = 10;

/// Whether preconditions will be negated to generate invalid programs
/// in order to test error paths.
/// Default is `false`
pub const NEGATE_PRECONDITIONS: bool = false;

/// The probability that preconditions will be negated for a pariticular
/// bytecode instruction.
/// Default is `0.1`
pub const NEGATION_PROBABILITY: f64 = 0.1;

/// Whether generation of instructions that require borrow checking will
/// be allowed. (Note that if `NEGATE_PRECONDITIONS` is true then these
/// instructions can still come up).
/// Default is `false`
pub const ALLOW_MEMORY_UNSAFE: bool = false;

/// Whether the generated programs should be run on the VM
/// Default is `true`
pub const RUN_ON_VM: bool = true;

/// Whether generated modules will be executed even if they fail the
/// the bytecode verifier.
/// Default is `false`
pub const EXECUTE_UNVERIFIED_MODULE: bool = false;

/// Whether gas will be metered when running generated programs. The default
/// is `true` to bound the execution time.
/// Default is `true`
pub const GAS_METERING: bool = true;

/// Call stack height limit. This is defined in the VM, and is replicated here. This should track
/// that constant.
pub const CALL_STACK_LIMIT: usize = 1024;

/// The value stack size limit. This is defined in the VM and is replicated here. This should
/// remain in sync with the constant for this defined in the VM.
pub const VALUE_STACK_LIMIT: usize = 1024;

/// Command line arguments for the tool
#[derive(Debug, StructOpt)]
#[structopt(
    name = "Bytecode Test Generator",
    author = "Libra",
    about = "Tool for generating tests for the bytecode verifier and Move VM runtime."
)]
pub struct Args {
    /// The number of programs that will be generated
    #[structopt(short = "i", long = "iterations")]
    pub num_iterations: u64,

    /// Path where a serialized module should be saved.
    /// If `None`, then the module will just be printed out.
    #[structopt(short = "o", long = "output")]
    pub output_path: Option<String>,

    /// The optional seed used for test generation.
    #[structopt(short = "s", long = "seed")]
    pub seed: Option<String>,
}
