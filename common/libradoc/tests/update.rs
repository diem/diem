// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_documentation_tool as libradoc;
use serde::Deserialize;
use serde_generate::rust;
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
    let definitions = rust::quote_container_definitions(&registry).unwrap();

    let input = r#"
<!-- @begin-libradoc name=Unknown -->
<!-- @end-libradoc -->
111111
<!-- @begin-libradoc name=MyStruct -->
222222
<!-- @end-libradoc -->
<!-- @begin-libradoc name=MyEnum -->
<!-- @end-libradoc -->
33333333
"#
    .to_string();

    let expected_output = r#"
<!-- @begin-libradoc name=Unknown -->
<!-- @end-libradoc -->
111111
<!-- @begin-libradoc name=MyStruct -->
```rust
struct MyStruct(u64);
```
<!-- @end-libradoc -->
<!-- @begin-libradoc name=MyEnum -->
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
<!-- @end-libradoc -->
33333333
"#
    .to_string();

    let reader = std::io::BufReader::new(input.as_bytes());
    assert_eq!(
        libradoc::update_rust_quotes(reader, &definitions).unwrap(),
        expected_output
    );
}
