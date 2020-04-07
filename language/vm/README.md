---
id: vm
title: Virtual Machine
custom_edit_url: https://github.com/libra/libra/edit/master/language/vm/README.md
---

# MoveVM Core

The MoveVM executes transactions expressed in the Move bytecode. There are
two main crates: the core VM and the VM runtime. The VM core contains the low-level
data type for the VM - mostly the file format and abstraction over it. A gas
metering logical abstraction is also defined there.

## Overview

The MoveVM is a stack machine with a static type system. The MoveVM honors
the specification of the Move language through a mix of file format,
verification (for reference [bytcode verifier README](https://github.com/libra/libra/blob/master/language/bytecode-verifier/README.md))
and runtime constraints. The structure of the file format allows the
definition of modules, types (resources and unrestricted types), and
functions. Code is expressed via bytecode instructions, which may have
references to external functions and types.  The file format also imposes
certain invariants of the language such as opaque types and private fields.
From the file format definition it should be clear that modules define a
scope/namespace for functions and types. Types are opaque given all fields
are private, and types carry no functions or methods.

## Implementation Details

The MoveVM core crate provides the definition of the file format and all
utilities related to the file format:
* A simple Rust abstraction over the file format
  (`libra/language/vm/src/file_format.rs`) and the bytecodes. These Rust
  structures are widely used in the code base.
* Serialization and deserialization of the file format. These define the
  on-chain binary representation of the code.
* Some pretty printing functionalities.
* A proptest infrastructure for the file format.

The `CompiledModule` and `CompiledScript` definitions in
`libra/language/vm/src/file_format.rs` are the top-level structs for a Move
*Module* or *Transaction Script*, respectively. These structs provide a
simple abstraction over the file format. Additionally, a set of
[*Views*](https://github.com/libra/libra/blob/master/language/vm/src/views.rs) are defined to easily navigate and inspect
`CompiledModule`s and `CompiledScript`s.

## Folder Structure

```
.
├── src             # VM core files
├── tests           # Proptests
└── vm-runtime      # Interpreter and runtime data types (see README in that folder)
```
