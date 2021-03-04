# Contributing to the Move standard library

This guide describes the process for adding, removing, and changing the Move modules and transaction scripts in the standard library.

## Overview

Every state change in the Diem blockchain occurs via executing a Move *transaction script* embedded in a [SignedTransaction](../../types/src/transaction/mod.rs). A transaction script invokes procedures of Move *modules* that update published *resources*. The Move standard library consists of:

1. The [modules](modules/) published in the genesis transaction.
2. The authorized [transaction scripts](transaction_scripts/) that can be included in a Diem transaction. A transaction with an unauthorized script will be discarded by validators.

## Environment Setup

Start by following the general Diem setup advice [here](../../CONTRIBUTING.md). Nothing else is strictly required, but you may want to consider a Move syntax highlighter for your editor (asking it to interpret `.move` files as Rust source is a decent start).

<!-- TODO: editor-specific suggestions, bash aliases -->

### Building

Execute

`cargo run`

inside `stdlib` to compile all of the standard library modules, transaction scripts, and supporting Rust wrappers. It is important to do this before running any tests that exercise your change.

### Testing

Most tests for the standard library live [here](../move-lang/functional-tests) and can be run with `cargo test`.

These tests use the Move functional testing framework, which we will briefly explain here (more details can be found in this [blog post](https://developers.diem.com/blog/2020/03/06/how-to-use-the-end-to-end-tests-framework-in-move).

A functional test is a sequence of Move transaction scripts that are executed against the genesis state of the blockchain. Tests typically call functions of the module under test and then use `assert`s to check that the call had the expected effect. The framework includes directives for checking that a transaction executed successfully (`// check: EXECUTED`) or aborted (e.g., `// check: ABORTED`). In addition, there are configuration macros (written `//!`) for creating accounts with human-readable names (`//! account: alice`), begining a new transaction (`//! new-transaction`), and setting the sender of a transaction (`//! sender: alice`).

The functional testing framework is very convenient, but can't express all of the tests we need to write. More heavyweight tests that create/execute transactions from Rust code live [here](../e2e-testsuite/src/tests) and can be run with `cargo test`.

## Changing the standard library

### Modules

- Add or edit the relevant `.move` file under [modules](modules/)
- [Build](#building) your changes and address compiler errors as needed
- Once the stdlib builds, add new functional [tests](#testing)
- If you have added a new `public` function that is intended to be called from a user-submitted transaction, make sure you introduce a new transaction script and add it to the allowlist (see [below](#transaction-scripts))

### Transaction Scripts

**Note**: The process for adding new transaction scripts to the system is in
flux. Below is the temporary. This will be updated once the new way of defining scripts is supported.

- Add or edit the relevant `.move` file under [transaction scripts](transaction_scripts)
- [Build](#building) your changes and address compiler errors as needed
- **Temporary**: a Rust builder for this script will be generated in [`tmp_new_transaction_builders.rs`](compiled/src/tmp_new_transaction_script_builders.rs). Note that this is not stable and will be going away soon.
- Add or modify tests for the script under the end-to-end [tests](../e2e-testsuite/src/tests/transaction_builder.rs). Make sure to use the `tmp_new_transaction_script_builders` builder for accessing the Rust builder for this script.
- If you have added a new script, don't forget to `git add` the new script binary

## Coding conventions

### Naming
- **Module names**: are camel case e.g., `DiemAccount`, `Diem`
- **Type names**: are camel case e.g., `WithdrawalCapability`, `KeyRotationCapability`
- **Function names**: are lower snake case e.g., `register_currency`
- **Constant names**: are upper snake case e.g., `TREASURY_COMPLIANCE_ADDRESS`
- Generic types should be descriptive, or anti-descriptive where appropriate (e.g. `T` for the Vector generic type parameter, `DiemAccount` for the core `DiemAccount` resource, `deposit<CoinType>(t: CoinType)` for depositing a token in the `Diem` module). Most of the time the "main" type in a module should be the same name as the module e.g., `Diem::Diem`, `DiemAccount::DiemAccount`.
- **Module file names**: are the same as the module name e.g., `DiemAccount.move`
- **Script file names**: should be lower snake case and named after the name of the “main” function in the script.
- **Mixed file names**: If the file contains multiple modules and/or scripts, the file name should be lower_snake_case, where the name does not match any particular module/script inside.

### Imports
- Functions and constants are imported and used fully qualified from the module in which they are declared, and not imported at the top level.
- Types are imported at the top-level. Where there are name clashes, `as` should be used to rename the type locally as appropriate.
 e.g. if there is a module
```rust
module Foo {
    resource struct Foo { }
    public const CONST_FOO: u64 = 0;
    public fun do_foo(): Foo { Foo{} }
    ...
}
```
this would be imported and used as:
```rust
module Bar {
    use 0x1::Foo::{Self, Foo};

    public fun do_bar(x: u64): Foo {
        if (x == Foo::CONST_FOO) {
            Foo::do_foo()
        } else {
            abort 0
        }
    }
    ...
}
```
And, if there is a local name-clash when importing two modules:
```rust
module OtherFoo {
    resource struct Foo {}
    ...
}

module Importer {
    use 0x1::OtherFoo::Foo as OtherFoo;
    use 0x1::Foo::Foo;
....
}
```


### Comments

- Each module, struct, resource, and public function declaration should be commented
- Move has both doc comments `///`, regular single-line comments `//`, and block comments `/* */`


## Formatting
We plan to have an autoformatter to enforce these conventions at some point. In the meantime...

- Four space indentation except for `script` and `address` blocks whose contents should not be indented
- Break lines longer than 100 characters
