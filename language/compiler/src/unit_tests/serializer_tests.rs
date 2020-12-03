// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::unit_tests::testutils::compile_script_string;

#[test]
fn serialize_script_ret() {
    let code = String::from(
        "
        main() {
            return;
        }
        ",
    );
    let compiled_script_res = compile_script_string(&code);
    let compiled_script = compiled_script_res.unwrap();
    let mut binary: Vec<u8> = Vec::new();
    let res = compiled_script.serialize(&mut binary);
    assert!(res.is_ok());
    println!("SCRIPT:\n{:?}", compiled_script);
    println!("Serialized Script:\n{:?}", binary);
    println!("binary[74]: {:?}", binary.get(74));
    println!("binary[76]: {:?}", binary.get(76));
    println!("binary[79]: {:?}", binary.get(79));
    println!("binary[82]: {:?}", binary.get(82));
    println!("binary[84]: {:?}", binary.get(84));
    println!("binary[96]: {:?}", binary.get(96));
    println!("binary[128]: {:?}", binary.get(128));
    println!("binary[75]: {:?}", binary.get(75));
    println!("binary[77]: {:?}", binary.get(77));
    println!("binary[80]: {:?}", binary.get(80));
    println!("binary[83]: {:?}", binary.get(83));
    println!("binary[85]: {:?}", binary.get(85));
    println!("binary[97]: {:?}", binary.get(97));
    println!("binary[129]: {:?}", binary.get(129));
    // println!("SCRIPT:\n{}", compiled_script);
}
