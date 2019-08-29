// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use criterion::{BatchSize, Bencher};
use ir_to_bytecode::{
    compiler::compile_program,
    parser::{ast, parse_program},
};
use language_e2e_tests::{
    account::{Account, AccountData},
    executor::FakeExecutor,
};
use libra_types::transaction::TransactionStatus;
use libra_types::vm_error::{StatusCode, VMStatus};
use libra_types::{
    account_address::AccountAddress,
    transaction::{Script, SignedTransaction, TransactionArgument, TransactionPayload},
};
use stdlib::stdlib_modules;
use vm::{
    access::ModuleAccess,
    file_format::{CompiledModule, CompiledScript},
};

/// Benchmarking support for VM.
#[derive(Debug)]
pub struct VMBencher {
    data: VMBencherData,
    executor: FakeExecutor,
}

#[derive(Debug)]
struct VMBencherData {
    account: Account,
    sequence_number: u64,
    scripts: Vec<(u64, Script)>,
}

impl VMBencher {
    pub fn new() -> Self {
        VMBencher {
            data: VMBencherData::new(),
            executor: FakeExecutor::from_genesis_file(),
        }
    }

    pub fn add_module(&mut self, module: CompiledModule) {
        self.executor
            .add_module(&module.as_module().self_id(), &module);
    }

    pub fn run_script(&mut self) {
        let code = r"
            import 0x0.Vector;

            main() {
              let vec: Vector.T<u64>;
              let i: u64;
              let ref: &mut u64;
              vec = Vector.empty<u64>();

              i = 0;
              while (copy(i) < 10) {
                Vector.push_back<u64>(&mut vec, copy(i));
                i = move(i) + 1;
              }
              i = 0;
              while (copy(i) < 10) {
                _ = Vector.pop_back<u64>(&mut vec);
                i = move(i) + 1;
              }
              i = 0;
              while (copy(i) < 10) {
                Vector.push_back<u64>(&mut vec, copy(i));
                i = move(i) + 1;
              }
              i = 0;
              while (copy(i) < 9) {
                Vector.swap<u64>(&mut vec, copy(i), copy(i) + 1);
                i = move(i) + 1;
              }
              i = 0;
              while (copy(i) < 10) {
                _ = Vector.borrow_mut<u64>(&mut vec, copy(i));
                i = move(i) + 1;
              }
              i = 0;
              while (copy(i) < 10) {
                _ = Vector.pop_back<u64>(&mut vec);
                i = move(i) + 1;
              }
              Vector.destroy_empty<u64>(move(vec));

              return;
            }
        ";
        let program = parse_program(code).expect("must compile");
        let script = compile_script(program);
        self.add_script(script, vec![], 1000);
    }

    pub fn add_script(
        &mut self,
        script: CompiledScript,
        args: Vec<TransactionArgument>,
        count: u64,
    ) {
        let mut binary = Vec::new();
        script.serialize(&mut binary).expect("must serialize");
        let txn_script = Script::new(binary, args);
        self.data.add_script(txn_script, count);
    }

    pub fn bench(&mut self, bench: &mut Bencher) {
        self.executor
            .add_account_data(&self.data.get_account_data());
        let data = &mut self.data;
        let executor = &mut self.executor;
        bench.iter_batched(
            || data.get_transactions(),
            |txns| {
                let output = executor.execute_block(txns);
                for out in output {
                    assert_eq!(
                        out.status(),
                        &TransactionStatus::Keep(VMStatus::new(StatusCode::EXECUTED))
                    );
                }
            },
            // The input here is the entire list of signed transactions, so it's pretty large.
            BatchSize::LargeInput,
        )
    }
}

impl VMBencherData {
    fn new() -> Self {
        VMBencherData {
            account: Account::new(),
            sequence_number: 1,
            scripts: Vec::new(),
        }
    }

    fn add_script(&mut self, script: Script, count: u64) {
        self.scripts.push((count, script));
    }

    fn get_account_data(&self) -> AccountData {
        AccountData::with_account(
            self.account.clone(),
            1_000_000_000_000,
            self.sequence_number,
        )
    }

    fn get_transactions(&mut self) -> Vec<SignedTransaction> {
        let mut txns = Vec::new();
        let mut sequence_number = self.sequence_number;
        for (count, script) in &self.scripts {
            for _ in 0..*count {
                txns.push(self.account.create_user_txn(
                    TransactionPayload::Script(script.clone()),
                    sequence_number,
                    1_000_000,
                    1,
                ));
                sequence_number += 1;
            }
        }
        txns
    }
}

fn compile_script(body: ast::Program) -> CompiledScript {
    let (compiled_program, _) =
        compile_program(AccountAddress::default(), body, stdlib_modules()).unwrap();
    compiled_program.script
}
