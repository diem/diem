use bytecode_to_boogie::translator::BoogieTranslator;
use bytecode_verifier::VerifiedModule;
use ir_to_bytecode::{
    compiler::{compile_module, compile_program},
    parser::{parse_module, parse_program},
};
use std::{
    env,
    fs::{self, File},
    io::prelude::*,
};
use types::account_address::AccountAddress;

fn compile_files(file_names: Vec<String>) -> Vec<VerifiedModule> {
    let mut verified_modules = vec![];
    let files_len = file_names.len();
    let dep_files = &file_names[0..files_len - 1];

    // assuming the last file is a program that might contain a script
    let main_file = &file_names[files_len - 1];
    let address = AccountAddress::default();
    for file_name in dep_files {
        let code = fs::read_to_string(file_name).unwrap();
        let module = parse_module(&code).unwrap();
        let compiled_module =
            compile_module(address, module, &verified_modules).expect("module failed to compile");
        let verified_module_res = VerifiedModule::new(compiled_module);

        match verified_module_res {
            Err(e) => {
                panic!("{:?}", e);
            }
            Ok(verified_module) => {
                verified_modules.push(verified_module);
            }
        }
    }
    let main_code = fs::read_to_string(main_file).unwrap();
    let program = parse_program(&main_code).unwrap();
    let address = AccountAddress::default();
    let compiled_program =
        compile_program(address, program, &verified_modules).expect("program failed to compile");
    let mut main_modules = compiled_program.modules;
    main_modules.push(compiled_program.script.into_module());
    for module in main_modules {
        let verified_module_res = VerifiedModule::new(module);

        match verified_module_res {
            Err(e) => {
                panic!("{:?}", e);
            }
            Ok(verified_module) => {
                verified_modules.push(verified_module);
            }
        }
    }

    verified_modules
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let file_names = &args[1..];
    // read files and compile into compiled modules
    let modules = compile_files(file_names.to_vec());

    let mut ts = BoogieTranslator::new(&modules);
    let mut res = String::new();

    // handwritten boogie code
    let written_code = fs::read_to_string("src/bytecode_instrs.bpl").unwrap();
    res.push_str(&written_code);
    res.push_str(&ts.translate());
    let mut f = File::create("output.bpl").expect("Unable to create file");

    // write resulting code into output.bpl
    write!(f, "{}", res).expect("unable to write file");
}
