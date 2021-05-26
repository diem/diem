
# Documentation Generator

This is the description of the documentation generator which is part of the Move prover project.

The generator provides a simple documentation generator for Move including Move specifications. The documentation
generator is embedded in the Move Prover. This is not only for technical reasons but also because eventually,
we want to use the Prover to derive information for docs using formal reasoning, as well as mark verification
results in the generated docs.

## Calling the Generator

The generator is called from within the Diem tree as such:

```shell script
> cargo run -p move-prover -- --docgen <flags> .. <sources>
```

... where most commonly used flags are:

-  `-d=<path>`: search path for Move dependencies. For Move compilation.
-  `--doc-path=<path>`: search path for already generated documentation of Move code; for cross-references.
-  `--doc-spec-inline=true|false`: whether specs should be included together with function declarations or in
    a separate section at the end of the document. Default is true.
-  `--doc-include-impl=true|false`: whether function implementation bodies shall be included. Default is true.
-  `--doc-include-private=true|false`: whether private functions shall be included. Default is false.
-  `--output=<path>`: file where to store generated markdown.

For full command line help, use `cargo run -p move-prover -- --help`.

## Guide for Documentation Writers

### Documentation Comments

Documentation comments in the Move source start either with `///` or `/**`. Like in many similar other tools,
documentation comments must be placed before the item being documented. Currently, the following items in the
Move source can have documentation comments:

-  Modules
-  Structs
-  Struct fields
-  Functions
-  Spec blocks
-  Individual spec block members

A series of comments in front of one item is collapsed into on documentation block. For example, the following
fragments are all equivalent and associate documentation with function `f`:

```move
struct T {}
/// This is a documentation comment for `f`.

/// This is another documentation comment for `f`.
fun f() { ... }
```

```move
struct T {}
/// This is a documentation comment for `f`.
/// This is another documentation comment for `f`.
fun f() { ... }
```

```move
struct T {}
/// This is a documentation comment for `f`.
/** This is another documentation comment for `f`. */
fun f() { ... }
```

```move
struct T {}
/**
This is a documentation comment for `f`.
This is another documentation comment for `f`.
*/
fun f() { ... }
```

### Markdown in Comments

Documentation comments can use arbitrary markdown (we recommend to use a Markdown flavor which is compatible with
Github). One can also use section headers in documentation comments; those headers are placed one level underneath
the context in which they are included in the overall doc. For example:

```move
/// This is the Diem account module
///
/// # Overview
///
/// This module does fancy things!
///
/// # Details
///
/// The following details need to be considered:
/// ...
module DiemAccount {
   ...
}
```

If the module documentation is included in a larger context, the section tags will be adjusted in the generated
doc:

```move
# Module `0x1::DiemAccount`

This is the Diem account module.

## Overview
...

## Details
...
```

### Code Decoration

Code (either in single back-quotes inline of the markdown text, or code which is produced by the generator itself) is
decorated as follows:

-  Keywords are highlighted. Note since specifically the Move spec language has a number of "weak" keywords (identifiers
   which play a special role in a particular syntactic context but are not reserved), highlighting may have some false
   positives, as the generator does not analyze the syntax right now.

-  Identifiers are attempted to resolve against the documented code and on success, hyperlinked to the declaration.
   For example, within the `DiemAccount` module, all occurences of `T`, `Self::T`, `DiemAccount::T`, and
   `0x1::DiemAccount:T` will resolve into a link to the declaration. This resolution is heuristic and may have
   positive and negative false positives. Specifically, it currently does not consider aliases and use-declarations.

   If you use a simple name in code comments. like `foo`, it will not resolve against a function `foo` in the current
   module unless it is either followed by `(` or `<`. This to avoid false positives of matching functions. You can
   use `foo()` or `Self::foo` instead.

### Organization of Specification Blocks

Specification blocks can be placed anywhere in the Move source, which creates a certain challenge to place them
in the appropriate place in the documentation. This specifically applies to schema and module spec blocks, which
have no specified target like a function or struct spec block.

The documentation generator associates all schema and module spec blocks with any **preceding** function or struct
spec block, or, if no previous one exists, with the module. Example:

```move
module M {
    spec module {
        // Associated with the module.
    }
    spec schema Some {
        // Associated with module.
    }
    struct S { .. }
    spec schema Other {
        // Gotcha! Still associated with module, as there is no struct spec block before.
    }
    spec S {
        // Associated with S because of named spec block target. Notice that this can be in fact anywhere in
        // the file (before declaration of S, at the end of the file, etc.).
    }
    spec schema YetAnother {
        // Associated with S because of preceding spec block.
    }
    spec module {
        // Associated with S because of preceding spec block.
    }
    fun f() { ... }
    spec schema FAbortsIf {
        // Gotcha! The last explicitly targeted spec block was for S, not for f, so this will go with S.
    }
    spec f {
        // Associated with f because of named spec block target.
    }
    spec schema FEnsures {
        // Associated with f because of preceding spec block.
    }
}
```

Notice that one can enforce an association of subsequent spec block to the module level by introducing a dummy, empty
`spec module` block. Because such blocks don't declare properties, they do not appear in the generated docs:

```move
module M {
    fun f(): T { ... }
    spec f {
       aborts_if f_aborts();
       ensures result == f_result();
    }
    // This goes with the documentation of function f
    spec fun f_aborts() { .. }
    spec module {}
    // This goes with the documentation of the module
    spec fun f_result(): T { .. }
}
```
