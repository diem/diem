// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This performs instruction cost synthesis for the various bytecode instructions that we have. We
//! separate instructions into three sets:
//! * Global-memory independent instructions;
//! * Global-memory dependent instructions; and
//! * Native operations.
use cost_synthesis::{
    global_state::{account::Account, inhabitor::RandomInhabitor},
    module_generator::ModuleGenerator,
    natives::StackAccessorMocker,
    stack_generator::RandomStackGenerator,
    with_loaded_vm,
};
use csv;
use move_ir_natives::hash;
use std::{collections::HashMap, convert::TryFrom, path::Path, time::Instant, u64};
use vm::{
    errors::VMErrorKind,
    file_format::{
        AddressPoolIndex, ByteArrayPoolIndex, Bytecode, FieldDefinitionIndex,
        FunctionDefinitionIndex, FunctionHandleIndex, StringPoolIndex, StructDefinitionIndex,
    },
    transaction_metadata::TransactionMetadata,
};
use vm_cache_map::Arena;
use vm_runtime::{
    code_cache::module_cache::{ModuleCache, VMModuleCache},
    loaded_data::function::{FunctionRef, FunctionReference},
    txn_executor::TransactionExecutor,
};
use vm_runtime_tests::data_store::FakeDataStore;

const MAX_STACK_SIZE: u64 = 100;
const NUM_ITERS: u16 = 10000;

fn output_to_csv(path: &Path, data: HashMap<String, Vec<u64>>) {
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

// The only instruction that we don't implement here is `EmitEvent`. This is on purpose -- the emit
// event instruction will be changing soon, so it's not worth implementing at the moment until we
// have decided the semantics of the instruction.
fn stack_instructions() {
    use Bytecode::*;
    let stack_opcodes: Vec<Bytecode> = vec![
        ReadRef,
        WriteRef,
        ReleaseRef,
        FreezeRef,
        MoveToSender(StructDefinitionIndex::new(0)),
        Exists(StructDefinitionIndex::new(0)),
        BorrowGlobal(StructDefinitionIndex::new(0)),
        MoveFrom(StructDefinitionIndex::new(0)),
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
        Not,
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
    let mut account = Account::new();
    with_loaded_vm! (mod_gen, account => vm, loaded_module, module_cache);
    let costs: HashMap<String, Vec<u64>> = stack_opcodes
        .into_iter()
        .map(|instruction| {
            println!("Running: {:?}", instruction);
            let stack_gen = RandomStackGenerator::new(
                &account.addr,
                &loaded_module,
                &module_cache,
                &instruction,
                MAX_STACK_SIZE,
                NUM_ITERS,
            );
            let instr_costs: Vec<u64> = stack_gen
                .map(|stack_state| {
                    let instr = RandomStackGenerator::stack_transition(
                        &mut vm.execution_stack,
                        stack_state,
                    );
                    // Clear the VM's data cache -- otherwise we'll windup grabbing the data from
                    // the cache on subsequent iterations and across future instructions that
                    // effect global memory.
                    vm.clear_writes();
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
                    u64::try_from(time).unwrap()
                })
                .collect();
            (format!("{:?}", instruction), instr_costs)
        })
        .collect();

    output_to_csv(Path::new("data/bytecode_instruction_costs.csv"), costs);
}

macro_rules! bench_native {
    ($name:expr, $function:path, $table:ident) => {
        let mut stack_access = StackAccessorMocker::new();
        let per_byte_costs: Vec<u64> = (1..512)
            .map(|i| {
                stack_access.set_hash_length(i);
                let time = (0..NUM_ITERS).fold(0, |acc, _| {
                    stack_access.next_bytearray();
                    let before = Instant::now();
                    let _ = $function(&mut stack_access).unwrap();
                    acc + before.elapsed().as_nanos()
                });
                // Time per byte averaged over the number of iterations that we performed.
                u64::try_from(time).unwrap() / (u64::from(NUM_ITERS) * (i as u64))
            })
            .collect();
        $table.insert($name, per_byte_costs);
    };
}

fn natives() {
    let mut cost_table = HashMap::new();
    bench_native!(
        "keccak_256".to_string(),
        hash::native_keccak_256,
        cost_table
    );
    bench_native!(
        "ripemd_160".to_string(),
        hash::native_ripemd_160,
        cost_table
    );
    bench_native!(
        "native_sha2_256".to_string(),
        hash::native_sha2_256,
        cost_table
    );
    bench_native!(
        "native_sha3_256".to_string(),
        hash::native_sha3_256,
        cost_table
    );
    output_to_csv(Path::new("data/native_function_costs.csv"), cost_table);
}

pub fn main() {
    stack_instructions();
    natives();
}
