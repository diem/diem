// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This performs instruction cost synthesis for the various bytecode instructions that we have. We
//! separate instructions into three sets:
//! * Global-memory independent instructions;
//! * Global-memory dependent instructions; and
//! * Native operations.
use cost_synthesis::{
    module_generator::ModuleGenerator, natives::StackAccessorMocker,
    stack_generator::RandomStackGenerator, vm_runner::FakeDataCache, with_loaded_vm,
};
use move_ir_natives::hash;
use std::{collections::HashMap, time::Instant};
use vm::{
    errors::VMErrorKind,
    file_format::{
        AddressPoolIndex, ByteArrayPoolIndex, Bytecode, FieldDefinitionIndex,
        FunctionDefinitionIndex, FunctionHandleIndex, StringPoolIndex, StructDefinitionIndex,
    },
    transaction_metadata::TransactionMetadata,
};
use vm_cache_map::Arena;
use vm_genesis::STDLIB_MODULES;
use vm_runtime::{
    code_cache::module_cache::{ModuleCache, VMModuleCache},
    loaded_data::function::{FunctionRef, FunctionReference},
    txn_executor::TransactionExecutor,
};

const MAX_STACK_SIZE: u64 = 100;
const NUM_ITERS: u16 = 1000;

fn stack_instructions() {
    use Bytecode::*;
    let stack_opcodes: Vec<Bytecode> = vec![
        ReadRef,
        WriteRef,
        ReleaseRef,
        FreezeRef,
        BorrowField(FieldDefinitionIndex::new(0)),
        CopyLoc(0),
        MoveLoc(0),
        BorrowLoc(0),
        StLoc(0),
        Unpack(StructDefinitionIndex::new(0)),
        Pack(StructDefinitionIndex::new(0)),
        Call(FunctionHandleIndex::new(0)),
        CreateAccount,
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
        Lt,
        Gt,
        Le,
        Ge,
        Assert,
        LdFalse,
        LdTrue,
        LdConst(0),
        LdStr(StringPoolIndex::new(0)),
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

    let mod_gen: ModuleGenerator = ModuleGenerator::new(NUM_ITERS as u16, 3);
    with_loaded_vm! (mod_gen => vm, loaded_module, module_cache);
    let costs: HashMap<Bytecode, u128> = stack_opcodes
        .into_iter()
        .map(|instruction| {
            println!("Running: {:?}", instruction);
            let stack_gen = RandomStackGenerator::new(
                &loaded_module,
                &module_cache,
                &instruction,
                MAX_STACK_SIZE,
                NUM_ITERS,
            );
            let instr_cost: u128 = stack_gen
                .map(|stack_state| {
                    let instr = RandomStackGenerator::stack_transition(
                        &mut vm.execution_stack,
                        stack_state,
                    );
                    let before = Instant::now();
                    let ignore = vm.execute_block(&[instr], 0);
                    let time = before.elapsed().as_nanos();
                    // Check to make sure we didn't error. Need to special case the assertion
                    // bytecode.
                    if instruction != Bytecode::Assert {
                        // We want any errors here to bubble up to us with the actual VM error.
                        ignore.unwrap().unwrap();
                    } else {
                        // In the case of the Assert bytecode we want to only make sure that we
                        // don't have a VMInvariantViolation error, and then make sure that the any
                        // error generated was an assertion failure.
                        match ignore.unwrap() {
                            Ok(_) => (),
                            Err(err) => match err.err {
                                VMErrorKind::AssertionFailure(_) => (),
                                _ => panic!("Assertion bytecode failed"),
                            },
                        }
                    }
                    time
                })
                .sum();
            let average_time = instr_cost / u128::from(NUM_ITERS);
            (instruction, average_time)
        })
        .collect();

    println!("---------------------------------------------------------------------------");
    for (instr, cost) in costs {
        println!("{:?}: {}", instr, cost);
    }
    println!("---------------------------------------------------------------------------");
}

macro_rules! bench_native {
    ($name:expr, $function:path, $table:ident) => {
        let mut stack_access = StackAccessorMocker::new();
        let time_byte_mapping = (1..512)
            .map(|i| {
                stack_access.set_hash_length(i);
                let time = (0..NUM_ITERS).fold(0, |acc, _| {
                    stack_access.next_bytearray();
                    let before = Instant::now();
                    let _ = $function(&mut stack_access).unwrap();
                    acc + before.elapsed().as_nanos()
                });
                let time = time / u128::from(NUM_ITERS);
                (time, i as u64)
            })
            .collect::<HashMap<_, _>>();
        let time_per_byte = time_byte_mapping
            .into_iter()
            .fold(0, |acc, (time, bytes)| acc + (time / u128::from(bytes)))
            / 512;
        $table.insert($name, time_per_byte);
    };
}

fn natives() {
    let mut cost_table = HashMap::new();
    bench_native!("keccak_256", hash::native_keccak_256, cost_table);
    bench_native!("ripemd_160", hash::native_ripemd_160, cost_table);
    bench_native!("native_sha2_256", hash::native_sha2_256, cost_table);
    bench_native!("native_sha3_256", hash::native_sha3_256, cost_table);
    println!("------------------------ NATIVES ------------------------------------------");
    for (instr, cost) in cost_table {
        println!("{:?}: {}", instr, cost);
    }
    println!("---------------------------------------------------------------------------");
}

pub fn main() {
    natives();
    stack_instructions();
}

// Instructions left to implement:
//     BorrowGlobal(StructDefinitionIndex),
//     Exists(StructDefinitionIndex),
//     MoveFrom(StructDefinitionIndex),
//     MoveToSender(StructDefinitionIndex),
//     EmitEvent, <- Not yet/until it's implemented
