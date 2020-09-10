---
id: bytecode-test-generation
title: Bytecode Test Generation Tool
custom_edit_url: https://github.com/libra/libra/edit/master/language/testing-infra/test-generation/README.md
---

# Bytecode Test Generation Tool

## Overview

This tool can generate known-valid or known-invalid Move bytecode programs.
Known-valid means that the program is valid by construction; it was constructed
according to a formal specification of the Move bytecode language. Known-invalid
bytecode programs are those that diverge from the specification in a controlled way.

The generated modules are checked by the Move bytecode verifier and then run on the VM runtime.
If the module contains known-valid bytecode, we should expect that the program will
pass the bytecode verifier. If it does not pass the verifier then this indicates that either
there is a bug in the bytecode verifier or the formal specification of the failing
instruction is incorrect. Likewise, it should also run successfully on the VM runtime.

If the module contains known-invalid bytecode, we should expect that the verifier will
reject the module. If the verifier does not reject the module this indicates again
that either there is a bug in the bytecode verifier or the formal specification is
incorrect. Likewise for the VM runtime.

## Usage

### Building and Configuration

To build the tool
- `cargo build`

There are configuration options (and explanations for their behavior) in `src/config.rs`.
Most of the time these should be left as the provided defaults.

### Running

The tool expects up to two arguments:
- `--iterations`: The number of programs that should be generated and tested
- `--output`: (Optional) If provided, this is the path to which modules that result in errors will be serialied and saved. If not provided, failing cases will be logged to the console.

Additionally, there is an optional flag `RUST_LOG` that controls the verbosity of debug
information. It goes from `error`, the least information, to `debug` the most information.
The most common setting for this flag is `info` which will print stats such as the number
of iterations, the number of verified and executed programs, etc.

To run the tool
- `RUST_LOG=info cargo run -- --iterations N --output PATH`

## Architecture

This tool works by modeling the state of the VM abstractly, and by modeling the bytecode
instructions in terms of that abstract state. The abstract state is defined in
`abstact_state.rs`. It consists of type-level modeling of the VM stack, locals, and borrow
graph.

Instructions are defined in terms of their preconditions and effects. These definitions are
found in `summaries.rs` and use macros defined in `transitions.rs`. The preconditions of
an instruction are predicates that are true or false for a given abstract state. For example
the `Bytecode::Add` instruction requires the stack to contain two integers. This is modeled
by saying that the preconditions of `Bytecode::Add` are
- `state_stack_has!(0, Some(SignatureToken::U64))`
- `state_stack_has!(1, Some(SignatureToken::U64))`
where indexes 0 and 1 refer to the top two elements of the stack.

The effects of an instruction describe how the instruction modifies the abstract state. For
`Bytecode::Add` the effects are that it performs two pops on the stack and pushes a
`SignatureToken::U64` to the stack.

In this way, we are able to fully capture what each instruction needs and does.
This information is used to generate valid bytecode programs.

Generation of bytecode programs proceeds as follows:
1. In `lib.rs` the generator loop begins by initializing a `ModuleBuilder`
2. The `ModuleBuilder`, defined in `../utils/src/module_generator.rs`, generates a module definition
3. The `ModuleBuilder` calls the `generator` defined in `bytecode_generator.rs` to fill in function bodies within the module
4. The `generator` builds a control flow graph (CFG) in `control_flow_graph.rs`. Each block of the CFG is assigned a valid starting and ending abstract state.
5. The `generator` fills in blocks of the CFG according to the following algorithm:
    1. Given starting abstract state `AS1`, let `candidates` be the list of all instructions whose preconditions are all satisfied in `AS1`
        - If invalid generation is desired, then let `x` preconditions be `false`
    2. Select candidate `instr` from `candidates` according to the stack height heuristic
        - The stack height heuristic selects instructions that add to the stack when the height is small, and instructions that subtract from the stack when the height is large
    3. Apply the effects of `instr` to `AS1`, producing `AS2`
    4. If the stack is empty, terminate, otherwise repeat from step a with `AS2`

This results in the generation of one module. The module is then given to the bytecode
verifier and the bytecode verifier's behavior (i.e. no verification errors, some verification
errors, panic, crash) is recorded. Likewise with the VM runtime (except with runtime errors
rather than verification errors). Depending on the configuration that this tool is built
with, if the module caused a verification error, panic, or crash the module will then be
printed out or serialized to disk.

This will continue for the number of iterations specified when invoking the tool.

Other files:
- `error.rs` defines an error struct which is used to pass error messages.
- `tests/` contains a set of files that test the preconditions and effects of each bytecode instruction

## Extending the tool

The most common change or extension to this tool will probably be changing instruction
preconditions and effects. To do that follow these steps:
1. See if there already a macro defined in `transitions.rs` that captures your desired precondition/effect
2. If the macro is already defined, just add it to the summary of the instruction being changed in `summaries.rs`
3. If a suitable macro does not exist, define it in `transitions.rs`. Look at other macros in that file for examples.
4. Macros in `transitions.rs` have access to the public fields and functions of the `AbstractState`. If your macro needs access to something more, add a new helper method in `abstract_state.rs` and then invoke it in the macro.
