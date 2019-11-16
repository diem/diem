// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

//! This performs instruction cost synthesis for the various bytecode instructions that we have. We
//! separate instructions into three sets:
//! * Global-memory independent instructions;
//! * Global-memory dependent instructions; and
//! * Native operations.
use cost_synthesis::{
    global_state::{account::Account, inhabitor::RandomInhabitor},
    module_generator::generate_padded_modules,
    natives::StackAccessorMocker,
    stack_generator::RandomStackGenerator,
    with_loaded_vm,
};
use csv;
use language_e2e_tests::data_store::FakeDataStore;
use libra_types::vm_error::StatusCode;
use std::{
    collections::{HashMap, VecDeque},
    convert::TryFrom,
    path::Path,
    time::Instant,
    u64,
};
use structopt::StructOpt;
use vm::{
    file_format::{
        AddressPoolIndex, ByteArrayPoolIndex, Bytecode, FieldDefinitionIndex,
        FunctionDefinitionIndex, FunctionHandleIndex, StructDefinitionIndex, UserStringIndex,
        NO_TYPE_ACTUALS,
    },
    gas_schedule::{AbstractMemorySize, CostTable, GasAlgebra, GasCarrier},
    transaction_metadata::TransactionMetadata,
};
use vm_cache_map::Arena;
use vm_runtime::{
    code_cache::module_cache::{ModuleCache, VMModuleCache},
    interpreter::InterpreterForCostSynthesis,
    loaded_data::function::{FunctionRef, FunctionReference},
};
use vm_runtime_types::{native_functions::hash, value::Value};

#[derive(Debug, StructOpt)]
#[structopt(
    name = "Instruction Cost Synthesis",
    about = "Cost synthesis parameter settings"
)]
struct Opt {
    /// The number of iterations each instruction should be run for.
    #[structopt(short = "i", long = "num-iters", default_value = "10000")]
    num_iters: u16,

    /// The maximum stack size generated.
    #[structopt(short = "ms", long = "max-stack-size", default_value = "100")]
    max_stack_size: u64,

    #[structopt(short = "o", long = "output")]
    output: bool,
}

fn output_to_csv(path: &Path, data: HashMap<String, Vec<u64>>, output: bool) {
    if output {
        let mut writer = csv::Writer::from_path(path).unwrap();
        let keys: Vec<_> = data.keys().collect();
        let datavars: Vec<_> = data.values().collect();
        writer.write_record(&keys).unwrap();
        for i in 0..datavars[0].len() {
            let row: Vec<_> = datavars.iter().map(|v| v[i]).collect();
            writer.serialize(&row).unwrap();
        }

        writer.flush().unwrap();
    }
}

fn size_normalize_cost(instr: &Bytecode, cost: u64, size: AbstractMemorySize<GasCarrier>) -> u64 {
    match instr {
        Bytecode::MoveToSender(_, _)
        | Bytecode::Exists(_, _)
        | Bytecode::MutBorrowGlobal(_, _)
        | Bytecode::ImmBorrowGlobal(_, _)
        | Bytecode::Eq
        | Bytecode::Neq
        | Bytecode::LdStr(_)
        | Bytecode::LdByteArray(_)
        | Bytecode::StLoc(_)
        | Bytecode::CopyLoc(_)
        | Bytecode::Pack(_, _)
        | Bytecode::Unpack(_, _)
        | Bytecode::WriteRef
        | Bytecode::ReadRef
        | Bytecode::MoveFrom(_, _) => {
            cost / size.get() + if cost % size.get() == 0 { 0 } else { 1 }
        }
        _ => cost,
    }
}

fn stack_instructions(options: &Opt) {
    use Bytecode::*;
    let stack_opcodes: Vec<Bytecode> = vec![
        ReadRef,
        WriteRef,
        FreezeRef,
        MoveToSender(StructDefinitionIndex::new(0), NO_TYPE_ACTUALS),
        Exists(StructDefinitionIndex::new(0), NO_TYPE_ACTUALS),
        MutBorrowGlobal(StructDefinitionIndex::new(0), NO_TYPE_ACTUALS),
        ImmBorrowGlobal(StructDefinitionIndex::new(0), NO_TYPE_ACTUALS),
        MoveFrom(StructDefinitionIndex::new(0), NO_TYPE_ACTUALS),
        MutBorrowField(FieldDefinitionIndex::new(0)),
        ImmBorrowField(FieldDefinitionIndex::new(0)),
        CopyLoc(0),
        MoveLoc(0),
        MutBorrowLoc(0),
        ImmBorrowLoc(0),
        StLoc(0),
        Unpack(StructDefinitionIndex::new(0), NO_TYPE_ACTUALS),
        Pack(StructDefinitionIndex::new(0), NO_TYPE_ACTUALS),
        Call(FunctionHandleIndex::new(0), NO_TYPE_ACTUALS),
        Sub,
        Ret,
        Add,
        Mul,
        Mod,
        Div,
        BitOr,
        BitAnd,
        Xor,
        Or,
        And,
        Eq,
        Neq,
        Not,
        Lt,
        Gt,
        Le,
        Ge,
        Abort,
        LdFalse,
        LdTrue,
        LdConst(0),
        LdStr(UserStringIndex::new(0)),
        LdByteArray(ByteArrayPoolIndex::new(0)),
        LdAddr(AddressPoolIndex::new(0)),
        BrFalse(0),
        BrTrue(0),
        Branch(0),
        Pop,
        GetTxnGasUnitPrice,
        GetTxnMaxGasUnits,
        GetGasRemaining,
        GetTxnSenderAddress,
        GetTxnSequenceNumber,
        GetTxnPublicKey,
    ];

    let mut account = Account::new();
    with_loaded_vm! (3, options.num_iters as usize, account => vm, loaded_module, module_cache);
    let costs: HashMap<String, Vec<u64>> = stack_opcodes
        .into_iter()
        .map(|instruction| {
            println!("Running: {:?}", instruction);
            let stack_gen = RandomStackGenerator::new(
                &account.addr,
                &loaded_module,
                &module_cache,
                &instruction,
                options.max_stack_size,
                options.num_iters,
            );
            let instr_costs: Vec<u64> = stack_gen
                .map(|stack_state| {
                    let (instr, size) =
                        RandomStackGenerator::stack_transition(&mut vm, stack_state);
                    // Clear the VM's data cache -- otherwise we'll windup grabbing the data from
                    // the cache on subsequent iterations and across future instructions that
                    // effect global memory.
                    vm.clear_writes();
                    let before = Instant::now();
                    let ignore = vm.execute_code_snippet(&[instr]);
                    let time = before.elapsed().as_nanos();
                    // Check to make sure we didn't error. Need to special case the abort bytecode.
                    if instruction != Bytecode::Abort {
                        // We want any errors here to bubble up to us with the actual VM error.
                        ignore.unwrap();
                    } else {
                        // In the case of the Abort bytecode we want to only make sure that we
                        // don't have a VMInvariantViolation error, and then make sure that the any
                        // error generated was an abort failure.
                        match ignore {
                            Ok(_) => (),
                            Err(err) => match err.major_status {
                                StatusCode::ABORTED => (),
                                _ => panic!("Abort bytecode failed"),
                            },
                        }
                    }
                    size_normalize_cost(&instruction, u64::try_from(time).unwrap(), size)
                })
                .collect();
            (format!("{:?}", instruction), instr_costs)
        })
        .collect();

    output_to_csv(
        Path::new("data/bytecode_instruction_costs.csv"),
        costs,
        options.output,
    );
}

macro_rules! bench_native {
    ($name:expr, $function:path, $table:ident, $iters:expr) => {
        let mut stack_access = StackAccessorMocker::new();
        let cost_table = CostTable::zero();
        let per_byte_costs: Vec<u64> = (1..512)
            .map(|i| {
                stack_access.set_hash_length(i);
                let time = (0..$iters).fold(0, |acc, _| {
                    let before = Instant::now();
                    let mut args = VecDeque::new();
                    args.push_front(Value::byte_array(stack_access.next_bytearray()));
                    let _ = $function(args, &cost_table);
                    acc + before.elapsed().as_nanos()
                });
                // Time per byte averaged over the number of iterations that we performed.
                u64::try_from(time).unwrap() / (u64::from($iters) * (i as u64))
            })
            .collect();
        $table.insert($name, per_byte_costs);
    };
}

fn natives(options: &Opt) {
    let mut cost_table = HashMap::new();
    bench_native!(
        "native_sha2_256".to_string(),
        hash::native_sha2_256,
        cost_table,
        options.num_iters
    );
    bench_native!(
        "native_sha3_256".to_string(),
        hash::native_sha3_256,
        cost_table,
        options.num_iters
    );
    output_to_csv(
        Path::new("data/native_function_costs.csv"),
        cost_table,
        options.output,
    );
}

pub fn main() {
    let opt = Opt::from_args();
    stack_instructions(&opt);
    natives(&opt);
}
