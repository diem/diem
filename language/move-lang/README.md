---
id: move-lang
title: Move Source Language
custom_edit_url: https://github.com/libra/libra/edit/master/language/move-lang/README.md
---

## Code under this subtree is experimental. It is out of scope for the Libra Bug Bounty until it is no longer marked experimental.

# Move Source Language

## Summary

Move source language is an ergonomic language for writing Modules and Scripts that compile to Move bytecode.

## Overview

Move source language is an expression-based language that aims to simplify writing Move programs---modules and scripts---without hiding the core concepts in Move bytecode.

Currently, there are command line tools for Move.
* Move Check is used for checking code, but it does not generate bytecode
* Move Build is used for checking and then compiling to bytecode
In the future there should be other utilities for testing and play grounding the Move modules.

There is unfortunately no documentation for the language syntax or features. See the stdlib for examples.

## Command-line options

The two available programs are Move check and Move build.
* They can be built using `cargo build -p move-lang`
* Or run directly with
  * `cargo run -p move-lang --bin move-check -- [ARGS]`
  * `cargo run -p move-lang --bin move-build -- [ARGS]`


Move check is a command line tool for checking Move programs without producing bytecode

```text
move-check 0.0.1
Check Move source code, without compiling to bytecode.

USAGE:
    move-check [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -s, --sender <address>                             The sender address for modules and scripts
    -d, --dependencies <path-to-dependency-file>...    The library files needed as dependencies
    -f, --source-files <path-to-source-file>...        The source files to check/build
```

Move build is a command line tool for checking Move programs and producing serialized bytecode.
Dependencies will not be compiled.

```text
move-build 0.0.1
Compile Move source to Move bytecode.

USAGE:
    move-build [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -s, --sender <address>                             The sender address for modules and scripts
    -d, --dependencies <path-to-dependency-file>...    The library files needed as dependencies
    -o, --out-dir <path-to-output-directory>           The directory for outputing move bytecode [default: output]
    -f, --source-files <path-to-source-file>...        The source files to check and compile
```

## Folder Structure

```text
move-lang                                     # Main crate
├── src                                       # Source code for Move lang
│   ├── lib.rs                                # The entry points into compilation
|   |
│   ├── parser                                # Parsing the source input into an AST
│   │   ├── ast.rs                            # The target AST for Parsing
│   │   ├── mod.rs                            # Module for Parsing step
│   │   ├── lexer.rs                          # The lexer
│   │   └── syntax.rs                         # The parser
|   |
│   ├── expansion                             # Expands module aliases. Fixes syntax that could not be fully expressed in the grammar (such as assignments and pack)
│   │   ├── ast.rs                            # The target AST for Expansion
│   │   ├── mod.rs                            # Module for Expansion step
│   │   └── translate.rs                      # Parser ~> Expansion
|   |
│   ├── naming                                # Resolves names. This includes names in the current module, generics, locals, and builtin types/functions
│   │   ├── ast.rs                            # The target AST for Naming
│   │   ├── mod.rs                            # Module for Naming step
│   │   └── translate.rs                      # Expansion ~> Naming
|   |
│   ├── typing                                # Type checks the program. The checking is bidirectional in that it infers types while also checking them
│   |   ├── ast.rs                            # The target AST for Typing
│   |   ├── mod.rs                            # Module for Typing step
│   |   ├── translate.rs                      # Naming ~> Typing
│   |   ├── core.rs                           # Core type system code. This includes the typing context and rules for types
│   |   ├── expand.rs                         # After type inference, this expands all of the type variables with the inferred values
│   |   └── globals.rs                        # After expanding type variables, this checks proper access for resources (checks acquires)
|   |
│   ├── hlir                                  # The High Level IR. It changes the AST into a statement based representation as opposed to expression based
│   │   ├── ast.rs                            # The target AST for statement-ification
│   │   ├── mod.rs                            # Module for High Level IR step
│   │   └── translate.rs                      # Typing ~> High Level IR
|   |
│   ├── cfgir                                 # The Control Flow Graph IR. It removes the structured control flow and puts the blocks into a CFG. There are then control flow sensitive checks performed
│   │   ├── ast.rs                            # The target AST for the CFG-ification
│   │   ├── mod.rs                            # Module for CFG IR step
│   │   ├── translate.rs                      # High Level IR ~> CFG IR
│   │   ├── absint.rs                         # Abstract Interpretation library for control flow sensitive checks
│   │   ├── cfg.rs                            # Defines the CFG itself (where the AST just labels the blocks)
│   │   ├── locals                            # Checks proper local usage (no use after move, no resources left in locals)
│   │   │   ├── mod.rs                        # The module for the check. Includes the transfer functions
│   │   │   └── state.rs                      # The state used for abstract interpretation
│   │   └── borrows                           # The Borrow Checker. Checks the reference safety properties
│   │       ├── borrow_map.rs                 # The borrow graph used by the abstract state. Maintains internal relationships about what references are borrowing from where
│   │       ├── mod.rs                        # The module for the check. Includes the transfer functions
│   │       └── state.rs                      # The state used for abstract interpretation
|   |
│   ├── to_bytecode                           # Compilation to Move bytecode. Is not used by move-check
│   │   ├── mod.rs                            # Module for the compilation to bytecode
│   │   ├── translate.rs                      # CFG IR ~> Move bytecode
│   │   ├── context.rs                        # The context maps between IR construct and bytecode handles/offsets
│   │   ├── remove_fallthrough_jumps.rs       # The CFG IR blocks always end in a jump; Move bytecode blocks can fall through. This optimizes the usage of fallthroughs (removing unncessary jumps)
│   │   └── labels_to_offsets.rs              # During bytecode generation, the CFG IR labels are used. This switches the labels to bytecode offsets
|   |
│   ├── shared                                # Shared Utilities
│   │   ├── mod.rs                            # Shared utility code used by all modules (such as source location code)
│   │   └── unique_map.rs                     # A wrapper around BTreeMap that produces errors on duplicate values
|   |
│   ├── errors                                # Errors produced by the various checks
│   │   └── mod.rs                            # Module for Errors
|   |
│   ├── command_line                          # Utilities used by both command line binnaries
│   |   └── mod.rs                            # Module for Command LIne
|   |
│   └── bin                                   # Command line binaries
│       ├── move-check.rs                     # Defines the move-check command line tool
│       └── move-build.rs                     # Defines the move-build command line tool
|
└── stdlib                                    # Move standard library
    ├── modules                               # Core modules
    └── transaction_scripts                   # Core transaction scripts
```
