// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::unit_tests::testutils::compile_script_string;
use vm::file_format::Bytecode::*;

#[test]
fn compile_if() {
    let code = String::from(
        "
        main() {
            let x: u64;
            if (42 > 0) {
                x = 1;
            }
            return;
        }
        ",
    );

    let compiled_script_res = compile_script_string(&code);
    let compiled_script = compiled_script_res.unwrap();
    assert!(instr_count!(compiled_script, BrFalse(_)) == 1);
    assert!(instr_count!(compiled_script, Branch(_)) == 0);
}

#[test]
fn compile_if_else() {
    let code = String::from(
        "
        main() {
            let x: u64;
            let y: u64;
            if (42 > 0) {
                x = 1;
            } else {
                y = 1;
            }
            return;
        }
        ",
    );

    let compiled_script_res = compile_script_string(&code);
    let compiled_script = compiled_script_res.unwrap();
    assert!(instr_count!(compiled_script, BrFalse(_)) == 1);
    assert!(instr_count!(compiled_script, Branch(_)) == 1);
}

#[test]
fn compile_nested_if_else() {
    let code = String::from(
        "
        main() {
            let x: u64;
            if (42 > 0) {
                x = 1;
            } else {
                if (5 > 10) {
                    x = 2;
                } else {
                    x = 3;
                }
            }
            return;
        }
        ",
    );

    let compiled_script_res = compile_script_string(&code);
    let compiled_script = compiled_script_res.unwrap();
    assert!(instr_count!(compiled_script, BrFalse(_)) == 2);
    assert!(instr_count!(compiled_script, Branch(_)) == 2);
}

#[test]
fn compile_if_else_with_if_return() {
    let code = String::from(
        "
        main() {
            let x: u64;
            if (42 > 0) {
                return;
            } else {
                x = 1;
            }
            return;
        }
        ",
    );

    let compiled_script_res = compile_script_string(&code);
    let compiled_script = compiled_script_res.unwrap();
    assert!(instr_count!(compiled_script, BrFalse(_)) == 1);
    assert!(instr_count!(compiled_script, Branch(_)) == 0);
    assert!(instr_count!(compiled_script, Ret) == 2);
}

#[test]
fn compile_if_else_with_else_return() {
    let code = String::from(
        "
        main() {
            let x: u64;
            if (42 > 0) {
                x = 1;
            } else {
                return;
            }
            return;
        }
        ",
    );

    let compiled_script_res = compile_script_string(&code);
    let compiled_script = compiled_script_res.unwrap();
    assert!(instr_count!(compiled_script, BrFalse(_)) == 1);
    assert!(instr_count!(compiled_script, Branch(_)) == 1);
    assert!(instr_count!(compiled_script, Ret) == 2);
}

#[test]
fn compile_if_else_with_two_returns() {
    let code = String::from(
        "
        main() {
            if (42 > 0) {
                return;
            } else {
                return;
            }
            return;
        }
        ",
    );

    let compiled_script_res = compile_script_string(&code);
    let compiled_script = compiled_script_res.unwrap();
    assert!(instr_count!(compiled_script, BrFalse(_)) == 1);
    assert!(instr_count!(compiled_script, Branch(_)) == 0);
    assert!(instr_count!(compiled_script, Ret) == 3);
}

#[test]
fn compile_while() {
    let code = String::from(
        "
        main() {
            let x: u64;
            x = 0;
            while (copy(x) < 5) {
                x = copy(x) + 1;
            }
            return;
        }
        ",
    );
    let compiled_script_res = compile_script_string(&code);
    let compiled_script = compiled_script_res.unwrap();
    assert!(instr_count!(compiled_script, BrFalse(_)) == 1);
    assert!(instr_count!(compiled_script, Branch(_)) == 1);
}

#[test]
fn compile_while_return() {
    let code = String::from(
        "
        main() {
            while (42 > 0) {
                return;
            }
            return;
        }
        ",
    );
    let compiled_script_res = compile_script_string(&code);
    let compiled_script = compiled_script_res.unwrap();
    assert!(instr_count!(compiled_script, BrFalse(_)) == 1);
    assert!(instr_count!(compiled_script, Branch(_)) == 1);
    assert!(instr_count!(compiled_script, Ret) == 2);
}

#[test]
fn compile_nested_while() {
    let code = String::from(
        "
        main() {
            let x: u64;
            let y: u64;
            x = 0;
            while (copy(x) < 5) {
                x = move(x) + 1;
                y = 0;
                while (copy(y) < 5) {
                    y = move(y) + 1;
                }
            }
            return;
        }
        ",
    );
    let compiled_script_res = compile_script_string(&code);
    let compiled_script = compiled_script_res.unwrap();
    assert!(instr_count!(compiled_script, BrFalse(_)) == 2);
    assert!(instr_count!(compiled_script, Branch(_)) == 2);
}

#[test]
fn compile_break_outside_loop() {
    let code = String::from(
        "
        main() {
            break;
            return;
        }
        ",
    );
    let compiled_script_res = compile_script_string(&code);
    assert!(compiled_script_res.is_err());
}

#[test]
fn compile_continue_outside_loop() {
    let code = String::from(
        "
        main() {
            continue;
            return;
        }
        ",
    );
    let compiled_script_res = compile_script_string(&code);
    assert!(compiled_script_res.is_err());
}

#[test]
fn compile_while_break() {
    let code = String::from(
        "
        main() {
            while (true) {
                break;
            }
            return;
        }
        ",
    );
    let compiled_script_res = compile_script_string(&code);
    let compiled_script = compiled_script_res.unwrap();
    assert!(instr_count!(compiled_script, BrFalse(_)) == 1);
    assert!(instr_count!(compiled_script, Branch(_)) == 2);
}

#[test]
fn compile_while_continue() {
    let code = String::from(
        "
        main() {
            while (false) {
                continue;
            }
            return;
        }
        ",
    );
    let compiled_script_res = compile_script_string(&code);
    let compiled_script = compiled_script_res.unwrap();
    assert!(instr_count!(compiled_script, BrFalse(_)) == 1);
    assert!(instr_count!(compiled_script, Branch(_)) == 2);
}

#[test]
fn compile_while_break_continue() {
    let code = String::from(
        "
        main() {
            let x: u64;
            x = 42;
            while (false) {
                x = move(x) / 3;
                if (copy(x) == 0) {
                    break;
                }
                continue;
            }
            return;
        }
        ",
    );
    let compiled_script_res = compile_script_string(&code);
    let compiled_script = compiled_script_res.unwrap();
    assert!(instr_count!(compiled_script, BrFalse(_)) == 2);
    assert!(instr_count!(compiled_script, Branch(_)) == 3);
}

#[test]
fn compile_loop_empty() {
    let code = String::from(
        "
        main() {
            loop {
            }
            return;
        }
        ",
    );
    let compiled_script_res = compile_script_string(&code);
    let compiled_script = compiled_script_res.unwrap();
    assert!(instr_count!(compiled_script, Branch(_)) == 1);
}

#[test]
fn compile_loop_nested_break() {
    let code = String::from(
        "
        main() {
            loop {
                loop {
                    break;
                }
                break;
            }
            return;
        }
        ",
    );
    let compiled_script_res = compile_script_string(&code);
    let compiled_script = compiled_script_res.unwrap();
    assert!(instr_count!(compiled_script, Branch(_)) == 4);
}

#[test]
fn compile_loop_continue() {
    let code = String::from(
        "
        main() {
            loop {
                continue;
            }
            return;
        }
        ",
    );
    let compiled_script_res = compile_script_string(&code);
    let compiled_script = compiled_script_res.unwrap();
    assert!(instr_count!(compiled_script, Branch(_)) == 2);
}

#[test]
fn compile_loop_break_continue() {
    let code = String::from(
        "
        main() {
            let x: u64;
            let y: u64;
            x = 0;
            y = 0;

            loop {
                x = move(x) + 1;
                if (copy(x) >= 10) {
                    break;
                }
                if (copy(x) % 2 == 0) {
                    continue;
                }
                y = move(y) + copy(x);
            }

            return;
        }
        ",
    );
    let compiled_script_res = compile_script_string(&code);
    let compiled_script = compiled_script_res.unwrap();
    assert!(instr_count!(compiled_script, Branch(_)) == 3);
    assert!(instr_count!(compiled_script, BrFalse(_)) == 2);
}

#[test]
fn compile_loop_return() {
    let code = String::from(
        "
        main() {
            loop {
                loop {
                    return;
                }
                return;
            }
            return;
        }
        ",
    );
    let compiled_script_res = compile_script_string(&code);
    let compiled_script = compiled_script_res.unwrap();
    assert!(instr_count!(compiled_script, Branch(_)) == 2);
    assert!(instr_count!(compiled_script, Ret) == 3);
}
