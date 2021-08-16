[PROVER]: prover-guide.md

[PROVER_USAGE]: prover-guide.md

[PRE_POST_REFERENCE]: https://en.wikipedia.org/wiki/Design_by_contract

[FRAMEWORK]: ../../../diem-framework/modules/doc/overview.md

# Move Specification Language

**version 1.4**

This document describes the *Move specification language*, a subset of the Move language which
supports specification of the behavior of Move programs. Throughout this document, we abbreviate
this subset of Move
*MSL*.

MSL plays together with the [Move Prover][PROVER], a tool which can statically verify the
correctness of MSL specifications against Move programs. In contrast to traditional testing,
verification of MSL is exhaustive and holds for all possible inputs and global states of a Move
module or transaction script. At the same time, it is fast and automated enough that it can be used
at a similar place in the developer workflow where tests are (for example, for qualification of pull
requests in continuous integration).

While the Move programming language at this point is stable, the subset represented by MSL should be
considered evolving. This has no impact on platform stability, since MSL is not running in
production, but is used for offline quality assurance, where it is continuously improved for
evolving objectives.

This document describes the language only; see [here][PROVER_USAGE] for how to use the Move prover
tool. The reader is expected to have basic knowledge of the Move language, as well as basic
principles of pre/post condition specifications
(see e.g. [this article][PRE_POST_REFERENCE]). For examples of specifications, we refer to
the [Diem framework documentation][FRAMEWORK] which has specifications embedded.

---

- [Expressions](#expressions)
    - [Type System](#type-system)
    - [Naming](#naming)
    - [Operators](#operators)
    - [Function Calls](#function-calls)
    - [Statements](#statements)
    - [Pack and Unpack](#pack-and-unpack)
    - [Quantifiers](#quantifiers)
    - [Choice Operator](#choice-operator)
    - [Builtin Functions](#builtin-functions)
    - [Partial Semantics](#partial-semantics)
- [Specifications](#specifications)
    - [Separating Specifications](#separating-specifications)
    - [Pragmas and Properties](#pragmas-and-properties)
        - [Pragma Inheritance](#pragma-inheritance)
        - [General Pragmas and Properties](#general-pragmas-and-properties)
    - [Pre and Post State](#pre-and-post-state)
    - [Helper Functions](#helper-functions)
        - [Uninterpreted Functions](#uninterpreted-functions)
        - [Axioms](#axioms)
    - [Let Bindings](#let-bindings)
        - [Aborts-If Condition](#aborts-if-condition)
            - [Aborts-If Condition With Code](#aborts-if-condition-with-code)
        - [Aborts-With Condition](#aborts-with-condition)
        - [Requires Condition](#requires-condition)
        - [Ensures Condition](#ensures-condition)
        - [Modifies Condition](#modifies-condition)
        - [Invariant Condition](#invariant-condition)
            - [Function Invariants](#function-invariants)
            - [Struct Invariants](#struct-invariants)
            - [Global Invariants](#global-invariants)
                - [Disabling Invariants](#disabling-invariants)
                - [Update Invariants](#update-invariants)
                - [Isolated Global Invariants](#isolated-global-invariants)
                - [Modular Verification and Global Invariants](#modular-verification-and-global-invariants)
        - [Assume and Assert Conditions in Code](#assume-and-assert-conditions-in-code)
            - [Loop Invariants](#loop-invariants)
            - [Referring to Pre State](#referring-to-pre-state)
        - [Specification Variables](#specification-variables)
        - [Schemas](#schemas)
            - [Basic Schema Usage](#basic-schema-usage)
            - [Schema Expressions](#schema-expressions)
            - [Schema Apply Operation](#schema-apply-operation)
        - [Opaque Specifications](#opaque-specifications)
        - [Abstract Specifications](#abstract-specifications)
        - [Documentation Generation](#documentation-generation)
- [Expressiveness](#expressiveness)

---

# Expressions

Expressions in MSL are a subset of Move program expressions plus a set of additional constructs, as
discussed in the following sections.

## Type System

The type system of MSL is similar to that of Move, with the following differences:

- All integer types of Move (`u8`, `u64`, and `u128`) are treated as the same type. In
  specifications, this type is called `num`, which is an arbitrary precision *signed* integer type.
  When MSL refers to a Move name which represents an `u8` or such, it will be automatically widened
  to `num`. This allows writing MSL expressions like `x + 1 <= MAX_U128` or `x - y >= 0` without
  needing to worry about overflow or underflow.
- The Move types `&T`, `&mut T`, and `T` are considered equivalent for MSL. Equality is interpreted
  as value equality. There is no need to worry about dereferencing a reference from the Move
  program: these are automatically dereferenced as needed. This simplification is possible because
  MSL cannot modify values from a Move program and the program cannot directly reason about
  reference equality (which eliminates the need for doing so in MSL). (Note there is also a
  restriction in expressiveness coming with this, namely
  for [functions which return `&mut T`](#expressiveness), however, this is rarely hit in practice,
  and there are workarounds.)
- There is the additional type `type` which is the type of all types. It can only be used in
  quantifiers.
- There is the additional type `range` which represents an integer range (and the notation `n..m` to
  denote a value).

## Naming

Name resolution works similar to the Move language. `use` declarations can introduce aliases for
imported names. MSL functions and variable names must start with a lower case letter. Schema names
are treated like types and must start with a capital letter (schemas are a new named construct
discussed [later](#schemas)).

Move functions, MSL functions, Move types, and schemas all share the same namespace, and are
therefore unambiguous if aliased via a Move `use` clause. Because of the common name space, an MSL
function cannot have the same name than a Move function. This is often handled via the convention to
prefix MSL function as in
`spec_has_access` when the related Move function is called `has_access`.

## Operators

All Move operators are supported in MSL, except `&`, `&mut`, and `*` (dereference).

In addition to the existing operators, vector subscript `v[i]`, slicing `v[i..j]`, and range
construction
`i..j` are supported (the type of integer ranges is a new builtin type called `range`). Moreover,
boolean implication `p ==> q` is supported as a more intuitive form than `!p || q`.

## Function Calls

In MSL expressions, functions can be called like in Move. However, the callee must either be
a [MSL function](#helper-functions), or a **pure** Move function.

Move functions are considered pure if they do not modify global state and do not use Move expression
features which are not supported in MSL expressions (as defined here).

There is one extension. If a Move function definition contains a direct `assert`, this will be
ignored when it is called from an MSL expression, and the function will be considered pure. For
example:

```move
fun get(addr: address): &T { assert(exists<T>(addr), ERROR_CODE); borrow_global<T>(addr) }
```

This function is pure and can be called from an MSL expression. The assertion will be ignored, and
the function will be interpreted as:

```move
spec fun get(addr: address): T { global<T>(addr) }
```

This is justified by that MSL has [*partial semantics*](#partial-semantics).

## Statements

Limited sequencing of the form `{ let x = foo(); x + x }` is supported, as well as if-then-else.
Other statement forms of the Move language are not supported.

## Pack and Unpack

Pack expressions are supported. Unpack expressions are currently *not* supported.

## Quantifiers

Universal and existential quantification are supported. The general form is

```
forall <binding>, ..., <binding> [ where <exp> ] : <exp>
exists <binding>, ..., <binding> [ where <exp> ] : <exp>
```

- Bindings can either be of the form `name: <type>` or `name in <exp>`. For the second form, the
  expression currently must either be a `range` or a vector.
- The optional constraint `where <exp>` allows to restrict the quantified
  range. `forall x: T where p: q`
  is equivalent to `forall x: T : p ==> q` and `exists x: T where p: q` is equivalent to
  `exists x: T : p && q`.

Notice that it is possible to quantify over types. For example:

```
forall t: type, addr: address where exists<R<t>>(addr): exists<T<t>>(addr)
```

## Choice Operator

The choice operator allows to select a value which satisfies a predicate:

```
choose a: address where exists<R>(a) && global<R>(a).value > 0
```

If the predicate is not satisfiable, the result of the choice will be undetermined (see discussion
of
[partial semantics](#partial-semantics)).

The choice also comes in a form to select the *minimal* value from a set of integers, as in:

```
choose min i: num where in_range(v, i) && v[i] == 2
```

## Builtin Functions

MSL supports a number of builtin constants and functions. Most of them are not available in the Move
language:

- `MAX_U8: num`, `MAX_U64: num`, `MAX_U128: num` returns the maximum value of the corresponding
  type.
- `exists<T>(address): bool` returns true if the resource T exists at address.
- `global<T>(address): T` returns the resource value at address.
- `len<T>(vector<T>): num` returns the length of the vector.
- `update<T>(vector<T>, num, T>): vector<T>` returns a new vector with the element replaced at the
  given index.
- `vec<T>(): vector<T>` returns an empty vector.
- `vec<T>(x): vector<T>` returns a singleton vector.
- `concat<T>(vector<T>, vector<T>): vector<T>` returns the concatenation of the parameters.
- `contains<T>(vector<T>, T): bool` returns true if element is in vector.
- `index_of<T>(vector<T>, T): num` returns the index of the element in the vector, or the length of
  the vector if it does not contain it.
- `range<T>(vector<T>): range` returns the index range of the vector.
- `in_range<T>(vector<T>, num): bool` returns true if the number is in the index range of the
  vector.
- `in_range<T>(range, num): bool` returns true if the number is in the range.
- `update_field(S, F, T): S` updates a field in a struct, preserving the values of other fields,
  where `S` is some struct, `F` the name of a field in `S`, and `T` a value for this field.
- `old(T): T` delivers the value of the passed argument at point of entry into a Move function. This
  is allowed in
  `ensures` post-conditions,
  inline spec blocks (with additional restrictions), and
  certain forms of invariants, as discussed later.
- `TRACE(T): T` is semantically the identity function and causes visualization of the argument's
  value in error messages created by the prover.

Builtin functions live in an unnamed outer scope of a module. If the module defines a function `len`
then this definition will shadow that of the according builtin function. To access the builtin
function in such a situation, one can use the notation `::len(v)`.

## Partial Semantics

In MSL, expressions have a partial semantics. This is in contrast to Move program expressions, which
have a total semantics, since they either deliver a value or abort.

An expression `e[X]` which depends on some some variables `X` may have a known interpretation for
some assignments to variables in `X`, but unknown for others. An unknown interpretation for a
sub-expression causes no issue if its value is not needed for the overall expression result.
Therefore it does not matter if we say `y != 0 && x / y > 0`
or `x / y > 0 && y != 0`: boolean operators are commutative.

This basic principle inherits to higher-level language constructs. For example, in specifications,
it does not matter in which order conditions are
supplied: `aborts_if y != 0; ensures result == x / y;` is the same as
`ensures result == x / y; aborts_if y != 0;`. Also, `aborts_if P; aborts_if Q;` is the same
as `aborts_if Q || P`
.

Moreover, the principle of partial semantics is inherited
to [specification helper functions](#helper-functions)
, which behave transparently. Specifically, inlining those functions is equivalent to calling them (
call-by-expression parameter passing semantics).

# Specifications

Specifications are contained in so-called *specification blocks* (abbreviated **spec block**) that
can appear as module members and inside Move functions. The various types of spec blocks are shown
below, and will be discussed in subsequent sections.

```move
module M {
    resource struct Counter {
        value: u8,
    }

    public fun increment(a: address) acquires Counter {
        let r = borrow_global_mut<Counter>(a);
        spec {
            // spec block targeting this code position
            ...
        };
        r.value = r.value + 1;
    }

    spec increment {
        // spec block targeting function increment
        ...
    }

    spec Counter {
        // spec block targeting struct Counter
        ...
    }

    spec schema Schema {
        // spec block declaring a schema
        ...
    }

    spec fun f(x: num): num {
        // spec block declaring a helper function
        ...
    }

    spec module {
        // spec block targeting the whole module
        ...
    }
}
```

Apart of spec blocks inside Move functions, the textual position of spec block is irrelevant. Also,
a spec block for a struct, function, or module can be repeated multiple times, accumulating the
content.

## Separating Specifications

Instead of putting specifications into the same module as the regular Move definitions, one can also
put them into a separate "specification" module, which can live in the same or a different file:

```move
module M {
    ...
}
spec M {
    spec increment { .. }
}
```

The syntax of a specification module is the same as for a regular module, however, Move functions
and structures are not allowed.

A specification module must be compiled together with the Move module it is targeting, and cannot be
compiled and verified standalone.

In case Move definitions are far apart (e.g. in different files), it is possible to augment the
specification of a Move function with a signature of this function, to give sufficient context to
understand the specification. This syntax is optionally enabled in regular and in specification
modules:

```move
public fun increment(a: address) acquires Counter { .. }
...
spec increment(a: address) { .. }
```

## Pragmas and Properties

Pragmas and properties are a generic mechanism to influence interpretation of specifications. They
are also an extension point to experiment with new concepts before they come part of the mainstream
syntax. Here we give a brief introduction into their general syntax; individual instances are
discussed later.

The general form of a pragma is:

```move
spec .. {
    pragma <name> = <literal>;
}
```

The general form of a property is:

```move
spec .. {
    <directive> [<name> = <literal>] <content>; // ensures, aborts_if, include, etc..
}
```

The `<literal>` can be any value supported by MSL (or the Move language). A value assignment can
also be omitted, in which case a default is used. For example, it is common to use `pragma option;`
as a shortcut for
`pragma option = true;`.

Instead of a single pragma or property, a list can also be provided, as
in `invariant [global, isolated] P`.

### Pragma Inheritance

A pragma in a module spec block sets a value which applies to all other spec blocks in the module. A
pragma in a function or struct spec block can override this value for the function or struct.
Furthermore, the default value of some pragmas can be defined via the prover configuration.

As an example, we look at the `verify` pragma. This pragma is used to turn verification on or off.

```move
spec module {
    pragma verify = false; // By default, do not verify specs in this module ...
}

spec increment {
    pragma verify = true; // ... but do verify this function.
    ...
}
```

### General Pragmas and Properties

A number of pragmas control general behavior of verification. Those are listed in the table below.

| Name                             | Description |
|----------------------------------|--------------
| `verify`     | Turns on or off verification.
| `intrinsic`  | Marks a function to skip the Move implementation and use a prover native implementation. This makes a function behave like a native function even if it not so in Move.
| `timeout` | Sets a timeout (in seconds) for function or module. Overrides the timeout provided by command line flags.
| `verify_duration_estimate`     | Sets an estimate (in seconds) for how long the verification of function takes. If the configured `timeout` is less than this value, verification will be skipped.
| `seed` | Sets a random seed for function or module. Overrides the seed provided by command line flags.

The following properties control general behavior of verification:

| Name       | Description |
|------------|--------------
| `[deactivated]` | Excludes the associated condition from verification.

## Pre and Post State

Multiple conditions in spec blocks work with a *pre* and *post* state, relating them to each other.
Function specifications are one example of this: in the `ensures P` condition, the pre-state (at
function entry) and the post-state (at function exit) are related via the predicate `P`. However,
the concept is more general and also applied for invariants, where the pre-state is before and
post-state after a global update.

In contexts where a pre/post state is active, expressions are evaluated implicitly in the
post-state. To evaluate an expression in a pre-state, one uses the builtin function `old(exp)`,
which evaluates its parameter in the pre-state and returns its value. It is important to understand
that every sub-expression in `exp` is computed in the pre-state as well, including calls to helper
functions.

The 'state' in question here consists of assignments to global resource memory, as well as to any
parameters of the function of type `&mut T`. Examples:

```move
fun increment(counter: &mut u64) { *counter = *counter + 1 }
spec increment {
   ensures counter == old(counter) + 1;
}

fun increment_R(addr: address) {
    let r =  borrow_global_mut<R>(addr);
    r.value = r.value + 1;
}
spec increment_R {
    ensures global<R>(addr).value == old(global<R>(addr).value) + 1;
}
```

## Helper Functions

MSL allows defining helper functions. Those functions can then be used in expressions.

Helper functions are defined using the following syntax:

```move
spec fun exists_balance<Currency>(a: address): bool { exists<Balance<Currency>>(a) }
```

As seen in the example, helper functions can be generic. Moreover, they can access global state.

Definitions of helper functions are neutral regards whether they apply to
a [pre- or post-state](#pre-and-post-state). They are evaluated in the currently active state. For
instance, in order to see whether a balance existed in the pre-state, one
uses `old(exists_balance<Currency>(a))`. Consequently, the expression `old(..)` is not allowed
within the definition of a helper function.

Helper functions are partial functions; see discussion of [partial semantics](#partial-semantics).

### Uninterpreted Functions

A helper function can be defined as **uninterpreted** by simply omitting its body:

```move
spec fun something(x: num): num;
```

An uninterpreted function is one the prover is allowed to assign some arbitrary meaning to, as long
as it is consistent within a given verification context. Uninterpreted functions are a useful tool
for abstraction in specifications (see also [here](#abstract-specifications)).

### Axioms

The meaning of helper functions can be further constraint by using **axioms**. Currently, axioms
must be contained in module spec blocks:

```move
spec module {
    axiom forall x: num: something(x) == x + 1;
}
```

Axiom should be used with care as they can introduce unsoundness in the specification logic via
contradicting assumptions. The Move prover supports a smoke test for detecting unsoundness via
the `--check-inconsistency`
flag.

## Let Bindings

A spec block can contain let bindings which introduce names for expressions:

```move
fun get_R(account: signer): R { ... }
spec get_R {
    let addr = Signer::spec_address_of(account);
    aborts_if addr != ROOT;
    ensures result == global<R>(addr);
}
```

In a spec block which has a pre and post state (like a function specification), the `let name = e`
form will evaluate `e` in the pre-state. In order to evaluate an expression in the post state, one
uses `let post name = e`. In the rhs expression of this form, one can use `old(..)` to refer to the
pre-state.

## Aborts-If Condition

The `aborts_if` condition is a spec block member which can appear only in a function context. It
specifies conditions under which the function aborts.

In the following example, we specify that the function `increment` aborts if the `Counter` resource
does not exist at address `a` (recall that `a` is the name of the parameter of `increment`).

```move
spec increment {
    aborts_if !exists<Counter>(a);
}
```

If a function has more than one `aborts_if` condition, those conditions are or-ed with each other.
The evaluation of the combined aborts condition (or-ed from each individual condition) depends on
the value of the pragma `aborts_if_is_partial`. If this value is false (the default), the function
aborts *if and only if* the combined aborts condition is true. In this case, the above aborts
specification for `increment` will lead to a verification error, since there are additional
situations where `increment` can abort, namely if incrementing `Counter.value` would lead to an
overflow. To fix this, the specification can be completed like this:

```move
spec increment {
    pragma aborts_if_is_partial = false; // This is the default, but added here for illustration.
    aborts_if !exists<Counter>(a);
    aborts_if global<Counter>(a).value == 255;
}
```

If the value of `aborts_if_is_partial` is true, the combined aborts condition (the or-ed individual
conditions)
only *
imply* that the function aborts. Formally, if `A` is the combined aborts condition, then
with `aborts_if_is_partial = true`, we have `A ==> function_aborts`, otherwise we have
`A <==> function_aborts`. Therefore, the following does verify:

```move
spec increment {
    pragma aborts_if_is_partial = true;
    aborts_if !exists<Counter>(a);
}
```

<a name="risk-aborts-if-is-partial"></a>
> Note that there is a certain risk in setting `aborts_if_is_partial` to true, and best practice is to avoid it in specifications of public functions and transaction scripts once those are considered finalized. This is because changing the code after finalization of the spec can add new (non-trivial, undesired) abort situations which the original specification did not anticipate, but which will nevertheless silently pass verification.

If no aborts condition is specified for a function, abort behavior is unspecified. The function may
or may not abort, and verification will not raise any errors, whether `aborts_if_is_partial` is set
or not. In order to state that a function never aborts, use `aborts_if false`. One can use the
pragma `aborts_if_is_strict`
to change this behavior; this is equivalent to as if an `aborts_if false` has been added to each
function which does not have an explicit `aborts_if` clause.

### Aborts-If Condition with Code

The `aborts_if` condition can be augmented with a code:

```
fun get_value(addr: address): u64 {
    aborts(exists<Counter>(addr), 3);
    borrow_global<Counter>(addr).value
}
spec get_value {
    aborts_if !exists<Counter>(addr) with 3;
}
```

It is a verification error if the above function does not abort with code `3` under the given
condition.

In order to specify a direct VM abort, one can use the special constant `EXECUTION_FAILURE`:

```
fun get(addr: address): &Counter acquires Counter {
    borrow_global<Counter>(addr)
}
spec get {
    aborts_if !exists<Counter>(addr) with EXECUTION_FAILURE;
}
```

This same constant can be used for all other VM failures (division by zero, overflow, etc.)

## Aborts-With Condition

The `aborts_with` condition allows specifying with which codes a function can abort, independent
under which condition. It is similar to a 'throws' clause in languages like Java.

```move
fun get_one_off(addr: address): u64 {
    aborts(exists<Counter>(addr), 3);
    borrow_global<Counter>(addr).value - 1
}
spec get_one_off {
    aborts_with 3, EXECUTION_FAILURE;
}
```

If the function aborts with any other or none of the specified codes, a verification error will be
produced.

The `aborts_with` condition can be combined with `aborts_if` conditions. In this case,
the `aborts_with`
specifies any other codes with which the function may abort, in addition to the ones given in
the `aborts_if`:

```move
spec get_one_off {
    aborts_if !exists<Counter>(addr) with 3;
    aborts_with EXECUTION_FAILURE;
}
```

If this is not wanted, and the `aborts_with` should be independent of `aborts_if`, one can use the
property
`[check]`:

```move
spec get_one_off {
    aborts_if !exists<Counter>(addr) with 3;
    aborts_if global<Counter>(addr) == 0 with EXECUTION_FAILURE;

    aborts_with [check] 3, EXECUTION_FAILURE;
}
```

> TODO: `[check]` property is currently not longer implemented

## Requires Condition

The `requires` condition is a spec block member which postulates a pre-condition for a function. The
prover will produce verification errors for functions which are called with violating
pre-conditions.

A `requires` is different from an `aborts_if`: in the latter case, the function can be called, and
any aborts it produces will be propagated to the caller context. In the `requires` case, the prover
will not allow the function to be called in the first place. Nevertheless, the function can *still
be called at runtime* if verification is skipped. Because of this, `requires` are rare in Move
specifications, and
`aborts_if` are more common. Specifically, `requires` should be avoided for public APIs.

An example of `requires` is the following:

```move
spec increment {
    requires global<Counter>(a).value < 255;
}
```

## Ensures Condition

The `ensures` condition postulates a post-condition for a function which must be satisfied when the
function terminates successfully (i.e. does not abort). The prover will verify each `ensures` to
this end.

An example for the `ensures` condition is the following:

```move
spec increment {
    ensures global<Counter>(a) == old(global<Counter>(a)) + 1;
}
```

Within the expression for the `ensures` condition, one can use the `old` function,
[as discussed here](#pre-and-post-state).

## Modifies Condition

The `modifies` condition is used to provide permissions to a function to modify global storage. The
annotation itself comprises a list of global access expressions. It is specifically used together
with [opaque function specifications](#opaque-specifications).

```move
resource struct S {
    x: u64
}

fun mutate_at(addr: address) acquires S {
    let s = borrow_global_mut<S>(addr);
    s.x = 2;
}
spec mutate_at {
    pragma opaque;
    modifies global<S>(addr);
}
```

In general, a global access expression has the form `global<type_expr>(address_expr)`. The
address-valued expression is evaluated in the pre-state of the annotated function.

```move
fun read_at(addr: address): u64 acquires S {
    let s = borrow_global<S>(addr);
    s.x
}

fun mutate_S_test(addr1: address, addr2: address): bool acquires T {
    assert(addr1 != addr2, 43);
    let x = read_at(addr2);
    mutate_at(addr1); // Note we are mutating a different address than the one read before and after
    x == read_at(addr2)
}
spec mutate_S_test {
    aborts_if addr1 == addr2;
    ensures result == true;
}
```

In the function `mutate_S_test`, the assertion in the spec block is expected to hold. A benefit of
the modifies specification on `mutate_at` is that this assertion can be proved whether or
not `mutate_at` is inlined.

If the modifies annotation is omitted on a function, then that function is deemed to have all
possible permissions for those resources it may modify during its execution. The set of all
resources that may be modified by a function is obtained via an interprocedural analysis of the
code. In the example above, `mutate_S_test` does not have a modifies specification and modifies
resource `S` via the call to `mutate_at`. Therefore, it is considered to have modified `S` at any
possible address. Instead, if the programmer adds `modifies global<S>(addr1)`
to the specification of `mutate_S_test`, then the call to `mutate_at` is checked to make sure that
modify permissions granted to `mutate_S_test` cover the permissions it grants to `mutate_at`.

## Invariant Condition

The invariant condition can be applied on structs and on global level.

### Function Invariants

The `invariant` condition on a function is simply a shortcut for a `requires` and `ensures` with the
same predicate.

Thus the following spec block:

```move
spec increment {
    invariant global<Counter>(a).value < 128;
}
```

... is equivalent to:

```move
spec increment {
    requires global<Counter>(a).value < 128;
    ensures global<Counter>(a).value < 128;
}
```

### Struct Invariants

When the `invariant` condition is applied to a struct, it expresses a well-formedness property of
the struct data. Any instance of this struct which is currently not mutated will satisfy this
property (with exceptions as outlined below).

For example, we can postulate an invariant on our counter that it never must exceed the value of
127:

```move
spec Counter {
    invariant value < 128;
}
```

A struct invariant is checked by the prover whenever the struct value is constructed (packed). While
the struct is mutated (e.g. via a `&mut Counter`) the invariant does *not* hold (but see exception
below). In general, we consider mutation as an implicit unpack, and end of mutation as a pack.

The Move language semantics unambiguously identifies the point when mutation ends and starts. This
follows from the borrow semantics of Move, and includes mutation via an enclosing struct.
(The mutation of an inner struct ends when the mutation of the root struct where mutation started
ends.)

There is one exception to this rule. When a mutable reference to a struct declared in module M is
passed into a *
public*
function of M which does by itself *not* return any other mutable reference (which could be borrowed
from the input parameter), we treat this parameter as "packed". That means, on function entry, we
will unpack it and on function exit we will pack again, enforcing the invariant. This reflects that
in Move, struct data can only be mutated within the module which declares the struct, so for an
outside caller of the public function, the mutable reference can actually not be mutated unless by
calling public functions of module M again. It is a significant simplification of the verification
problem to exploit this in the semantics.

### Global Invariants

A global invariant appears as a member of module. It can express a condition over the global state
of the Move program, as represented by resources stored in memory. For example, the below invariant
states that a `Counter`
resource stored at any given address can never be zero:

```move
module M {
    invariant forall a: addr where exists<Counter>(a): global<Counter>(a).value > 0;
}
```

A global invariant is assumed to hold when data is read from the global state, and is asserted (and
may fail to verify)
at the moment the state is updated. For example, the below function will never abort with arithmetic
underflow because the counter value is always greater zero; however, it will create a verification
error since the counter can drop to zero:

```
fun decrement_ad(addr: address) acquires Counter {
    let counter = borrow_global_mut<Counter>(addr);
    let new_value = counter.value - 1;   // Will not abort because counter.value > 0
    *counter.value = new_value;          // Fails verification since value can drop to zero
}
```

#### Disabling Invariants

There are times when a global invariant holds almost everywhere,
except for a brief interval inside a function. In current move code,
this often occurs when something (e.g. an account) is being setup and
several structs are published together. Almost everywhere, an
invariant holds that all of the structs are published or none of them
are. But the code that publishes the structs must do so sequentially.
While they are being published, there will be a point where some are
published and others are not.

In order to verify invariants that hold except during small regions, there
is a feature to allow users to disable invariants temporarily.
Consider the following code fragment:

```move
fn setup() {
    publish1();
    publish2();
    }
}
```
where `publish1` and `publish2` publish two different structs, `T1` and `T2` at address `a`.

```move
module M {
    invariant [global] exists<T1>(a) == exists<T2>(a)
}
```

As written, the prover will report that the invariant is violated
after the call to `publish1` and before the call to `publish2`.
If either of `publish1` or `publish2` is without the other, the prover
will also report a violation of the invariant.

To prove that the invariant holds everywhere else, there is a pragma
```move
spec setup {
    pragma disable_invariants_in_body;
}
```
which says that invariants not required to hold while `setup` is executing,
but are assumed to hold on entry to and exit from `setup`.

This pragma changes the Prover's behavior. The invariants are assumed on
entry to `setup`, but not proved during or after `publish1` and
`publish2`.  Instead, all invariants that could be invalidated in the
body of `setup` are asserted and proved at the point of return from
`setup`.  A consequence of this processing is that the user may need
to provide stronger post-conditions on `publish1` and `publish2` to
make it possible to prove the invariants on exit from `setup`.

Another consequence of this processing is that invariants cannot
safely be assumed to hold during the execution of `publish1` and
`publish2` (unless nothing in the body of `setup` changes state
mentioned in the invariant). Therefore, if proving a post-condition
requires the invariant to be assumed, the post-condition will fail.

In the example, invariants hold at the call sites of `setup`, but not
in the body. For `publish1`, invariants don't necessarily hold at the
call site *or* in the body of the function.  In the example, that
behavior is implied because `publish1` is called in a context where
invariants are disabled.

When invariants are disabled in `setup` in the above example, the
prover cannot assume them on entry to `publish1` and `publish2`, and
should not try to prove them on exit from those functions. The Prover
would have the same behavior for any functions called by `publish1` or
`publish2`.  The Prover *automatically* adopts this behavior when
invariants are disabled in a calling function, but it is possible for
the user to declare that a function be treated like `publish1`.

For example, if `publish2` is *only* called from the setup function above,
and we did *not* disable invariants in `setup`, we could achieve a similar
effect by using this pragma, instead.

```move
spec setup {
    pragma delegate_invariants_to_caller;
}
```
This would only be legal if `setup` is a private or `public (friend)` function.
The difference between this and disabling invariants in `setup` is that the
invariants would not be assumed at the beginning of `setup` and would be proved
after `setup` returns at each site where it is called.

While both pragmas disable invariants in the body of a function, the
difference is that `disable_invariants_in_body` assumes invariants on
entry and proves them on exist, while `delegate_invariants_to_caller`
does neither.

There are some limitations on how these pragmas can be used.
`disable_invariants_in_body` cannot be declared for functions where
invariants are delegated to a caller, either explicitly via the pragma
or implicitly because the function is called in a context where
invariants have been disabled. (This restriction is to ensure
consistent processing, because on pragma assumes that invariants hold
in the calling context and the other does not).  Second, it is illegal
for a public or script function to delegate invariant checking to its
callers (since the Prover does not know all the call sites), *unless*
the function cannot possibly invalidate an invariant because it
doesn't change any of the state mentioned in `exists` and `global`
expressions appearing in the invariant.


#### Update Invariants

The `update` form of a global invariant allows to express a relation
between [pre-state and post-state](#pre-and-post-state) of a global state update. For example, the
following invariant states that the counter must decrease monotonically whenever it is updated:

```move
module M {
    invariant update [global] forall a: addr where old(exists<Counter>(a)) && exists<Counter>(addr):
        global<Counter>(a).value <= old(global<Counter>(a));
}
```

#### Isolated Global Invariants

A global invariant can be marked as `[isolated]` to indicate that it is not relevant for proving
other properties of the program. An isolated global invariant will not be assumed when the related
global state is read. It will only be assumed before the state is updated, to help prove that the
invariant still holds after the update. This feature is for improving performance in situations
where there are many global invariants but there have no direct influence on verification.

#### Modular Verification and Global Invariants

Certain usage of global invariants leads to verification problems which cannot be checked in a
modular fashion.
"Modular" here means that a module can be verified standalone and proven to be universally correct
in all usage contexts (if preconditions are met).

A non-modular verification problem may arise if a global invariant refers to state from multiple
modules. Consider a situation where module `M1` uses module `M2`, and `M1` contains the following
invariant, with the helper function `condition` referring to global state of each respective module:

```move
module M1 {
    invariant M1::condition() ==> M2::condition();
}
```

When we verify `M1` standalone, the Move prover will determine that it also needs to verify
functions in `M2`, namely those which update the M2 memory such that the invariant in M1 can fail.

> TODO: document mechanism and pragmas to defer invariant verification for code sections

## Assume and Assert Conditions in Code

A spec block might also occur anywhere an ordinary Move statement block can occur.
Here is an example:

```
fun simple1(x: u64, y: u64) {
    let z;
    y = x;
    z = x + y;
    spec {
        assert x == y;
        assert z == 2*x;
    }
}
```

In such inline spec blocks, only a subset of conditions are permitted:
- `assume` and `assert` statements are allowed in any code locations
- loop `invariant` statements are allowed only in code locations that represent loop headers.

An assert statement inside a spec block indicates a condition that must hold when control reaches
that block. If the condition does not hold, an error is reported by the Move Prover. An assume
statement, on the other hand, blocks executions violating the condition in the statement. The
function `simple2` shown below is verified by the Move Prover. However, if the first spec block
containing the assume statement is removed, Move Prover will show a violating to the assert
statement in the second spec block.

```
fun simple2(x: u64, y: u64) {
    let z: u64;
    spec {
        assume x > y;
    };
    z = x + y;
    spec {
        assert z > 2*y;
    }
}
```

### Loop Invariants

An `invariant` statement encodes a loop invariant and must beplaced at a loop head, as in the
following example.

```
fun simple3(n: u64) {
    let x = 0
    loop {
        spec {
            invariant x <= n;
        };
        if (x < n) {
            x = x + 1
        } else {
            break
        }
    };
    spec {
        assert x == n;
    }
}
```

A loop invariant is translated into two `assert` statements and one `assume` statement to
facilitate  the inductive reasoning of properties about the loop.
In break down, a loop invariant is translated to:
- An `assert` statement that checks the invariant holds when the loop is first encountered in the
  execution -- establishing the base case.
- An `assume` statement that encodes the property that the invariant holds at loop iteration `I`.
- An `assert` statement that checks whether the invariant continues to hold at loop iteration `I+1`.

### Referring to Pre State

Occasionally, we would like to refer to the pre state of a mutable function argument in inline spec
blocks. In MSL, this can be done with the `old(T)` expression. Similar to the semantics of `old(..)`
in post conditions, an `old(T)` expression in an `assume` or `assert` statement always yields the
value of `T` at the function entry point. Following is an example that illustrate the use of
`old(..)` in an inline spec block.

```
fun swap(x: &mut u64, y: &mut u64) {
    let t = *x;
    *x = *y;
    *y = t;
    spec {
        assert x == old(y);
        assert y == old(x);
    };
}
```

The above example is trivial as the same property can be expressed with post conditions
(i.e., `ensures`) too. But there are cases where we must use `old(..)` to refer to the pre
state, especially in the specification of loop invariants. Consider the following example
where we verify that the `vector_reverse` function properly reverse the order of all elements
in a vector:

```
fun verify_reverse<Element>(v: &mut vector<Element>) {
    let vlen = Vector::length(v);
    if (vlen == 0) return ();

    let front_index = 0;
    let back_index = vlen -1;
    while ({
        spec {
            assert front_index + back_index == vlen - 1;
            assert forall i in 0..front_index: v[i] == old(v)[vlen-1-i];
            assert forall i in 0..front_index: v[vlen-1-i] == old(v)[i];
            assert forall j in front_index..back_index+1: v[j] == old(v)[j];
            assert len(v) == vlen;
        };
        (front_index < back_index)
    }) {
        Vector::swap(v, front_index, back_index);
        front_index = front_index + 1;
        back_index = back_index - 1;
    };
}
spec verify_reverse {
    aborts_if false;
    ensures forall i in 0..len(v): v[i] == old(v)[len(v)-1-i];
}
```

Note the usage of `old(v)` in the loop invariants. Without them, it is hard to express the
invariant that the vector is partially reversed while the loop is iterating and the rest
remain unchanged.

However, unlike the `old(T)` expressions in `ensures` conditions where `T` can be any valid
expression (e.g., `old(v[i])` is allowed), the `old(T)` expressions in `assert` and `assumes`
statements only accept a single variable as `T` and that variable must be a function argument of
a mutable reference type. In the above example, `old(v[i])` is not allowed, and we should use
`old(v)[i]` instead.

## Specification Variables

MSL supports *spec variables*, also called *ghost variables* in the verification community. These
variables are used only in specifications and represent information derived from the global state of
resources. An example use case is to compute the sum of all coins available in the system and
specify that the sum can be changed only in certain scenarios.

We illustrate this feature by introducing a spec variable which maintains the sum of all `Counter`
resources from our running example. First, a spec variable is introduced via spec module block as
follows:

```move
spec module {
    global sum_of_counters: num;
}
```

This value is going to be updated whenever a `Counter` is packed or unpacked. (Recall that mutation
is interpreted as an implicit unpack and pack):

```move
spec Counter {
    invariant pack sum_of_counters = sum_of_counters + value;
    invariant unpack sum_of_counters = sum_of_counters - value;
}
```

Now we may for example want to specify that the sum of all Counter instances in the global state
should never exceed a particular value. We can do this as follows:

```move
spec module {
    invariant [global] sum_of_counters < 4711;
}
```

Note that spec variables can be referenced also from helper functions. Moreover, spec variables can
be generic:

```move
spec module {
    global some_generic_var<T>: num;
}
```

When using such a spec variable, a type parameter need to be provided, as in `some_generic_var<u64>`
. Effectively, a generic spec variable is like a family of variables indexed by types.

## Schemas

Schemas are a means for structuring specifications by grouping properties together. Semantically,
they are just syntactic sugar which expand to conditions on functions, structs, or modules.

### Basic Schema Usage

Schemas are used as such:

```move
spec schema IncrementAborts {
    a: address;
    aborts_if !exists<Counter>(a);
    aborts_if global<Counter>(a).value == 255;
}

spec increment {
    include IncrementAborts;
}
```

Each schema may declare a number of typed variable names and a list of conditions over those
variables. All supported condition types can be used in schemas. The schema can then be included in
another spec block:

- If that spec block is for a function or a struct, all variable names the schema declares must be
  matched against existing names of compatible type in the context.
- If a schema is included in another schema, existing names are matched and must have the same type,
  but non-existing names will be added as new declarations to the inclusion context.

When a schema is included in another spec block, it will be checked whether the conditions it
contains are allowed in this block. For example, including the schema `IncrementAborts` into a
struct spec block will lead to a compile time error.

When a schema is included the names it declares can also bound by expressions. For example, one can
write `include IncrementAborts{a: some_helper_address()}`. Effectively, not providing a binding is
equivalent to writing `IncrementAborts{a: a}` if `a` is an existing name in scope.

Schemas can be generic. Generic schemas must be fully instantiated where they are included; type
inference is not available for schemas.

### Schema Expressions

When a schema is included, one can use a limited set of Boolean operators as follows:

- `P ==> SchemaExp`: all conditions in the schema will be prefixed with `P ==> ..`. Conditions which
  are not based on boolean expressions will be rejected.
- `if (P) SchemaExp1 else SchemaExp2`: this is treated similar as including both
  `P ==> SchemaExp1` and `!P ==> SchemaExp2`.
- `SchemaExp1 && SchemaExp2`: this is treated as two includes for the both schema expressions.

### Schema Apply Operation

One of the main uses cases for schemas is to be able to name a group of properties and then apply
those to a set of functions. This is achieved by the `apply` operator. The `apply` spec block member
can only appear in module spec blocks.

The general form of the apply operator
is `apply Schema to FunctionPattern, .. except FunctionPattern, ..`. Here,
`Schema` can be a schema name or a schema name plus formal type arguments.
`FunctionPatterns` consists of an optional visibility modifier `public` or `internal` (if not
provided, both visibilities will match), a name pattern in the style of a shell file pattern (
e.g. `*`, `foo*`, `foo*bar`, etc.)
, and finally an optional type argument list. All type arguments provided to `Schema` must be bound
in this list and vice versa.

The apply operator includes the given schema in all function spec blocks which match the patterns,
except those which are excluded via the `except` patterns.

A typical use of the apply operator is to provide common pre and post conditions to all functions in
a module with some exceptions. Example:

```move
spec schema Unchanged {
    let resource = global<R>(ADDR):
    ensures resource == old(resource);
}

spec module {
    // Enforce Unchanged for all functions except the initialize function.
    apply Unchanged to * except initialize;
}
```

Notice that while with [global invariants](#global-invariants) we can express similar things, we *
cannot*
express the restriction of the invariant to only specific functions.

## Opaque Specifications

With the pragma `opaque`, a function is declared to be solely defined by its specification at caller
sides. In contrast, if this pragma is not provided, then the function's implementation will be used
as the basis to verify the caller.

Using `opaque` requires the specification to be sufficiently complete for the verification problem
at hand. Without opaque, the Move prover will use the implementation as the source of truth of the
definition of the function. But with opaque, if there is an aspect of the function definition
unspecified, an arbitrary meaning will be assumed. For example, with the specification below,
the `increment` function can abort under arbitrary conditions:

```move
spec increment {
    pragma opaque;
    // aborts_if !exists<Counter>(a);  // We need to add this to make the function not abort arbitrarily
    ensures global<Counter>(a) == old(global<Counter>(a)) + 1;
}
```

In general, opaque functions enable modular verification, as they abstract from the implementation
of functions, resulting in much faster verification.

If an opaque function modifies state, it is advised to use
the [`modifies` condition](#modifies-condition) in its specification. If this is omitted,
verification of the state changes will fail.

## Abstract Specifications

The `[abstract]` property allow to specify a function such that an abstract semantics is used at the
caller side which is different from the actual implementation. This is useful if the implementation
is too complex for verification, and an abstract semantics is sufficient for verification goals.
The `[concrete]` property, in turn, allows to still specify conditions which are verified against
the implementation, but not used at the caller side.

Consider the following example of a hash function. The actual value of the hash is not relevant for
verification of callers, and we use an [uninterpreted helper function](#uninterpreted-functions)
delivering an arbitrary value chosen by the prover. We can still specify the concrete implementation
and verify its correctness:

```move
fun hash(v: vector<u8>): u64 {
    <<sum up values>>(v)
}
spec hash {
    pragma opaque;
    aborts_if false;
    ensures [concrete] result == <<sum up values>>(v);
    ensures [abstract] result == spec_hash_abstract(v);
}
spec fun abstract_hash(v: vector<u8>): u64; // uninterpreted function
```

The soundness of the abstraction is the responsibility of the specifier, and not verified by the
prover.

> NOTE: the abstract/concrete properties should only be used with opaque specifications, but the prover will
> currently not generate an error if not.

> NOTE: the `modifies` clause does currently not support abstract/concrete. Also, if no modifies is given, the
> modified state will be computed from the implementation anyway, possibly conflicting with `[abstract]` properties.

## Documentation Generation

The organization of specification blocks in a file is relevant for documentation generation -- even
though it is not for the semantics. See discussion
of [organization of specification blocks](docgen.md#organization-of-specification-blocks).

# Expressiveness

The Move Specification language is expressive enough to represent the full Move language semantics (
formal argument outstanding) with one exception: functions which return a `&mut T` type.

Consider the following code:

```move
struct S { x: u64, y: u64 }

fun x_or_y(b: bool, s: &mut S): &mut u64 {
    if (b) &mut s.x else &mut s.y
}
spec x_or_y {
    ensures b ==> result == s.x;
    ensures !b ==> result == s.y;
}
```

We are not able to specify the *full* semantics of `x_or_y` in MSL, because we cannot capture the
semantics of mutable references. While we can say something about the value behind the reference at
function exit, subsequent effects as in `*x_or_y(b, &mut s) = 2` cannot be specified.

However, the Move prover *does* understand the meaning of such functions -- the restriction is only
in what we can specify. Practically this means we cannot make the function `x_or_y` opaque, and must
let verification rely on that the prover directly works with the implementation. Specifically, we
can verify the following
(which can then be opaque):

```move
fun x_or_y_test(s: S): S {
    *x_or_y(true, &mut s) = 2;
    s
}
spec x_or_y_test {
    pragma opaque;
    ensures result.x == 2;
    ensures result.y == s.y;
}/in
```
