use bytecode_verifier::CodeUnitVerifier;
use types::vm_error::StatusCode;
use vm::file_format::{self, Bytecode, SignatureToken};

#[test]
fn invalid_fallthrough_br_true() {
    let module =
        file_format::dummy_procedure_module(vec![], vec![Bytecode::LdFalse, Bytecode::BrTrue(1)]);
    let errors = CodeUnitVerifier::verify(&module);
    assert_eq!(errors[0].major_status, StatusCode::INVALID_FALL_THROUGH);
}

#[test]
fn invalid_fallthrough_br_false() {
    let module =
        file_format::dummy_procedure_module(vec![], vec![Bytecode::LdTrue, Bytecode::BrFalse(1)]);
    let errors = CodeUnitVerifier::verify(&module);
    assert_eq!(errors[0].major_status, StatusCode::INVALID_FALL_THROUGH);
}

// all non-branch instructions should trigger invalid fallthrough; just check one of them
#[test]
fn invalid_fallthrough_non_branch() {
    let module = file_format::dummy_procedure_module(vec![], vec![Bytecode::LdTrue, Bytecode::Pop]);
    let errors = CodeUnitVerifier::verify(&module);
    assert_eq!(errors[0].major_status, StatusCode::INVALID_FALL_THROUGH);
}

#[test]
fn valid_fallthrough_branch() {
    let module = file_format::dummy_procedure_module(vec![], vec![Bytecode::Branch(0)]);
    let errors = CodeUnitVerifier::verify(&module);
    assert!(errors.is_empty());
}

#[test]
fn valid_fallthrough_ret() {
    let module = file_format::dummy_procedure_module(vec![], vec![Bytecode::Ret]);
    let errors = CodeUnitVerifier::verify(&module);
    assert!(errors.is_empty());
}

#[test]
fn valid_fallthrough_abort() {
    let module =
        file_format::dummy_procedure_module(vec![], vec![Bytecode::LdConst(7), Bytecode::Abort]);
    let errors = CodeUnitVerifier::verify(&module);
    assert!(errors.is_empty());
}

#[test]
fn valid_large_branching_factor_diamond() {
    use Bytecode::*;

    // "Difficulty" size for the
    const DUPLICATE_NUM: u64 = 100;
    // branching factor
    const NUM_BRANCHES: u16 = 10;

    // Configuration for branches/jumps
    const BLOCK_LEN: u16 = 5;
    const START: u16 = 40;
    const END: u16 = 90;

    let mut locals = vec![SignatureToken::Reference(Box::new(SignatureToken::U64))];
    (0..NUM_BRANCHES).for_each(|_| locals.push(SignatureToken::U64));
    let mut code = vec![];
    for i in 0..NUM_BRANCHES {
        code.append(&mut vec![LdConst(42), StLoc((i + 1) as u8)])
    }
    for i in 0..NUM_BRANCHES {
        code.append(&mut vec![LdFalse, BrTrue(START + (BLOCK_LEN * i))]);
    }
    assert!(code.len() == (START as usize));
    for i in 0..NUM_BRANCHES {
        let mut block = vec![
            ImmBorrowLoc((i + 1) as u8),
            StLoc(0),
            CopyLoc(0),
            Pop,
            Branch(END),
        ];
        assert!(block.len() == (BLOCK_LEN as usize));
        code.append(&mut block);
    }

    assert!(code.len() == (END as usize));
    (0..DUPLICATE_NUM).for_each(|_| code.push(CopyLoc(0)));
    (0..DUPLICATE_NUM).for_each(|_| code.push(Pop));
    code.push(Ret);

    let module = file_format::dummy_procedure_module(locals, code);
    let errors = CodeUnitVerifier::verify(&module);
    assert!(errors.is_empty());
}

#[test]
fn valid_large_branching_factor_line() {
    use Bytecode::*;

    // "Difficulty" size for the
    const DUPLICATE_NUM: u64 = 100;
    // branching factor
    const NUM_BRANCHES: u16 = 10;

    // Configuration for branches/jumps
    const BLOCK_LEN: u16 = 7;
    const START: u16 = 20;
    const END: u16 = 87;

    let mut locals = vec![SignatureToken::Reference(Box::new(SignatureToken::U64))];
    (0..NUM_BRANCHES).for_each(|_| locals.push(SignatureToken::U64));
    let mut code = vec![];
    for i in 0..NUM_BRANCHES {
        code.append(&mut vec![LdConst(42), StLoc((i + 1) as u8)])
    }
    let mut code = vec![];
    for i in 0..NUM_BRANCHES {
        code.append(&mut vec![LdConst(42), StLoc((i + 1) as u8)])
    }
    assert!(code.len() == (START as usize));

    for i in 1..NUM_BRANCHES {
        let mut block = vec![
            ImmBorrowLoc(i as u8),
            StLoc(0),
            CopyLoc(0),
            Pop,
            LdFalse,
            BrTrue(START + (BLOCK_LEN * i)),
            Branch(END),
        ];
        assert!(block.len() == (BLOCK_LEN as usize));
        code.append(&mut block);
    }
    let mut block = vec![ImmBorrowLoc(NUM_BRANCHES as u8), StLoc(0), CopyLoc(0), Pop];
    code.append(&mut block);
    assert!(code.len() == (END as usize));

    (0..DUPLICATE_NUM).for_each(|_| code.push(CopyLoc(0)));
    (0..DUPLICATE_NUM).for_each(|_| code.push(Pop));
    code.push(Ret);

    let module = file_format::dummy_procedure_module(locals, code);
    let errors = CodeUnitVerifier::verify(&module);
    assert!(errors.is_empty());
}
