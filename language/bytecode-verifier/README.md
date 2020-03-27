---
id: bytecode-verifier
title: Bytecode Verifier
custom_edit_url: https://github.com/libra/libra/edit/master/language/bytecode-verifier/README.md
---

# Bytecode Verifier

## Overview

The bytecode verifier contains a static analysis tool for rejecting invalid Move bytecode. It checks the safety of stack usage, types, resources, and references.

The body of each function in a compiled module is verified separately while trusting the correctness of function signatures in the module. Checking that each function signature matches its definition is a separate responsibility. The body of a function is a sequence of bytecode instructions. This instruction sequence is checked in several phases described below.

## CFG Construction

A control-flow graph is constructed by decomposing the instruction sequence into a collection of basic blocks. Each basic block contains a contiguous sequence of instructions; the set of all instructions is partitioned among the blocks. Each block ends with a branch or return instruction. The decomposition into blocks guarantees that branch targets land only at the beginning of some block. The decomposition also attempts to ensure that the generated blocks are maximal. However, the soundness of the analysis does not depend on maximality.

## Stack Safety

The execution of a block happens in the context of a stack and an array of local variables. The parameters of the function are a prefix of the array of local variables. Arguments and return values are passed across function calls via the stack. When a function starts executing, its arguments are already loaded into its parameters. Suppose the stack height is *n* when a function starts executing; then valid bytecode must enforce the invariant that when execution lands at the beginning of a basic block, the stack height is *n*. Furthermore, at a return instruction, the stack height must be *n*+*k* where *k*, s.t. *k*>=0 is the number of return values. The first phase of the analysis checks that this invariant is maintained by analyzing each block separately, calculating the effect of each instruction in the block on the stack height, checking that the height does not go below *n*, and that is left either at *n* or *n*+*k* (depending on the final instruction of the block and the return type of the function) at the end of the block.

## Type Safety

The second phase of the analysis checks that each operation, primitive or defined function, is invoked with arguments of appropriate types. The operands of an operation are values located either in a local variable or on the stack. The types of local variables of a function are already provided in the bytecode. However, the types of stack values are inferred. This inference and the type checking of each operation can be done separately for each block. Since the stack height at the beginning of each block is *n* and does not go below *n* during the execution of the block, we only need to model the suffix of the stack starting at *n* for type checking the block instructions. We model this suffix using a stack of types on which types are pushed and popped as the instruction stream in a block is processed. Only the type stack and the statically-known types of local variables are needed to type check each instruction.

## Resource Safety

Resources represent the assets of the blockchain. As such, there are certain restrictions on these types that do not apply to normal values. Intuitively, resource values cannot be copied and must be used by the end of the transaction (this means that they are moved to global storage or destroyed). Concretely, the following restrictions apply:

* `CopyLoc` and `StLoc` require that the type of local is not of resource kind.
* `WriteRef`, `Eq`, and `Neq` require that the type of the reference is not of resource kind.
* At the end of a function (when `Ret` is reached), no local whose type is of resource kind must be empty, i.e., the value must have been moved out of the local.

As mentioned above, this last rule around `Ret` implies that the resource *must* have been either:

* Moved to global storage via `MoveToSender`.
* Destroyed via `Unpack`.

Both `MoveToSender` and `Unpack` are internal to the module in which the resource is declared.

## Reference Safety

References are first-class in the bytecode language. Fresh references become available to a function in several ways:

* Inputing parameters.
* Taking the address of the value in a local variable.
* Taking the address of the globally published value in an address.
* Taking the address of a field from a reference to the containing struct.
* Returning value from a function.

The goal of reference safety checking is to ensure that there are no dangling references. Here are some examples of dangling references:

* Local variable `y` contains a reference to the value in a local variable `x`; `x` is then moved.
* Local variable `y` contains a reference to the value in a local variable `x`; `x` is then bound to a new value.
* Reference is taken to a local variable that has not been initialized.
* Reference to a value in a local variable is returned from a function.
* Reference `r` is taken to a globally published value `v`; `v` is then unpublished.

References can be either exclusive or shared; the latter allow read-only access. A secondary goal of reference safety checking is to ensure that in the execution context of the bytecode program, including the entire evaluation stack and all function frames, if there are two distinct storage locations containing references `r1` and `r2` such that `r2` extends `r1`, then both of the following conditions hold:

* If `r1` is tagged as exclusive, then it must be inactive, i.e. it is impossible to reach a control location where `r1` is dereferenced or mutated.
* If `r1` is shared, then `r2` is shared.

The two conditions above establish the property of referential transparency, important for scalable program verification, which looks roughly as follows: consider the piece of code `v1 = *r; S; v2 = *r`, where `S` is an arbitrary computation that does not perform any write through the syntactic reference `r` (and no writes to any `r'` that extends `r`). Then `v1 == v2`.

### Analysis Setup

The reference safety analysis is set up as a flow analysis (or abstract interpretation). An abstract state is defined for abstractly executing the code of a basic block. A map is maintained from basic blocks to abstract states. Given an abstract state *S* at the beginning of a basic block *B*, the abstract execution of *B* results in state *S'*. This state *S'* is propagated to all successors of *B* and recorded in the map. If a state already existed for a block, the freshly propagated state is “joined” with the existing state. If the join fails an error is reported. If the join succeeds but the abstract state remains unchanged, no further propagation is done. Otherwise, the state is updated and propagated again through the block. An error may also be reported when an instruction is processed during the propagation of abstract state through a block.

**Errors.** As mentioned earlier, an error is reported by the checker in one of the following situations:

* An instruction cannot be proven to be safe during the propagation of the abstract state through a block.
* Join of abstract states propagated via different incoming edges into a block fails.

## How is this module organized?

```text
*
├── invalid-mutations  # Library used by proptests
├── src                # Core bytecode verifier files
├── tests              # Proptests
```
