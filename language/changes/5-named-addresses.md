# Named Addresses

- Status: Implemented in Move 1.4, updated in Move 1.5

## Introduction

Named addresses are a new source language only feature. (That is, the named address feature is
compiled away and not present in the Move bytecode format). The feature allows names to be used in
place of numerical values in any spot where addresses are used. Named addresses are declared as top
level elements (outside of modules and scripts) with `address MyAddr;` and can be assigned like so
`address MyAddr = 0x42;`

With the landing of this feature, Move standard library modules now reside under the address `Std`,
e.g. `Std::Vector`. Similarly, Diem Framework modules now reside under the address `DiemFramework`,
e.g. `DiemFramework::XUS`.

Named address declarations are opaque, meaning they must be accessed via the name and not their
underlying numeric value. Any existing code must be updated to use named addresses when accessing
standard library or Diem Framework modules.

Alongside this change, there is new syntax for address literals when used as expression values, e.g.
`let addr = @0x42;`.

## Motivations

Fixed, numerical addresses were "good enough" for the language starting off, but the inability to
set addresses via a configuration at build time severely hinders code portability and usability.
Additionally, the lack of named address support has been painful both for account configuration in
testing and for basic readability of code.

To combat this, we are adding named addresses. They compile down to the same address system that
exists today, but it greatly increases the portability, testability, and readability of source
language programs.

## Description

### New Address Literal Syntax

Addresses now come in two flavors, named or numerical. The syntax for a named address follows the
same rules for any named identifier in Move. The syntax of a numerical address is no longer
restricted to `0x`-prefixed values, and now any valid numerical literal can be used.

To make room for the named address feature, address expression values have a new syntax. This new
syntax reduces the complexity around named addresses as it prevents shadowing issues around module
members (specifically constants) and local variables.

In the old syntax, all address values began with `0x` and this hex prefix could not be used for
integer values. In the new syntax, all address values are prefixed with `@`. Following the `@`
operator any valid address can be used. For example:

```move
let _: u8 = 0x1u8;
let _: u64 = 0x42u64;
let _: u128 = 0x42u128;
let a1: address = @Std;
let a2: address = @66;
let a3: address = @0x42;
```

You can think of `@` as an operator that takes an address from being a namespace item to an
expression item.

Named addresses are not declared in Move source code. Instead they
must be declared---and given a value---when invoking the Move compiler. E.g.,

```bash
cargo run --bin move-build --addresses MyAddr=0x42 ...
```

A named address can be used in both module accesses and as expression values (with the new
`@` syntax)

```move
script {
    fun example() {
        MyAddr::M::foo(@MyAddr);
    }
}
```

A named address can be used multiple times throughout a program.

```move
// file1.move
module MyAddr::M {
    ...
}
```

```move
// file2.move
address MyAddr {
module N {
    ...
}
}
```

### Assigning Named Addresses

Named addresses can only be assigned a value by passing their value as a parameter to the compiler with `<addr_name>=<number value>`:

```bash
cargo run --bin move-build --addresses MyAddr=0xC0FFEE ...
```

An address can be assigned any number of times on the command line as long as it is given only _one_ value.
The following would be fine, since the address `MyAddr` is given the same value in both assignments:

```bash
cargo run --bin move-build --addresses MyAddr=0xC0FFEE MyAddr=12648430 ... # decimal representation of 0xC0FFEE
```

Assigning `MyAddr` two different values will result in an error:

```bash
cargo run --bin move-build --addresses MyAddr=0xC0FFEE MyAddr=0xDEADBEEF... # ERROR!
```

### Opaqueness

Address assignments, and the name system as whole, only exist at the source
language level and during compilation. Names will be fully substituted for
their value at the byte code level. So the example from before would be
equivalent to

```move
script {
    fun example() {
        0xC0FFEE::M::foo(@0xC0FFEE);
    }
}
```

But at the source language level, the two are not interchangeable. If we had the declaration:

```move
module MyAddr::M {
    public fun bar() {}
}
```

The function `M::bar` _must_ be accessed through the `MyAddr` named address, not through the
numerical value assigned to it.

For example:

```move
script {
    fun example() {
        // ERROR! 0xC0FFEE::M::bar();
        MyAddr::M::bar()
    }
}
```

## Move Standard Library and Diem Framework Modules

As mentioned above, all standard library modules now live in `Std` and all Diem Framework modules
live in `DiemFramework`. The `DiemFramework` address is set to `0x1` and this hard-coded assignment
will be fine. However, our hope is to allow `Std` to be assigned different numerical values
depending on the deployment of those modules. For now, `Std` has a hardcoded assignment of `0x1`.
See the 'Future Work' section below for details about how this might work in the future.

## Backwards Compatibility

Since, all standard library modules and all Diem Framework modules live in `Std` and `DiemFramework`
respectively, source code must be updated to use those named addresses. The named addresses are
opaque, so the numeric values can no longer be used to access the modules. For example, any use of
`0x1::Vector` must now be `Std::Vector`.

Note, as this is just a syntactic change, the compiled module binaries will not be affected.

## Update for release 1.5

The support for assigning values to named addresses in Move source code and declaring named addresses in release 1.4

```move
address MyAddr = 0x19;
```

and

```move
address MyAddr;
```

was removed.

Support for assigning address values was added to the command line and compiler
options were added for use in Move packages and the Move command line as described above.


## Future Work

Named address support will be expanded in a new package system. The intent is that with this system,
a Move program will never assign a value to a named address within the `*.move` files. Instead, all
assignment of a named addresses will exist in a config file, similar to Rust's `Cargo.toml` files.
To enable this package system, additional support will likely be needed from the compiler for
configuring and assigning named addreses.
