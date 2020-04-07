# Contributing to the Move standard library

This guide describes the process for adding, removing, and changing the Move modules and transaction scripts in the standard library.

## Overview

Every state change in the Libra blockchain occurs via executing a Move *transaction script* embedded in a [SignedTransaction](../../types/src/transaction/mod.rs). A transaction script invokes procedures of Move *modules* that update published *resources*. The Move standard library consists of:

1. The [modules](modules/) published in the genesis transaction.
2. The authorized [transaction scripts](transaction_scripts/) that can be included in a Libra transaction. A transaction with an unauthorized script will be discarded by validators.

## Environment Setup

Start by following the general Libra setup advice [here](../../CONTRIBUTING.md). Nothing else is strictly required, but you may want to consider a Move syntax highlighter for your editor (asking it to interpret `.move` files as Rust source is a decent start).

<!-- TODO: editor-specific suggestions, bash aliases -->

### Building

Execute

`cargo run`

inside `stdlib` to compile all of the standard library modules, transaction scripts, and supporting Rust wrappers. It is important to do this before running any tests that exercise your change.

### Testing

Most tests for the standard library live [here](../move-lang/tests/functional) and can be run with `cargo test`.

These tests use the Move functional testing framework, which we will briefly explain here (more details can be found in this [blog post](https://developers.libra.org/blog/2020/03/06/how-to-use-the-end-to-end-tests-framework-in-move).

A functional test is a sequence of Move transaction scripts that are executed against the genesis state of the blockchain. Tests typically call functions of the module under test and then use `assert`s to check that the call had the expected effect. The framework includes directives for checking that a transaction executed successfully (`// check: EXECUTED`) or aborted (e.g., `// check: ABORTED`). In addition, there are configuration macros (written `//!`) for creating accounts with human-readable names (`//! account: alice`), begining a new transaction (`//! new-transaction`), and setting the sender of a transaction (`//! sender: alice`).

The functional testing framework is very convenient, but can't express all of the tests we need to write. More heavyweight tests that create/execute transactions from Rust code live [here](../e2e-tests/src/tests) and can be run with `cargo test`.

## Changing the standard library

### Modules

- Add or edit the relevant `.move` file under [modules](modules/)
- [Build](#building) your changes and address compiler errors as needed
- Once the stdlib builds, add new functional [tests](#testing)
- If you have added a new `public` function that is intended to be called from a user-submitted transaction, make sure you introduce a new transaction script and add it to the whitelist (see [below](#transaction-scripts))

### Transaction Scripts

- Add or edit the relevant `.move` file under [transaction scripts](transaction_scripts)
- [Build](#building) your changes and address compiler errors as needed
- If you have added a new script, extend the `StdlibScript` enum and script whitelist (`all()` function) [here](src/stdlib.rs). Don't forget to `git add` the compiled binary for the new script (`your_script.mv` [here](staged/transaction_scripts)).
- In addition, add a Rust wrapper for your script [here](../transaction-builder/src/lib.rs) to allow client code and tests to create the script.
- Add or modify tests for the script under the end-to-end [tests](../e2e-tests/src/tests/transaction_builder.rs)
- If you have added a new script, don't forget to `git add` the new script binary

## Coding conventions

### Naming
- Module names should be capitalized and camel case.
- File names should be lowercase-underscore; e.g., a module named `MyModule` should be stored in a file named `my_module.move`
- Modules that declare a single resource or struct type should name the type `T`. The reason for this convention is to avoid redundancy: outside of the module, the type will be referred to as `ModuleName::T`.
- If a module declares multiple resources/structs, the "main" type that clients of the module will use should be called `T` (this is obviously subjective).
- Function names should be lowercase-underscore
- Generic type parameter names should be long and descriptive (e.g., `deposit<Token>(t: Token)`). `T` should not be used as a generic type parameter name to avoid confusion with the resource/struct naming convention.

### Comments

- Each module, struct, resource, and public function declaration should be commented
- Note: Move only has `//` comments (no block or doc comments like `///`, `//!` or `/* */`) at the moment

## Formatting
We plan to have an autoformatter to enforce these conventions at some point. In the meantime...

- Four space identation
- Break lines longer than 100 characters
