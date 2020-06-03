# The Move Specification Language (DRAFT)

The Move Specification language (abbreviated as *Spec language*) is a language for specifying properties of Move smart
contracts. It plays together with the [Move Prover](./prover-guide.md), a tool which can formally verify the
correctness of such properties. In contrast to traditional testing, verification of properties in the Spec language
is exhaustive and holds for all possible inputs and blockchain states of a Move contract. This document gives
an overview of the design and usage of the Spec language. Knowledge of the Move language itself is expected
from a reader.

- [Type System](#type-system)
- [Naming](#naming)
- [Expressions](#expressions)
- [Builtin Functions](#builtin-functions)
- [Spec Blocks](#spec-blocks)
- [Pragmas](#pragmas)
- [Helper Functions](#helper-functions)
- [AbortsIf Condition](#abortsif-condition)
- [TerminatesIf Condition](#terminatesif-condition)
- [Requires Condition](#requires-condition)
- [Requires Module Condition](#requires-module-condition)
- [Ensures Condition](#ensures-condition)
- [Assume and Assert Conditions](#assume-and-assert-conditions)
- [Invariant Condition on Functions](#invariant-condition-on-functions)
- [Invariant Condition on Modules](#invariant-condition-on-modules)
- [Invariant Condition on Structs](#invariant-condition-on-structs)
- [Specification Variables and Pack/Unpack Invariants](#specification-variables-and-packunpack-invariants)
- [Schemas](#schemas)
- [Schema Expressions](#schema-expressions)
- [Schema Apply Operation](#schema-apply-operation)


## Type System

The type system of the Spec language is mostly identical to that of Move. However, there are a few differences:

- All integer types of Move (`u8`, `u64`, and `u128`) are treated as the same type. In specifications, this
  type is called `num`, which is an arbitrary precision signed integer type. When the Spec language refers to a name
  which represents an `u8` or such, it will be automatically widened to `num`. This allows to write Spec expressions
  like `x + 1 <= max_u128()` or `x - y >= 0` without needing to worry about overflow or underflow.
- The Move types `&T`, `&mut T`, and `T` are considered to be equivalent for the Spec language. There is no need
  to worry about dereferencing when working with a reference from the Move program; it will be automatically
  dereferenced as needed. This is enabled by that the Spec language cannot modify values from a Move program,
  and reference equality is not something the Move language can reason about (and therefore no need for doing so
  in the Spec language).
- There are a few additional types compared to the Move type system. Those will be discussed as we look at expression
  constructs which support them.

## Naming

Name resolution works similar as with the Move language. `use` declarations can introduce aliases for imported names.
Spec function and variable names must start with a lower case letter. Schema names are treated like types and must start
with a capital letter (schemas are a new named construct discussed [later](#schemas)).

Move functions, Spec functions, Move types, and schemas all share the same namespace, and are therefore unambiguous
if aliased via a Move `use` clause.

> Note: this alignment is currently in development. Document final resolution rules or point to them
> once it's done.

## Expressions

The available expressions in Spec language are a subset of the Move language, plus a set of additional constructs.

- All Move operators are supported, except `&`, `&mut`, and `*` (dereference).
- In addition to the existing operators, vector subscription `v[i]`, slicing `v[i..j]`, and range construction
  `i..j` is supported (the type of integer ranges is a new builtin type called `range`). Moreover, boolean
  implication `p ==> q` is supported as a more intuitive form than `!p || q`.
- Function calls are supported (but the target is a Spec function not a Move function -- Move functions cannot be
  called from specs).
- Limited sequences of the form `{ let x = foo(); x + x }` are supported.
- Pack expressions are supported. Unpack expressions are currently *not* supported.
- If-then-else is supported.
- A [spec variable](#specification-variables-and-packunpack-invariants) can be generic, and therefore the
  notation `name<T>` (or `Module::name<T>`) is supported. In Move, type arguments can only be provided to calls.
- A form of lambda, denoted as `|x| x + 1`, is supported. The lambda can only appear as a parameter of the `all`
  or `any` function (see next section).

## Builtin Functions

The Spec language supports a number of builtin functions. Most of them are not available in the Move language:

- `exists<T>(address): bool` returns true of the resource exists at address.
- `global<T>(address): T` returns the resource at address.
- `sender(): address` returns the address of the sender.
- `max_u8(): num`, `max_u64(): num`, `max_u128(): num` returns the maximum value of the according type.
- `len(vector<T>): num` returns the length of the vector.
- `update(vector<T>, num, T>): vector<T>` returns a new vector with the element replaced at the given index.
- `type<T>()` returns an opaque value of Spec language type `type` which represents
   the type T. Type values can be only compared for equality.
- `domain<T>()` returns the set of all values of type T. This expression can only be used as a parameter for
   the `all` or `any` function.
- `all(vector<T>, |T|bool`, `all(range, |num|bool)`, `all(domain<T>, |T|bool)` is universal quantification over
   the values in a vector, the numbers in a range, or the whole type T. The 2nd parameter must be a lambda expression.
- `any(vector<T>, |T|bool`, `any(range, |num|bool)`, `any(domain<T>, |T|bool)` is existential quantification.
- `old(T): T` delivers the value of the passed argument at point of entry into a Move function. This is only allowed
  in `ensures` post-conditions and certain forms of invariants, as discussed later.
- `TRACE(T): T` semantically identity, this causes visualization of the argument's value in error messages the
  prover creates.

## Spec Blocks

Specification entities are contained in so-called *spec blocks*. Spec blocks can appear as module members and
in code blocks. A spec block on module level declares the *target* of specification in a header:

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

    spec fun increment {
        // spec block targeting function increment
        ...
    }

    spec struct Counter {
        // spec block targeting struct Counter
        ...
    }

    spec schema Schema {
        // spec block declaring a schema
        ...
    }

    spec module {
        // spec block targeting the whole module
        ...
    }
}
```

The position of a spec block on module level is not relevant. A spec block for the same struct, function, or module can
be repeated multiple times, accumulating the content.

Each spec block contains a number of members, separated by trailing `;`. Not all member types are allowed in all
contexts, as will be discussed when they are introduced.

## Pragmas

Pragmas are special spec block members which allow to influence verification behavior by specifying a configuration
option to the prover. The general form of a pragma is:

```move
spec <target> {
    pragma <name> = <literal>;
}
```

The `literal` can be any value as supported by the Spec language (which are the same as the Move language).

There are multiple pragmas which the prover understands. They will be introduced in the context
where they apply. A general mechanism with pragmas is *inheritance*.
A pragma in a module spec block sets a value which applies to all other spec blocks in the module. A pragma
in a function or struct spec block can override this value for the function or struct. Furthermore, the
default value of some pragmas can be defined via the prover configuration.

As an example, we look at the `verify` pragma. This pragma allows to turn on or off verification.

```move
spec module {
    pragma verify = false; // By default, do not verify specs in this module ...
}

spec fun increment {
    pragma verify = true; // ... but do verify this function.
    ...
}
```

## Helper Functions

The Spec language allows to define helper functions. Those functions can than be used in expressions.

Helper functions are defined as part of `spec module` block using the `define` keyword, e.g.:

```move
spec module {
    define account_exists<Token>(a: address): bool { exists<Account<Token>>(a) }
}
```

Is seen in the example, they can be generic. Helper functions can also access global state as done in the
example.

## AbortsIf Condition

The `aborts_if` condition is a spec block member which can appear only in a function context. It specifies conditions
under which the function aborts.

> Currently there is no way to specify with which code the function will abort; this feature may be added later

In the following example, we specify that the function `increment` aborts if the `Counter` resource does not exist
at address `a` (recall that `a` is the name of the parameter of `increment`).

```move
spec fun increment {
    aborts_if !exists<Counter>(a);
}
```

If a function has more than one `aborts_if` condition, those conditions are or-ed with each other.
The evaluation of the combined aborts condition (or-ed from each individual condition) depends on the value of the
pragma `aborts_if_is_partial`. If this value is false (the default), the function aborts *if and only if* the
combined aborts condition is true. In this case, the above aborts specification for `increment` will
lead to a verification error, since there are additional situations where `increment` can abort, namely if
incrementing `Counter.value` would lead to an overflow. To fix this, the specification can be
completed like this:

```move
spec fun increment {
    pragma aborts_if_is_partial = false; // This is the default, but added here for illustration.
    aborts_if !exists<Counter>(a);
    aborts_if global<Counter>(a).value == 255;
}
```

If the value of `aborts_if_is_partial` is true, the combined aborts condition (the or-ed individual
conditions) only *imply* that the function aborts. Formally, if `A` is the combined aborts condition,
then with `aborts_if_is_partial = true`, we have `A ==> function_aborts`, otherwise we have
`A <==> function_aborts`. Therefore, the following does verify:

```move
spec fun increment {
    pragma aborts_if_is_partial = true;
    aborts_if !exists<Counter>(a);
}
```

<a name="risk-aborts-if-is-partial"></a>
> Note that there is a certain risk in setting `aborts_if_is_partial` to true, and best practice is to avoid it
in specifications of public functions and transaction scripts once those are considered finalized. This is because
changing the code after finalization
of the spec can add new (non-trivial, undesired) abort situations which the original specification did not
anticipate, but which will nevertheless silently pass verification.

If no aborts condition is specified for a function, abort behavior is unspecified. The function may or
may not abort, and verification will not raise any errors, whether `aborts_if_is_partial` is set or not.
In order to state that a function never aborts, use `aborts_if false`. One can use the pragma `aborts_if_is_strict`
to change this behavior; this is equivalent to as if an `aborts_if false` has been added to each function which
does not have an explicit `aborts_if` clause.

> NOTE: the pragmas `aborts_if_is_partial` and `aborts_if_is_strict` are not yet implemented. Current behavior
> is as they both would be set to their respective default (false).

## TerminatesIf Condition

> NOTE: this conditions is currently not implemented and a proposal by @DavidLDill

The `terminates_if` condition allows to positively express when a function is expected to terminate with no abortion.
In the presence of `pragma aborts_if_is_partial = true` is true, it might help
to minimize the risk of this model as discussed in the [note](#risk-aborts-if-is-partial) above.

If there are multiple `terminates_if` conditions, they are or-ed into a combined terminates condition. That is,
each individual `terminates_if` is a condition under which the function should always succeed.

Consider a combined aborts condition `A` (which is `false` if there is no `aborts_if`) and a combined terminates
condition `T ` (which is false if there is no `terminates_if`). Then abortion of a function is governed by the
following predicate:

```
A ==>  function_aborts && T ==> !functions_abort    // if aborts_if_is_partial is true
A <==> function_aborts && T ==> !functions_abort    // if aborts_if_is_partial is false
```

Notice that in the 2nd case, we should expect for a sound specification that `!A ==> T`, so
termination conditions are in fact redundant if `aborts_if_is_partial = true`.

> Note that mixing `aborts_if` and `terminates_if` conditions can cause unsound (contradicting) specifications,
> as illustrated above, and should be done with care.


## Requires Condition

The `requires` condition is a spec block member which postulates a pre-condition for a function. In contrast
to an `aborts_if` condition, if the pre-condition of a function does not hold, the result of calling the function
is undefined. The prover verifies that such a situation does not occur: for each call to the function, the prover
checks that the `requires` condition holds for the given parameters and global state. In turn, when the function
is executed, the prover can assume that the pre-condition holds.

An example of a precondition is the following:

```move
spec fun increment {
    requires global<Counter>(a).value < 255;
}
```

Notice that in the presence of an `aborts_if` with a negation of this condition
(as [illustrated here](#abortsif-condition)) the requires is redundant. In fact the prover takes the combined
aborts condition of a function to be or-ed with a requires condition. That is `requires P` is translated to
`requires P || A` (with `A` the combined aborts condition). For our example,
`requires global<Counter>(a).value < 255 || global<Counter>(a).value == 255` simplifies to `requires true`.

> The automatic enrichment of `requires` with the aborts condition is not yet implemented.


## Requires Module Condition

The `requires module` condition is a form of requires which is used to model module level invariants. It is
usually not explicitly used, but automatically derived from [`invariant module`](#invariant-condition-on-functions).

This condition is similar to the `requires` condition except that it is *assumed* to hold when a function is called
instead of being verified. That is justified by that every function call to the module [ensures](#ensures-condition)
on exit that the same condition holds as well.

> Note that maintaining the soundness of `requires module` is the responsibility of the specifier. We are looking
> at potentially automating the check for soundness in the future.

## Ensures Condition

The `ensures` condition postulates a post-condition for a function which must be satisfied when the function
terminates successfully (i.e. does not abort). The prover will verify each `ensures` to this end.

An example for the `ensures` condition is the following:

```move
spec fun increment {
    ensures global<Counter>(a) == old(global<Counter>(a)) + 1;
}
```

Within the expression for the `ensures` condition, one can use the `old` function. Within the `old` function,
any access to state (global resource memory or mutable reference parameters) refers to the value of this
state at point of entering the function. The `old` context also applies to any [helper functions](#helper_functions)
used in the expression. For example, the below is equivalent to the above:

```move
spec module {
    define get_counter(a: address): num { global<Counter>(a).value }
}
spec fun increment {
    ensures get_counter(a) == old(get_counter(a)) + 1;
}
```

## Assume and Assert Conditions

Those conditions can only appear in code spec blocks.

TBD

## Invariant Condition on Functions

The `invariant` condition on a function is simply a shortcut for a `requires` and `ensures` with the same predicate.
Similarly, the `invariant module` condition is a shortcut for `requires module` and `ensures` with the same predicate.

Thus the following spec block:

```move
spec fun increment {
    invariant module global<Counter>(a).value < 128;
}
```

... is equivalent to:

```move
spec fun increment {
    requires module global<Counter>(a).value < 128;
    ensures global<Counter>(a).value < 128;
}
```

## Invariant Condition on Modules

The `invariant` or `invariant module` condition can also be associated with a `spec module` block. In this case,
the condition will simply be copied to each *public* function of the module. See the
[schema apply operation](#schema-apply-operation) for an alternative and more flexible way to associate
conditions with multiple functions.

## Invariant Condition on Structs

If the `invariant` condition is applied to a struct, it expresses a well-formedness property of the struct's data.
Any instance of this struct which is currently not mutated will satisfy this property (with exceptions as
outlined below).

For example, we can postulate an invariant on our counter that it never must exceed the value of 127:

```move
spec struct Counter {
    invariant value < 128;
}
```

A struct invariant is checked by the prover whenever the struct value is constructed (packed). While the struct
is mutated (e.g. via a `&mut Counter`) the invariant does *not* hold (but see exception below). In general,
we consider mutation as an implicit unpack, and end of mutation as a pack.

The Move language semantics unambiguously identifies the point when mutation ends and starts.
This follows from the borrow semantics of Move, and includes mutation via an enclosing struct.
(The mutation of an inner struct ends when the mutation of the root struct where mutation started ends.)

There is one exception to this rule. When a mutable reference to a struct declared in module M is passed into
a *public* function of M which does by itself *not* return any other mutable reference (which could be borrowed from
the input parameter), we treat this parameter as "packed". That means, on function entry, we will unpack it
and on function exit we will pack again, enforcing the invariant. This treatment is in place to enable
use cases for [specification variables](#specification-variables-and-packunpack-invariants). It also reflects that in
Move, struct data can only be mutated within the module which declares the struct, so for an outside caller
of the public function, the mutable reference can actually not be mutated unless by calling public functions of
module M again. It is a significant simplification of the verification problem to exploit this in the encoding.

## Specification Variables and Pack/Unpack Invariants

The Spec language supports maintaining so called *spec variables*, which are often also called *ghost variables*
in the verification community. Those variables allow to represent global information derived from the global state
of resources which can be used in specifications. An example use case is f.i. to compute the sum of all coins
available in the system and specify that it can be only changed in certain scenarios.

We illustrate this feature by introducing a spec variable which maintains the sum of all `Counter` resources from
our running example. First, a spec variable is introduced via spec module block as follows:

```move
spec module {
    global sum_of_counters: num;
}
```

This value is going to be updated whenever a `Counter` is packed or unpacked. (Recall that mutation is
interpreted as an implicit unpack and pack):

```move
spec struct Counter {
    invariant pack sum_of_counters = sum_of_counters + value;
    invariant unpack sum_of_counters = sum_of_counters - value;
}
```

Now we may for example want to specify that the sum of all Counter instances in the global state should never
exceed a particular value. We can do this as follows:

```move
spec module {
    invariant module sum_of_counters < 4711;
}
```

This module invariant will expand to a `requires module` and `ensures` with the above condition for the `increment`
function. Obviously,
this will fail verification. We would need to extend the code, for example, by introducing a resource on
contract level which tracks counter values. This will be left open here because it is not relevant for
explaining the spec language.

Note that spec variables can be referenced also from helper functions. Moreover, spec variables can be generic:

```move
spec module {
    global some_generic_var<T>: num;
}
```

When using such a spec variable, a type parameter need to be provided, as in `some_generic_var<u64>`. Effectively,
a generic spec variable is like a family of variables indexed by types.

## Schemas

Schemas are a means for structuring specifications by grouping properties together. Semantically, they are just
syntactic sugar which expand to conditions on functions, structs, or modules.

Schemas are used as such:

```move
spec schema IncrementAborts {
    a: address;
    aborts_if !exists<Counter>(a);
    aborts_if global<Counter>(a).value == 255;
}

spec fun increment {
    include IncrementAborts;
}
```

Each schema may declare a number of typed variable names and a list of conditions over those variables. All supported
condition types can be used in schemas. The schema can then be included in another spec block:

- If that spec block is for a function or a struct, all variable names the schema declares must be matched against
  existing names of compatible type in the context.
- If a schema is included in another schema, existing names are matched and must have the same type, but non-existing
  names will be added as new declarations to the inclusion context.

When a schema is included in another spec block, it will be checked whether the conditions it contains are allowed
in this block. For example, including the schema `IncrementAborts` into a struct spec block will lead to a compile
time error.

When a schema is included the names it declares can also bound by expressions. For example, one
can write `include IncrementAborts{a: some_helper_address()}`. Effectively, not providing a binding
is equivalent to writing `IncrementAborts{a: a}` if `a` is an existing name in scope.

Schemas can be generic. Generic schemas must be fully instantiated where they are included; type inference is
not available for schemas.

## Schema Expressions

When a schema is included, one can use a limited set of Boolean operators as follows:

- `P ==> SchemaExp`: all conditions in the schema will be prefixed with `P ==> ..`.
    Conditions which are not based on boolean expressions will be rejected.
- `if (P) SchemaExp1 else SchemaExp2`: this is treated similar as including both
  `P ==> SchemaExp1` and `!P ==> SchemaExp2`.
- `SchemaExp1 && SchemaExp2`: this is treated as two includes for the both schema
   expressions.

## Schema Apply Operation

One of the main uses cases for schemas is to be able to name a group of properties and then apply those
to a set of functions. This is achieved by the `apply` operator. The `apply` spec block member can only appear
in module spec blocks.

The general form of the apply operator is `apply Schema to FunctionPattern, .. except FunctionPattern, ..`. Here,
`Schema` can be a schema name or a schema name plus formal type arguments.
 `FunctionPatterns` consists of an optional
visibility modifier `public` or `internal` (if not provided, both visibilities will match), a name pattern in
the style of a shell file pattern (e.g. `*`, `foo*`, `foo*bar`, etc.), and finally an optional type argument list.
All type arguments provided to `Schema` must be bound in this list and vice versa.

The apply operator includes the given schema in all function spec blocks which match the patterns, except those
which are excluded via the `except` patterns.

A typical use of the apply operator is to provide module invariants. Example:

```move
spec schema ModuleInvariant {
    invariant module sum_of_counters < 4711;
}

spec module {
    // Include invariant in both public and private functions except the initialize function.
    apply ModuleInvariant to * except initialize;
}
```

Notice that the mechanism we described for [module level invariants](#invariant-condition-on-modules) cannot provide
this semantics. It only applies to public function, and does not allow to exclude special functions, like
`initialize` which is a private function used in a special way in Libra genesis. Module invariants can also
not deal with generic functions, which `apply` can.

> The `apply` operator need to be handled with care, because it is easy to get something wrong
with it. For example, in `apply Schema<T> to *<T>`, those functions which have not exactly one
type argument will be silently excluded because they do not match. On the other hand, until now, this kind
of mistakes often lead to verification errors, and not to the more dangerous instance of unintended
verification success.
