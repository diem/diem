---
id: move-language
title: Move Language
custom_edit_url: https://github.com/libra/libra/edit/master/language/README.md
---

# Move

Move is a new programming language developed to provide a safe and programmable foundation for the Libra Blockchain.

## Overview

The Move language directory consists of five parts:

- [virtual machine](vm/) (VM) &mdash; contains the bytecode format, a bytecode interpreter, and infrastructure for executing a block of transactions. This directory also contains the infrastructure to generate the genesis block.

- [bytecode verifier](bytecode-verifier/) &mdash; contains a static analysis tool for rejecting invalid Move bytecode. The virtual machine runs the bytecode verifier on any new Move code it encounters before executing it. The compiler runs the bytecode verifier on its output and surfaces the errors to the programmer.

- [compiler](compiler/) &mdash; contains the Move intermediate representation (IR) compiler which compiles human-readable program text into Move bytecode. *Warning: the IR compiler is a testing tool. It can generate invalid bytecode that will be rejected by the Move bytecode verifier. The IR syntax is a work in progress that will undergo significant changes.*

- [standard library](stdlib/) &mdash; contains the Move IR code for the core system modules such as `LibraAccount` and `LibraCoin`.

- [tests](functional_tests/) &mdash; contains the tests for the virtual machine, bytecode verifier, and compiler. These tests are written in Move IR and run by a testing framework that parses the expected result of running a test from special directives encoded in comments.

## How the Move Language Fits Into Libra Core

Libra Core components interact with the language component through the VM. Specifically, the [admission control](../admission_control/) component uses a limited, read-only [subset](../vm_validator/) of the VM functionality to discard invalid transactions before they are admitted to the mempool and consensus. The [execution](../execution/) component uses the VM to execute a block of transactions.

## Exploring Move IR

* You can find many small Move IR examples in the [tests](functional_tests/tests/testsuite) directory. The easiest way to experiment with Move IR is to create a new test in this directory and follow the instructions for running the tests.
* More substantial examples can be found in the [standard library](stdlib/modules) directory. The two notable ones are [LibraAccount.mvir](stdlib/modules/libra_account.mvir), which implements accounts on the Libra blockchain, and [LibraCoin.mvir](stdlib/modules/libra_coin.mvir), which implements Libra coin.
* The four transaction scripts supported in the Libra testnet are also in the standard library directory. They are [peer-to-peer transfer](stdlib/transaction_scripts/peer_to_peer_transfer.mvir), [account creation](stdlib/transaction_scripts/create_account.mvir), [minting new Libra](stdlib/transaction_scripts/mint.mvir), and [key rotation](language/stdlib/transaction_scripts/rotate_authentication_key.mvir). The transaction script for minting new Libra will only work for an account with proper privileges.
* Refer to the [IR compiler README](compiler/README.md) for more details on writing Move IR code.

## How is this folder organized?

```
├── README.md          # This README
├── benchmarks         # Benchmarks for the Move language VM and surrounding code
├── bytecode-verifier  # The bytecode verifier
├── e2e-tests          # Infrastructure and tests for the end-to-end flow
├── functional_tests   # Testing framework for the Move language
├── compiler           # The IR to Move bytecode compiler
├── stdlib             # Core Move modules and transaction scripts
├── test.sh            # Script for running all the language tests
└── vm
    ├── src            # Bytecode language definitions, serializer, and deserializer
    ├── tests          # VM tests
    └── vm-runtime     # The bytecode interpreter
```
