// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::{
    fs::File,
    io::{self, Read, Write},
    time::Duration,
};

use bytecode_verifier::VerifiedModule;
use compiler::Compiler;
use failure::Error;
use getopts::{Options, ParsingStyle};
use hex;
use stdlib::stdlib_modules;
use types::{
    account_address::AccountAddress,
    byte_array::ByteArray,
    transaction::{Program, RawTransaction, SignedTransaction, TransactionArgument},
};
use vm_runtime::static_verify_program;
use vm_runtime_tests::{
    account::{Account, AccountResource},
    executor::FakeExecutor,
};

struct Repl {
    accounts: Vec<Account>,
    executor: FakeExecutor,
    modules: Vec<VerifiedModule>,
    source_parser: Options,
    publish_parser: Options,
    get_account_parser: Options,
}

pub fn parse_address(s: Option<String>) -> usize {
    s.map(|s| {
        if s == "" || s.as_bytes()[0] != b'a' {
            0
        } else {
            s[1..].parse::<usize>().unwrap_or(0)
        }
    })
    .unwrap_or(0)
}

const ACCOUNT_SIZE: usize = 10;
const GENESIS_BALANCE: u64 = 100_000_000;
const DEFAULT_GAS_COST: u64 = 1;
const DEFAULT_MAX_GAS: u64 = 10_000;

impl Repl {
    pub fn get_sequence_number(&self, account: &Account) -> u64 {
        let sender_resource = self
            .executor
            .read_account_resource(account)
            .expect("sender must exist");
        AccountResource::read_sequence_number(&sender_resource)
    }

    pub fn new() -> Self {
        let mut executor = FakeExecutor::from_genesis_file();
        let accounts = executor.create_accounts(ACCOUNT_SIZE, GENESIS_BALANCE, 0);
        let mut source_parser = Options::new();
        source_parser.parsing_style(ParsingStyle::FloatingFrees);
        source_parser.reqopt("f", "file", "Script that you want to execute", "FILE");
        source_parser.optopt("s", "sender", "Sender of this transaction", "ADDRESS");
        source_parser.optopt(
            "",
            "sign_with",
            "PrivKey of account used to sign this transaction",
            "PRIVKEY",
        );
        source_parser.optflag("v", "verbose", "Display the transaction output");

        let mut publish_parser = Options::new();
        publish_parser.reqopt("f", "file", "Module that you want to publish", "FILE");
        publish_parser.optopt("s", "sender", "Publisher of the account", "ADDRESS");

        let mut get_account_parser = Options::new();
        get_account_parser.optopt("s", "sender", "Account you want to query", "ADDRESS");

        Repl {
            executor,
            accounts,
            modules: stdlib_modules().to_vec(),
            source_parser,
            publish_parser,
            get_account_parser,
        }
    }

    pub fn create_signed_txn_with_args(
        &mut self,
        program_str: String,
        args: Vec<TransactionArgument>,
        sender_address: AccountAddress,
        signer: Account,
        sequence_number: u64,
        max_gas_amount: u64,
        gas_unit_price: u64,
    ) -> SignedTransaction {
        let compiler = Compiler {
            code: &program_str,
            address: sender_address,
            extra_deps: self.modules.clone(),
            ..Compiler::default()
        };
        let compiled_program = compiler.into_compiled_program().expect("Failed to compile");

        let (verified_script, to_be_published_modules) = static_verify_program(
            &sender_address,
            compiled_program.script,
            compiled_program.modules,
        )
        .expect("verification failed");

        self.modules.extend(to_be_published_modules.clone());

        let mut script_blob = vec![];
        verified_script.serialize(&mut script_blob).unwrap();
        let mut modules_blob = vec![];
        for m in to_be_published_modules {
            let mut module_blob = vec![];
            m.serialize(&mut module_blob).unwrap();
            modules_blob.push(module_blob);
        }
        println!("program: {}, args: {:?}", program_str, args);

        let program = Program::new(script_blob, modules_blob, args);
        RawTransaction::new(
            sender_address,
            sequence_number,
            program,
            max_gas_amount,
            gas_unit_price,
            Duration::from_secs(u64::max_value()),
        )
        .sign(&signer.privkey, signer.pubkey)
        .unwrap()
        .into_inner()
    }

    pub fn eval_arg(&mut self, input: String) {
        let args: Vec<&str> = input.trim().split(' ').collect();
        match args[0] {
            "publish" => self.publish(&args[1..]),
            "source" => self.source(&args[1..]),
            "get_account_info" => self.get_account_info(&args[1..]),
            "new_key_pair" => {
                let account = Account::new();
                println!("New Account at {}: {:?}", self.accounts.len(), account);
                self.accounts.push(account);
                Ok(())
            }
            _ => {
                println!("Try these commands: publish, source, get_account_info, new_key_pair");
                Ok(())
            }
        }
        .unwrap_or_else(|e| println!("{:?}", e))
    }

    pub fn source(&mut self, args: &[&str]) -> Result<(), Error> {
        let matches = self.source_parser.parse(args).map_err(|e| {
            println!("{}", self.source_parser.usage("Execute a transaction. Escape parameters will be parsed as arguments to the transaction"));
            e
        })?;
        let sender = parse_address(matches.opt_str("s"));
        let txn_code = {
            let mut buffer = String::new();
            File::open(matches.opt_str("f").unwrap())?
                .read_to_string(&mut buffer)
                .unwrap();
            buffer
        };
        let signer = parse_address(matches.opt_str("sign_with"));
        let txn_args = {
            let mut v = vec![];
            for s in matches.free.iter() {
                v.push(if s.starts_with('a') {
                    TransactionArgument::Address(
                        *self.accounts[parse_address(Some(s.clone()))].address(),
                    )
                } else if s.starts_with("b0x") {
                    TransactionArgument::ByteArray(ByteArray::new(hex::decode(&s[3..])?))
                } else {
                    TransactionArgument::U64(s.parse::<u64>()?)
                })
            }
            v
        };
        let txn = self.create_signed_txn_with_args(
            txn_code,
            txn_args,
            *self.accounts[sender].address(),
            self.accounts[signer].clone(),
            self.get_sequence_number(&self.accounts[sender]),
            DEFAULT_MAX_GAS,
            DEFAULT_GAS_COST,
        );
        for o in self.executor.execute_block(vec![txn]).iter() {
            if matches.opt_present("v") {
                println!("{:?}", o);
            } else {
                println!("Gas Consumed: {}", o.gas_used());
            }
            self.executor.apply_write_set(o.write_set());
        }
        Ok(())
    }

    pub fn publish(&mut self, args: &[&str]) -> Result<(), Error> {
        let matches = self.publish_parser.parse(args).map_err(|e| {
            println!(
                "{}",
                self.publish_parser
                    .usage("Publish a module under a given sender")
            );
            e
        })?;
        let file = {
            let mut buffer = String::new();
            File::open(matches.opt_str("f").unwrap())?
                .read_to_string(&mut buffer)
                .unwrap();
            buffer
        };
        let sender = matches
            .opt_str("s")
            .map(|s| s.parse::<usize>().unwrap_or(0))
            .unwrap_or(0);

        let txn = self.create_signed_txn_with_args(
            format!("modules: {} script: main() {{ return; }}", file),
            vec![],
            *self.accounts[sender].address(),
            self.accounts[sender].clone(),
            self.get_sequence_number(&self.accounts[sender]),
            DEFAULT_MAX_GAS,
            DEFAULT_GAS_COST,
        );
        for o in self.executor.execute_block(vec![txn]).iter() {
            if matches.opt_defined("v") {
                println!("{:?}", o);
            } else {
                println!("Gas Consumed: {}", o.gas_used());
            }
        }
        Ok(())
    }

    pub fn get_account_info(&mut self, args: &[&str]) -> Result<(), Error> {
        let matches = self.get_account_parser.parse(args)?;
        let sender = parse_address(matches.opt_str("s"));
        let account = &self.accounts[sender];
        println!(
            "Address: 0x{}",
            hex::encode(self.accounts[sender].address())
        );
        if let Some(v) = self.executor.read_account_resource(account) {
            println!("balance: {}", AccountResource::read_balance(&v));
            println!(
                "sequence_number: {}",
                AccountResource::read_sequence_number(&v)
            );
        } else {
            println!("Account don't exist");
        }
        Ok(())
    }
}

fn main() {
    let mut repl = Repl::new();
    loop {
        let mut input = String::new();
        print!("> ");
        io::stdout().flush().unwrap();
        if io::stdin().read_line(&mut input).is_ok() {
            repl.eval_arg(input);
        }
    }
}
