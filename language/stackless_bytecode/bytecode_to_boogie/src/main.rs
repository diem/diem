use bytecode_to_boogie::translator::BoogieTranslator;
use bytecode_verifier::VerifiedModule;
use ir_to_bytecode::{compiler::compile_module, parser::parse_module};
use std::{
    env,
    fs::{self, File},
    io::prelude::*,
};
use types::account_address::AccountAddress;
use vm::file_format::CompiledModule;
// mod translator;

fn main() {
    let args: Vec<String> = env::args().collect();
    let file_name = &args[1];

    // read file and compile into compiled module
    let code = fs::read_to_string(file_name).unwrap();
    let module = parse_module(&code).unwrap();
    let address = &AccountAddress::default();
    let deps: Vec<CompiledModule> = vec![];
    let compiled_module = compile_module(&address, &module, &deps).unwrap();
    let verified_module_res = VerifiedModule::new(compiled_module);
    match verified_module_res {
        Ok(verified_module) => {
            // uncomment to print out bytecode for debugging purpose
            // let bc = &verified_module.as_inner().as_inner().function_defs[0].code.code;
            // for b in bc {
            //     println!("{:?}", b);
            // }

            let mut ts = BoogieTranslator::new(&verified_module);
            let mut res = String::new();

            // handwritten boogie code
            let written_code = fs::read_to_string("src/bytecode_instrs.bpl").unwrap();
            res.push_str(&written_code);
            res.push_str(&ts.translate());
            let mut f = File::create("output.bpl").expect("Unable to create file");

            // write resulting code into output.bpl
            write!(f, "{}", res).expect("unable to write file");
        }
        Err(e) => {
            println!("{:?}", e);
        }
    }
}
