# Move Version 1.3

Version 1.3 of Move (released along with Diem Core version 1.3) introduces some syntax changes
to the Move language so that you may need to update Move source code when moving to this release.
The bytecode format remains the same as in version 1.2.

## Highlights

The main highlight of this release is a new language feature for unit testing.
This provides an easy way to test individual functions and features in Move.
More details are available in the unit testing [change description](changes/4-unit-testing.md).

## Move Language

In addition to the new unit testing feature, this release includes a few other changes to the
Move language:

* Added new module address syntax, e.g., `module 0x1::M`, to specify the address of a module
from within Move code
([#7915](https://github.com/diem/diem/pull/7915)).
This replaces the compiler's `--sender` option to specify the address on the command line.

* The syntax for an address value is changed to `@` followed by a number
([#8285](https://github.com/diem/diem/pull/8285)).
Previously an account address value was specified
as a hexadecimal value with an `0x` prefix, and hexadecimal values could not be
used as ordinary integer numbers. With this change, addresses and numbers can be
specified as either decimal and hexadecimal values, and the `@` prefix distinguishes
the address values.

* Introduced a general syntax for attributes in Move
([#8169](https://github.com/diem/diem/pull/8169)).
Move attributes are based on the Rust attribute syntax, which is in turn based on
the standards found in ECMA-334 and ECMA-335. Attributes can currently be attached to
address blocks, modules, scripts, and any module top level member. They are currently
used for unit testing, and other attributes may be defined in the future.

## Compiler

* Removed the compiler's `--sender` option. Instead of specifying the address on the command line,
you can use the new module address syntax in the Move code
([#7915](https://github.com/diem/diem/pull/7915)).
* Fixed crashes during internal testing with a precompiled standard library
when error messages reference the precompiled files
([#8344](https://github.com/diem/diem/pull/8344)).

## Prover

The syntax for specifications in Move is still in development and is
documented separately from the rest of the language. This release includes a
number of changes for Move specifications:

* Extended and renamed builtin functions
* New `let` binding semantics (`let x = E` and `let post y = E`)
* Support for axiom for constraining uninterpreted specification functions
* New `choose x where p` and `choose min i where p` expression forms
* New invariant syntax (`module M { invariant p; }`) for global invariants
* New syntax for function and struct specifications (`spec f` instead of `spec fun f`)
* New syntax for specification modules which can be put into separate files
* Removed `succeeds_if`
* Removed `invariant module`
* Removed `type<T>()` expression

In addition to the specification changes, the Move Prover has been improved with
bug fixes and some larger changes, including:

* Overhauled handling of global invariants
* Changed to perform monomorphization of generics in the Prover backend and memory model,
which has helped the Prover run faster and avoid timeouts.

## Standard Library

* Added a `BitVector` module
([#8315](https://github.com/diem/diem/pull/8315)).
* Added a `Vault` module for capability-based secure storage
([#8396](https://github.com/diem/diem/pull/8396)).

## VM

* Renamed `gas_schedule::CostStrategy` to `GasStatus` and cleaned up some of its APIs
([#7797](https://github.com/diem/diem/pull/7797)).
* Encapsulated both the `ChangeSet` and `AccountChangeSet` types so that their fields must be
accessed by API functions, which also enforce a new invariant that the `AccountChangeSet` is
not empty
([#8288](https://github.com/diem/diem/pull/8288)).
* Added a missing bounds check in the bytecode verifier for the `self_module_handle_idx` field
([#8389](https://github.com/diem/diem/pull/8389)).

## Miscellaneous

* Fixed the Move disassembler to work correctly with abilities
([#8128](https://github.com/diem/diem/pull/8128)).
* Renamed the `vm` Rust crate to `move-binary-format`,
which is a much better description of its contents.
([#8161](https://github.com/diem/diem/pull/8161)).
* Removed a number of dependencies on Diem crates,
continuing our effort to make Move usable apart from Diem.
* Added an `ident_str!` macro to create const `IdentStr` values
([#8300](https://github.com/diem/diem/pull/8300)).
* Refactored the `MoveResource` trait to add a separate `MoveStructType` trait
([#8346](https://github.com/diem/diem/pull/8346)).
* Added back the Move language documentation files, now in the `mdBook` format
([#8450](https://github.com/diem/diem/pull/8450)).
* Fixed the Move script binding generator so that the generated code is valid
when there are no transaction scripts or script functions
([#8465](https://github.com/diem/diem/pull/8465)).


# Move Version 1.2

Version 1.2 of Move (released along with Diem Core version 1.2) includes several new language features, a new version of the bytecode format, significant improvements to the Move Prover, and numerous bug fixes.

## Highlights

* Move Language Enhancements: This version of Move adds support for three new language features. Each of these is described in more detail in separate change descriptions.
    * [Friend Visibility](changes/1-friend-visibility.md): a new visibility modifier that allows a function to be called only by a set of declared `friend` modules.
    * [Script Visibility](changes/2-script-visibility.md): a new visibility modifier that allows a function to be called only from a transaction or another script function.
    * [Abilities](changes/3-abilities.md): a generalization of the existing `resource`/`struct` distinction to enable more fine-grained control over the operations allowed on a record value.
* Version 2 of the Move bytecode format: The bytecode format has been changed to support the new features. The Move VM still reads and processes older versions of the Move bytecode, but new bytecode files will require the new Move VM version.
* Move Prover: verification speed improvements of 2x and more via new internal architecture.

## VM

This release includes several changes and enhancements:

* Arguments to Move functions are now specified as BCS-serialized values ([#7170](https://github.com/diem/diem/pull/7170)) and the VM also returns serialized values ([#7599](https://github.com/diem/diem/pull/7599)). The VM’s `execute_function` API now returns the serialized return values ([#7671](https://github.com/diem/diem/pull/7671)).
* The VM’s file format deserializer now supports versioning so that it can seamlessly read multiple versions of Move bytecode files ([#7323](https://github.com/diem/diem/pull/7323)).
* The VM’s module publishing API now allows republishing an existing module, as long as the updated module is backward compatible with the previous version ([#7143](https://github.com/diem/diem/pull/7143)). This includes a new bytecode verifier check for module updates that introduce cyclic dependencies ([#7234](https://github.com/diem/diem/pull/7234)) and related checks for cyclic dependencies when building and loading the standard library ([#7475](https://github.com/diem/diem/pull/7475)).
* A new  `InternalGasUnits` type has been introduced to distinguish the unscaled units within the VM from the scaled `GasUnits` type ([#7448](https://github.com/diem/diem/pull/7448)).

**Fixed bugs:**

* Creating a normalized struct type now correctly uses the module handle associated with the `StructHandleIndex` rather than the module containing the declaration ([#7321](https://github.com/diem/diem/pull/7321)).
* The expected output files for internal tests no longer used colons in the file names, for the sake of file systems that do not support that ([#7770](https://github.com/diem/diem/issues/7770)).
* The `parse_type_tag` function can now handle struct names containing underscores ([#7151](https://github.com/diem/diem/issues/7151)).
* Missing signature checks for the `MoveToGeneric`, `ImmBorrowFieldGeneric`, and `MutBorrowFieldGeneric`  instructions have been added to the bytecode verifier ([#7752](https://github.com/diem/diem/pull/7752)).

## Standard Library

To make it easier to use Move for projects besides Diem, we are working toward separating the parts of Move that are specific to Diem. There is much more to do, but in this release, the standard library has been separated into two parts: `move-stdlib` ([#7633](https://github.com/diem/diem/pull/7633)) and `diem-framework` ([#7529](https://github.com/diem/diem/pull/7529)).

## Compiler

Besides adding support for the new language features mentioned above, the compiler in this release includes a number of fixes and usability enhancements:

* Attempting to use a global storage builtin, e.g., `move_to`, in a script context will no longer crash the compiler ([#4577](https://github.com/diem/diem/issues/4577)).
* Hex strings with an odd number of characters are no longer accepted by the compiler ([#6577](https://github.com/diem/diem/issues/6577)).
* A `let` binding with a name starting with an underscore, e.g., `_x`, can now be used later in the code: the underscore prefix merely disables the compiler diagnostic about unused locals ([#6786](https://github.com/diem/diem/pull/6786)).
* Fixed a compiler crash when a `break` is used outside of a loop ([#7560](https://github.com/diem/diem/issues/7560)).
* Added a missing check for recursive types when binding to a local variable, which fixed a compiler crash with a stack overflow ([#7562](https://github.com/diem/diem/issues/7562)).
* Fixed a compiler crash for an infinite loop with unreachable exits ([#7568](https://github.com/diem/diem/issues/7568)).
* Fixed a compiler crash due to an unassigned local used in an equality comparison ([#7569](https://github.com/diem/diem/issues/7569)).
* Fixed a compiler crash due to borrowing a divergent expression ([#7570](https://github.com/diem/diem/issues/7570)).
* Fixed a compiler crash due to a missing constraint for references in the type checker ([#7573](https://github.com/diem/diem/issues/7573)).
* Fixed a compiler crash related to expressions with short-circuiting ([#7574](https://github.com/diem/diem/issues/7574)).
* Fixed an incorrect code generation bug that could occur when a function parameter is assigned a new value exactly once in the function ([#7370](https://github.com/diem/diem/pull/7370)).
* Fixed the bytecode source map mapping from local names to indexes so that function parameters go before locals ([#7371](https://github.com/diem/diem/pull/7371)).
* Fixed a compiler crash when a struct is assigned without specifying its fields ([#7385](https://github.com/diem/diem/issues/7385)).
* Fixed a compiler crash when attempting to put a `spec` block inside a `spec` context ([#7387](https://github.com/diem/diem/issues/7387)).
* An integer literal value that is too large for its declared type will no longer cause a compiler crash ([#7388](https://github.com/diem/diem/issues/7388)).
* Fixed a compiler crash caused by incorrect number of type parameters in pack/unpack expressions ([#7401](https://github.com/diem/diem/pull/7401)).
* Module names and module members are now restricted from starting with underscores (‘_’) , which also avoids a crash ([#7572](https://github.com/diem/diem/issues/7572)).
* Prover specifications are now included in the compiler’s dependency ordering calculation ([#7960](https://github.com/diem/diem/pull/7960)).
* Modified the compiler optimization to remove fall-through jumps so that loop headers are not coalesced, which improves the prover’s ability to handle loop specifications ([#8049](https://github.com/diem/diem/pull/8049)).

## Command Line Interpreter (CLI)

The Move CLI has been enhanced in several ways:

* The CLI now supports safe module republishing with checks for breaking changes ([#6753](https://github.com/diem/diem/pull/6753)).
* Added a new `doctor` command to detect inconsistencies in storage ([#6971](https://github.com/diem/diem/pull/6971), [#7010](https://github.com/diem/diem/pull/7010), and [#7013](https://github.com/diem/diem/pull/7013)).
* The `publish` command’s `—-dry-run` option has been removed ([#6957](https://github.com/diem/diem/pull/6957)). Use the equivalent "check" command instead.
* The `test` command has a new `--create` option to create test scaffolding ([#6969](https://github.com/diem/diem/pull/6969)).
* The verbose output with the `-v` option now includes the number of bytes written ([#7757](https://github.com/diem/diem/pull/7757)).

## Other Tools

* Created a new bytecode-to-source explorer tool for Move ([#7508](https://github.com/diem/diem/pull/7508)).
* The resource viewer can now be better used to traverse data structures because the fields of `AnnotatedMoveStruct` are no longer private and `AnnotatedMoveValue::Vector` preserves the type information for its elements ([#7166](https://github.com/diem/diem/pull/7166)).
* The `diem-writeset-generator` and `diem-transaction-replay` tools have been significantly enhanced to support the process of upgrading the Diem Framework.
