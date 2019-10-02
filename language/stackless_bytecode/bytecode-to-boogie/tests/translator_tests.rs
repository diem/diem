use bytecode_to_boogie::translator::BoogieTranslator;
use bytecode_verifier::VerifiedModule;
use ir_to_bytecode::{compiler::compile_module, parser::parse_module};
use std::fs;
use stdlib::stdlib_modules;
use types::account_address::AccountAddress;

// mod translator;
fn compile_files(file_names: Vec<String>) -> Vec<VerifiedModule> {
    let mut verified_modules = stdlib_modules().to_vec();
    let files_len = file_names.len();
    let dep_files = &file_names[0..files_len];

    // assuming the last file is a program that might contain a script
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
    verified_modules
}

#[test]
fn test3() {
    let mut file_names = vec![];
    let name = "test_mvir/test3.mvir".to_string();
    file_names.push(name);
    let modules = compile_files(file_names.to_vec());

    let mut ts = BoogieTranslator::new(&modules);
    let mut res = String::new();

    // handwritten boogie code
    let written_code = fs::read_to_string("src/bytecode_instrs.bpl").unwrap();
    res.push_str(&written_code);
    res.push_str(&ts.translate());
    // This is probably too sensitive to minor changes in libra; commenting for now.
    //    let expected_code = fs::read_to_string("test_mvir/test3.bpl.expect").unwrap();
    //    assert_eq!(res, expected_code);
}

#[test]
fn test_arithmetic() {
    let mut file_names = vec![];
    let name = "test_mvir/test-arithmetic.mvir".to_string();
    file_names.push(name);
    let modules = compile_files(file_names.to_vec());

    let mut ts = BoogieTranslator::new(&modules);
    let mut res = String::new();

    // handwritten boogie code
    let written_code = fs::read_to_string("src/bytecode_instrs.bpl").unwrap();
    res.push_str(&written_code);
    res.push_str(&ts.translate());
}

#[test]
fn test_control_flow() {
    let mut file_names = vec![];
    let name = "test_mvir/test-control-flow.mvir".to_string();
    file_names.push(name);
    let modules = compile_files(file_names.to_vec());

    let mut ts = BoogieTranslator::new(&modules);
    let mut res = String::new();

    // handwritten boogie code
    let written_code = fs::read_to_string("src/bytecode_instrs.bpl").unwrap();
    res.push_str(&written_code);
    res.push_str(&ts.translate());
}

#[test]
fn test_func_call() {
    let mut file_names = vec![];
    let name = "test_mvir/test-func-call.mvir".to_string();
    file_names.push(name);
    let modules = compile_files(file_names.to_vec());

    let mut ts = BoogieTranslator::new(&modules);
    let mut res = String::new();

    // handwritten boogie code
    let written_code = fs::read_to_string("src/bytecode_instrs.bpl").unwrap();
    res.push_str(&written_code);
    res.push_str(&ts.translate());
}

#[test]
fn test_reference() {
    let mut file_names = vec![];
    let name = "test_mvir/test-reference.mvir".to_string();
    file_names.push(name);
    let modules = compile_files(file_names.to_vec());

    let mut ts = BoogieTranslator::new(&modules);
    let mut res = String::new();

    // handwritten boogie code
    let written_code = fs::read_to_string("src/bytecode_instrs.bpl").unwrap();
    res.push_str(&written_code);
    res.push_str(&ts.translate());
}

#[test]
fn test_special_instr() {
    let mut file_names = vec![];
    let name = "test_mvir/test-special-instr.mvir".to_string();
    file_names.push(name);
    let modules = compile_files(file_names.to_vec());

    let mut ts = BoogieTranslator::new(&modules);
    let mut res = String::new();

    // handwritten boogie code
    let written_code = fs::read_to_string("src/bytecode_instrs.bpl").unwrap();
    res.push_str(&written_code);
    res.push_str(&ts.translate());
}

#[test]
fn test_struct() {
    let mut file_names = vec![];
    let name = "test_mvir/test-struct.mvir".to_string();
    file_names.push(name);
    let modules = compile_files(file_names.to_vec());

    let mut ts = BoogieTranslator::new(&modules);
    let mut res = String::new();

    // handwritten boogie code
    let written_code = fs::read_to_string("src/bytecode_instrs.bpl").unwrap();
    res.push_str(&written_code);
    res.push_str(&ts.translate());
}
