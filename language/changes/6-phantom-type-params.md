# Phantom Type Parameters

- Status: Implemented in Move 1.4

## Introduction

A _phantom type parameter_ is one that doesn't show up at runtime, but is checked statically at
compile time. Phantom type parameters can be useful in a couple of situations. For example, the
struct `Event::EventHandle<T>` in the Move Standard library doesn't contain any field of type `T`,
but uses the type parameter to ensure at the type level that the handle can only be used for
messages of type `T`. Another prominent example is `Diem::Diem<CoinType>`, which is generic on a
`CoinType` specifying the currency of the coin and allowing code to be written generically on any
currency. As a last example, the capabilities in `Vault` also rely heavily on phantom type
parameters.

Previously, Move's type system didn't make a difference on whether a type parameter was phantom, but
with this new feature, a struct's type parameter can be explicitly declared as phantom. A parameter
declared as phantom is not considered when computing the
[conditional abilities with generic types](3-abilities.md#Conditional-Abilities-with-Generic-Types).
For this relaxed rule to be sound, Move's type system guarantees that a parameter declared as
phantom is either not used at all in the struct definition, or it is only used as an argument to
type parameters also declared as phantom.

## Motivations

Previously, defining a struct `S` with a phantom type parameter required spurious ability
annotations to satisfy the requirements of the abilities declared for `S`. This resulted in
increased chance of bugs and security vulnerability because types had to be weakened with
unnecessary ability declarations. Moreover, the spurious annotations were infectious, requiring many
functions generic on the phantom type parameter to also include the necessary constraints. With the
new feature, arguments to phantom type parameters are not considered when deriving the abilities for
generic types, thus avoiding the need for spurious ability annotations.

For example, `Diem::Diem<CoinType>` is declared with the ability `key`. This required all arguments
to `CoinType` to be declared with `store`. Specifically, the types `XUS::XUS` and `XDX::XDX` were
forced to have a `store` annotation even though they were never stored in global storage! Moreover
the extra `store` requirement polluted the entire Diem Framework and many functions had to specify
extra `store` constraints on their type parameters. With the new feature, `Diem::Diem<CoinType>` now
declares `CoinType` to be phantom which avoids the spurious `store` annotations sprinkled over the
code.

## Description

### Declaration

In a struct definition a type parameter can be declared as phantom by adding the `phantom` keyword
before its declaration. If a type parameter is declared as phantom we say it is a phantom type
parameter. When defining a struct, Move's type checker ensures that every phantom type parameter is
either not used inside the struct definition or it is only used as an argument to a phantom type
parameter.

More formally, if a type is used as an argument to a phantom type parameter we say the type appears
in _phantom position_. With this definition in place, the rule for the correct use of phantom
parameters can be specified as follows: **A phantom type parameter can only appear in phantom
position**.

The following two examples show valid uses of phantom parameters. In the first one, the parameter
`T1` is not used at all inside the struct definition. In the second one, the parameter `T1` is only
used as an argument to a phantom type parameter.

```
struct S1<phantom T1, T2> { f: u64 }
                  ^^
                  Ok: T1 does not appear inside the struct definition


struct S2<phantom T1, T2> { f: S1<T1, T2> }
                                  ^^
                                  Ok: T1 appears in phantom position
```

The following code shows examples of violations of the rule:

```

struct S1<phantom T> { f: T }
                          ^
                          Error: Not a phantom position

struct S2<T> { f: T }

struct S3<phantom T> { f: S2<T> }
                             ^
                             Error: Not a phantom position
```

### Instantiation

When instantiating a struct, the arguments to phantom parameters are excluded when deriving the
struct abilities. For example, consider the following code:

```
struct S<T1, phantom T2> has copy { f: T1 }
struct NoCopy {}
struct HasCopy has copy {}
```

Consider now the type `S<HasCopy, NoCopy>`. Since `S` is defined with `copy` and all non-phantom
arguments have copy then `S<HasCopy, NoCopy>` also has copy.

### Phantom Type Parameters with Ability Constraints

Ability constraints and phantom type parameters are orthogonal features in the sense that phantom
parameters can be declared with ability constraints. When instantiating a phantom type parameter
with an ability constraint, the type argument has to satisfy that constraint, even though the
parameter is phantom. For example, the following definition is perfectly valid:

```
struct S<phantom T: copy> {}
```

The usual restrictions apply and `T` can only be instantiated with arguments having `copy`.
