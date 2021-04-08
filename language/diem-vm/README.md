---
id: vm-runtime
title: MoveVM Runtime
custom_edit_url: https://github.com/diem/diem/edit/main/language/move-binary-format/vm-runtime/README.md
---

# MoveVM Runtime

The MoveVM runtime is the verification and execution engine for the Move
bytecode format. The runtime is imported and loaded in 2 modes:
verification mode (by the [admission control](../../admission_control)
and [mempool](../../mempool) components) and execution mode (by the
[execution](../../execution) component).

## Overview

The MoveVM runtime is a stack machine. The VM runtime receives as input a
*block* which is a list of *transaction scripts* and a *data view*. The
data view is a **read only** snapshot of the data and code in the blockchain at
a given version (i.e., block height). At the time of startup, the runtime
does not have any code or data loaded. It is effectively *“empty”*.

Every transaction executes within the context of a [Diem
account](../../diem-framework/modules/diem_account.mvir)---specifically the transaction
submitter's account.  The execution of every transaction consists of three
parts: the account prologue, the transaction itself, and the account
epilogue. This is the only transaction flow known to the runtime, and it is
the only flow the runtime executes. The runtime is responsible to load the
individual transaction from the block and execute the transaction flow:

1. ***Transaction Prologue*** - in verification mode the runtime runs the
   bytecode verifier over the transaction script and executes the
   prologue defined in the [Diem account
   module](../../diem-framework/modules/diem_account.mvir). The prologue is responsible
   for checking the structure of the transaction and
   rejecting obviously bad transactions. In verification mode, the runtime
   returns a status of either `success` or `failure` depending upon the
   result of running the prologue. No updates to the blockchain state are
   ever performed by the prologue.
2. ***Transaction Execution*** - in execution mode, and after verification,
   the runtime starts executing transaction-specific/client code.  A typical
   code performs updates to data in the blockchain. Execution of the
   transaction by the VM runtime produces a write set that acts as an
   atomic state change from the current state of the blockchain---received
   via the data view---to a new version that is the result of applying the
   write set.  Importantly, on-chain data is _never_ changed during the
   execution of the transaction. Further, while the write set is produced as the
   result of executing the bytecode, the changes are not applied to the global
   blockchain state by the VM---this is the responsibility of the
   [execution module](../../../execution/).
3. ***Transaction Epilogue*** - in execution mode the epilogue defined in
   the [Diem account module](../../diem-framework/modules/diem_account.mvir) is
   executed to perform actions based upon the result of the execution of
   the user-submitted transaction. One example of such an action is
   debiting the gas fee for the transaction from the submitting account's
   balance.

During execution, the runtime resolves references to code by loading the
referenced code via the data view. One can think of this process as similar
to linking. Then, within the context of a block of transactions---a list of
transactions coupled with a data view---the runtime caches code and
linked and imported modules across transactions within the block.
The runtime tracks state changes (data updates) from one transaction
to the next within each block of transactions; the semantics of the
execution of a block specify that transactions are sequentially executed
and, as a consequence, state changes of previous transactions must be
visible to subsequent transactions within each block.

## Implementation Details

* The runtime top level structs are in `runtime` and `diem vm` related
  code.
* The transaction flow is implemented in the [`process_txn`](./src/process_txn.rs)
  module.
* Code caching logic and policies are defined under the [code
  cache](../../move-vm/runtime/src/code_cache/) directory.
* Runtime loaded code and the type system view for the runtime is defined
  under the [loaded data](src/loaded_data/) directory.
* The data representation of values, and logic for write set generation can
  be found under the [value](./src/value.rs) and [data
  cache](./src/data_cache.rs) files.

## Folder Structure

```
.
├── src                 # VM Runtime files
│   ├── code_cache      # VM Runtime code cache
│   ├── loaded_data     # VM Runtime loaded data types, runtime caches over code
│   ├── unit_tests      # unit tests
├── vm-cache-map        # abstractions for the code cache
```

## This Module Interacts With

This crate is mainly used in two parts: AC and mempool use it to determine
if it should accept a transaction or not; the Executor runs the MoveVM
runtime to execute the program field in a SignedTransaction and convert
it into a TransactionOutput, which contains a writeset that the
executor need to patch to the blockchain as a side effect of this
transaction.
