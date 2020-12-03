// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use diem_documentation_tool as diem_doc;
use serde::Deserialize;
use serde_reflection::{Samples, Tracer, TracerConfig};

#[allow(dead_code)]
#[derive(Deserialize)]
enum MyEnum {
    Unit,
    Newtype(MyStruct),
    Tuple(u16, Option<bool>),
    Struct { a: u32 },
    NewTupleArray((u16, u16, u16)),
}

#[derive(Deserialize)]
struct MyStruct(u64);

#[test]
fn test_doctool() {
    let mut tracer = Tracer::new(TracerConfig::default());
    let samples = Samples::new();
    tracer.trace_type::<MyEnum>(&samples).unwrap();
    let registry = tracer.registry().unwrap();
    let definitions = diem_doc::quote_container_definitions(&registry).unwrap();

    let input = r#"
<!-- @begin-diemdoc name=Unknown -->
<!-- @end-diemdoc -->
111111
<!-- @begin-diemdoc name=MyStruct -->
222222
<!-- @end-diemdoc -->
<!-- @begin-diemdoc name=MyEnum -->
<!-- @end-diemdoc -->
33333333
"#
    .to_string();

    let expected_output = r#"
<!-- @begin-diemdoc name=Unknown -->
<!-- @end-diemdoc -->
111111
<!-- @begin-diemdoc name=MyStruct -->
```rust
struct MyStruct(u64);
```
<!-- @end-diemdoc -->
<!-- @begin-diemdoc name=MyEnum -->
```rust
enum MyEnum {
    Unit,
    Newtype(MyStruct),
    Tuple(u16, Option<bool>),
    Struct {
        a: u32,
    },
    NewTupleArray([u16; 3]),
}
```
<!-- @end-diemdoc -->
33333333
"#
    .to_string();

    let reader = std::io::BufReader::new(input.as_bytes());
    assert_eq!(
        diem_doc::update_rust_quotes(reader, &definitions).unwrap(),
        expected_output
    );
}
