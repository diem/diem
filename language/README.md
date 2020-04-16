---
id: move-language
title: Move Language
custom_edit_url: https://github.com/libra/libra/edit/master/language/README.md
---

# Move

Move is a new programming language developed to provide a safe and programmable foundation for the Libra Blockchain.

## Overview

The Move language directory consists of four main parts:

- [virtual machine](vm/) (VM) &mdash; contains the bytecode format, a bytecode interpreter, and infrastructure for executing a block of transactions. This directory also contains the infrastructure to generate the genesis block.

- [bytecode verifier](bytecode-verifier/) &mdash; contains a static analysis tool for rejecting invalid Move bytecode. The virtual machine runs the bytecode verifier on any new Move code it encounters before executing it. The compiler runs the bytecode verifier on its output and surfaces the errors to the programmer.

- [move-lang](move-lang/) &mdash; contains the Move source language compiler.

- [standard library](stdlib/) &mdash; contains the Move code for the core system modules (such as `LibraAccount`), as well as the standard library transaction scripts.

## How the Move Language Fits Into Libra Core

Libra Core components interact with the language component through the VM. Specifically, the [admission control](../admission_control/) component uses a limited, read-only [subset](../vm_validator/) of the VM functionality to discard invalid transactions before they are admitted to the mempool and consensus. The [execution](../execution/) component uses the VM to execute a block of transactions.

## Exploring the Move language

- You can find many small Move examples in the [tests](move-lang/tests/functional/) directory. The easiest way to experiment with Move is to create a new test in this directory and run it with `cargo test`.
- More substantial examples can be found in the [standard library](stdlib/modules) directory. The Two particularly notable ones are [LibraAccount](stdlib/modules/libra_account.move), which implements accounts on the Libra blockchain, and [Libra](stdlib/modules/libra.move), which implements generic currency logic used by all of the currencies the Libra payment network supports.
- The transaction scripts supported in the Libra blockchain are also in the standard library directory. Move tests and local instances of the Libra blockchain can execute arbitrary transaction scripts, but the Libra blockchain and testnet are limited to accepting the scripts in this directory.
