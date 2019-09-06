use bytecode_verifier::CodeUnitVerifier;
use types::vm_error::StatusCode;
use vm::file_format::{self, Bytecode};

#[test]
fn one_pop_no_push() {
    let module = file_format::dummy_procedure_module(vec![], vec![Bytecode::Pop, Bytecode::Ret]);
    let errors = CodeUnitVerifier::verify(&module);
    println!("{:?}", errors);
    assert_eq!(
        errors[0].major_status,
        StatusCode::NEGATIVE_STACK_SIZE_WITHIN_BLOCK
    );
}

#[test]
fn one_pop_one_push() {
    // Height: 0 + (-1 + 1) = 0 would have passed original usage verifier
    let module =
        file_format::dummy_procedure_module(vec![], vec![Bytecode::ReadRef, Bytecode::Ret]);
    let errors = CodeUnitVerifier::verify(&module);
    assert_eq!(
        errors[0].major_status,
        StatusCode::NEGATIVE_STACK_SIZE_WITHIN_BLOCK
    );
}

#[test]
fn two_pop_one_push() {
    // Height: 0 + 1 + (-2 + 1) = 0 would have passed original usage verifier
    let module = file_format::dummy_procedure_module(
        vec![],
        vec![Bytecode::LdConst(0), Bytecode::Add, Bytecode::Ret],
    );
    let errors = CodeUnitVerifier::verify(&module);
    assert_eq!(
        errors[0].major_status,
        StatusCode::NEGATIVE_STACK_SIZE_WITHIN_BLOCK
    );
}

#[test]
fn two_pop_no_push() {
    let module =
        file_format::dummy_procedure_module(vec![], vec![Bytecode::WriteRef, Bytecode::Ret]);
    let errors = CodeUnitVerifier::verify(&module);
    assert_eq!(
        errors[0].major_status,
        StatusCode::NEGATIVE_STACK_SIZE_WITHIN_BLOCK
    );
}
