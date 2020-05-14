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

## Design Principles

### Mission

Deliver a minimalistic, expressive, safe, and transparent language to produce--and link with--Move bytecode.

### Primary Principles

* **More Concise than Bytecode** Move is expression based, which lends itself to concise and composed programs without the need for extra locals or structure. The Move bytecode is stack based (with the addition of local variables), so a language without stack access would need to be more verbose than the bytecode. In the Move source language, expressions allow for programming directly on the stack in a controlled and safe mode, and in that way, the language gives the same level of functionality as the bytecode but in a more concise and readable environment.

* **Move Bytecode Transparency** The Move source language tries to lift up concepts in the Move bytecode into a source language; it is not trying to hide them. The bytecode already has some strong opinions (much stronger than you might expect to find in a bytecode language), and the source language is trying to keep that programming model and line of thinking. The intention of this principle is to remove the need to write bytecode directly. Additionally, this means full interoperability with functions and types declared in published modules.

* **Stricter than Bytecode** The source language often adds additional levels of restrictions. At an expression level, this means no arbitrary manipulation of the stack (only can do so through expressions), and no dead code or unused effects. At a module level, this could mean additional warnings for unused types or un-invocable functions. At a conceptual/program level, this will also mean adding integration for formal verification.

### Secondary Principles

* **Pathway of Learning** Syntax choices and error messages are intended to give a natural flow of learning. For example, some of the choices around expression syntax could be changed to be more familiar to various other languages, but they would hurt the plug-n-play feeling of the expression based syntax, which might hurt developing a deeper understanding of the Move source language.

* **Aiding Common Community Patterns** As Move becomes more heavily used, common patterns for modules are likely to appear. Move might add new language features to make these patterns easier, clearer, or safer. But, they will not be added if it violates some other key design goal/principle of the language.

* **Semantic Preserving Optimizations** Optimizations are an important developer tool, as they let a programmer write code in a more natural way. However, all of the optimizations performed must be semantic preserving, to prevent any catastrophic exploits or errors from occurring in optimized code. That being said, it is not the primary goal of the Move source language to produce *heavily* optimized code, but it is a nice feature to have.

### Non-Principles

* **Heavy Abstractions** The Move source language does not intend to hide the details of the Move bytecode, this ranges from everything of references to global storage. There might be some abstractions that make interacting with these items easier, but they should always be available in Move at their most basic (bytecode equivalent) level. This does not mean that conveniences currently given by the source language, such as easy field access or implicit freezing, are against the core set of principles, but only that conveniences should not be ambiguous or opaque in how they interact at the bytecode level. Note though, this does not preclude the addition of features to the lanugage, such as access modifiers that translate to compiler-generated dynamic checks. It is just that it is not an active goal of the language to add on heavy abstractions just for the sake of obscuring bytecode design choices.

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
    move-check [OPTIONS] [--] [PATH_TO_SOURCE_FILE]...

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -s, --sender <ADDRESS>                           The sender address for modules and scripts
    -d, --dependency <PATH_TO_DEPENDENCY_FILE>...    The library files needed as dependencies

ARGS:
    <PATH_TO_SOURCE_FILE>...    The source files to check
```

Move build is a command line tool for checking Move programs and producing serialized bytecode.
Dependencies will not be compiled.

```text
move-build 0.0.1
Compile Move source to Move bytecode.

USAGE:
    move-build [FLAGS] [OPTIONS] [--] [PATH_TO_SOURCE_FILE]...

FLAGS:
    -m, --source-map    Save bytecode source map to disk
    -h, --help          Prints help information
    -V, --version       Prints version information

OPTIONS:
    -s, --sender <ADDRESS>                           The sender address for modules and scripts
    -d, --dependency <PATH_TO_DEPENDENCY_FILE>...    The library files needed as dependencies
    -o, --out-dir <PATH_TO_OUTPUT_DIRECTORY>         The Move bytecode output directory [default: move_build_output]

ARGS:
    <PATH_TO_SOURCE_FILE>...    The source files to check and compile
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
