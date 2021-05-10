# Script Visibility

* Status: Implemented in Move 1.2

## Introduction

Script visibility is a new feature in Move version 1.2 to allow module functions to be safely and directly invoked much like scripts. Previously a function had either public or private visibility, where a public function is accessible anywhere but a private function can be called only from within the module where it is defined. A function with script visibility can be invoked directly, much like scripts, and is restricted to only being callable from scripts or other script visible functions.

## Motivations

### Managing the Hash-based Allow-list

In the Diem Framework, there was a hash-based allowlist of valid scripts that could be sent to the network. If the hash of a script was present in the list, the script could be executed. Otherwise, the transaction was rejected.

This solution worked but was rather cumbersome, especially if all the hash values change due to something like a new Move bytecode version. In an upgrade scenario, the allowlist would have to be upgraded in the same write-set transaction. While this difficult upgrade path could work, it would not scale well in the future. To the extent possible, Diem should continue to support old transactions, both to ease the upgrade process for clients and also to allow pre-signed transactions that are rarely updated (e.g., for emergency key rotations). The allowlist would then need to include all the old hash values from previous releases, and it would quickly grow in size and become hard to manage. Script visibility solves this problem by including the allowed scripts as part of the Diem Framework instead of tracking their hash values.

### Diem Framework Updates

At some time in the future, when Diem allows arbitrary transactions with script-functions, public APIs in the Diem Framework will be extremely difficult to change, but for now, it is still important to evolve the framework with critical changes that are incompatible with scripts from previous versions. This conflicts with the desire to continue supporting transactions from previous releases. With script visibility, the contents of the supported scripts can evolve along with the framework because the Move bytecodes for those scripts are not contained within the transactions, but are instead stored on chain as part of the framework.

### Meaningless Wrappers

Many of the scripts we have seen are simple wrappers around a single function call or two.  These scripts did not contain any interesting ad-hoc computation (and could not be doing so in the Diem Framework with its allow list). It would be convenient if a module writer could autogenerate scripts for certain functions, or simply mark which functions could be invoked directly, as if they were scripts.

## Description

The `script` visibility level solves these problems. In the previous version of Move, functions in a module could be declared as either `public` or private (`Public` or `Private` in the Move bytecode file format).  With this change, the possible visibility levels are now: private (no modifier), `public(friend)`, `public(script)`, and `public`. These respectively correspond to `Private`, `Friend`, `Script`, and `Public` in the file format. (See the [Friend Visibility](1-friend-visibility.md) change description for more details on that new feature.)

A `public(script)` function can only be called from 1) other `public(script)` functions, or 2) from transaction scripts. And, if the function has a signature that meets the necessary restrictions for a script-function, it can be invoked directly by the Move VM as if it was a script.

### New Visibility Modifier

`public(script)` is a new visibility modifier that can be applied to any module’s functions. A `public(script)` function can be invoked by any other `public(script)` function (whether or not it is in the same module) or by a script-function. Besides this visibility rule, `public(script)` functions follow the same rules as any other module function, meaning they can invoke private functions, create new struct instances, access global storage (of types declared in that module), etc.

Unlike script-functions, the signature of `public(script)` functions is not restricted. Any signature that is valid for another module function is a valid signature for a `public(script)` function. But, to be invoked by the Move VM as a script, the `public(script)` function must meet the same restrictions of a script-function. In other words, while every script-function has a restricted signature, the restrictions of a `public(script)` function are checked dynamically when used as an entry point to execution.

### New VM Entry Point

A new entry point `execute_script_function` is added to the VM to allow the invocation of a `public(script)` function in published modules. The entry point takes the following signature:

```
fn execute_script_function(
    &self,
    module: &ModuleId,
    function_name: &IdentStr,
    ty_args: Vec<TypeTag>,
    args: Vec<Vec<u8>>,
    senders: Vec<AccountAddress>,
    data_store: &mut impl DataStore,
    cost_strategy: &mut CostStrategy,
    log_context: &impl LogContext,
) -> VMResult<()>
```

The entry point is designed to be similar to the existing `execute_script` entry point, with only one change:

* argument `script: Vec<u8>` (i.e., a serialized script in raw bytes) is replaced by a pair of `module: &ModuleId` and `function_name: &IdentStr` that uniquely identifies a `public(script)` function in a published module (assuming the function exists).

The VM will reject the execution with proper status codes in the following situations:

* the `module` or `function_name` does not exist
* the function being pointed to is not a `public(script)` function
* the signature of the `public(script)` function does not pass the script signature check:
    * All `signer` arguments must occur before non-`signer` arguments
    * The function does not return any value
    * Each non-`signer` type in function type arguments is a valid type for a constant
        * Ostensibly, the type has the `copy` ability and is not a struct
    * Each type in the function type variables is closed, i.e., does not refer to other type variables
* the `senders`, `args`, or `ty_args` do not match the declared function signature.

## Examples

The feature is best used when you have a script that is a simple wrapper around a function in a module:

```
script {
    fun call_foo(account: signer, amount: u64) {
        0x42::M::foo(account, amount)
    }
}
```

Changing the module’s function from `public` to `public(script)` will remove the need for this simple wrapper script:

```
address 0x42 {
module M {
    ...
    // Replace previous "public" visibility...
    public(script) fun foo(account: signer, amount: u64) {
        ...
    }
}
}
```

However, keep in mind that the function can now only be called from other `public(script)` functions or script-functions.

```
address 0x43 {
module Other {
    fun private_call_foo(account: signer, amount: u64) {
        0x42::M::foo(account, amount) // ERROR Now invalid
    }

    public fun public_call_foo(account: signer, amount: u64) {
        0x42::M::foo(account, amount) // ERROR Now invalid
    }

    public(script) fun script_call_foo(account: signer, amount: u64) {
        0x42::M::foo(account, amount) // Still a valid call
    }
}
}

script {
    fun still_valid(account: signer) {
        0x42::M::foo(account, 0) // Still a valid call
    }
}
```

## Alternatives

We did not see many other options that would address the script versioning problem caused by the hash-based allowlist. Converting transaction scripts into `public(script)` functions that are published and change alongside the corresponding module was the most straightforward solution.

For the issue of meaningless wrappers alone, we considered compiler support to have scripts auto-generated. The generation by the compiler would be relatively simple, but it is unnecessary in the presence of `public(script)` functions.
