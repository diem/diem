use bytecode_to_boogie::translator::BoogieTranslator;
use bytecode_verifier::VerifiedModule;
use ir_to_bytecode::{compiler::compile_module, parser::parse_module};
use std::{
    env,
    fs::{self, File},
    io::prelude::*,
};
use types::account_address::AccountAddress;

// mod translator;
fn compile_files(file_names: Vec<String>) -> Vec<VerifiedModule> {
    let mut verified_modules = vec![];
    let address = &AccountAddress::default();
    for file_name in file_names {
        let code = fs::read_to_string(file_name).unwrap();
        let module = parse_module(&code).unwrap();
        let compiled_module =
            compile_module(&address, &module, &verified_modules).expect("module failed to compile");
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
