# Abilities

Abilities are a typing feature in Move that control what actions are permissible for values of a given type. This system grants fine grained control over the "linear" typing behavior of values, as well as if and how values are used in global storage. This is implemented by gating access to certain bytecode instructions so that for a value to be used with the bytecode instruction, it must have the ability required (if one is required at all—not every instruction is gated by an ability).

<!-- TODO future section on detailed walk through maybe. We have some examples at the end but it might be helpful to explain why we have precisely this set of abilities

If you are already somewhat familiar with abilities from writing Move programs, but are still confused as to what is going on, it might be helpful to skip to the [motivating walkthrough](#motivating-walkthrough) section to get an idea of what the system is setup in the way that it is. -->

## The Four Abilities

The four abilities are:

* [`copy`](#copy)
    * Allows values of types with this ability to be copied.
* [`drop`](#drop)
    * Allows values of types with this ability to be popped/dropped.
* [`store`](#store)
    * Allows values of types with this ability to exist inside a struct in global storage.
* [`key`](#key)
    * Allows the type to serve as a key for global storage operations.

### `copy`

The `copy` ability allows values of types with that ability to be copied. It gates the ability to copy values out of local variables with the [`copy`](./variables.md#move-and-copy) operator and to copy values via references with [dereference `*e`](./references.md#reading-and-writing-through-references).

If a value has `copy`, all values contained inside of that value have `copy`.

### `drop`

The `drop` ability allows values of types with that ability to be dropped. By dropped, we mean that value is not transferred and is effectively destroyed as the Move program executes. As such, this ability gates the ability to ignore values in a multitude of locations, including:
* not using the value in a local variable or parameter
* not using the value in a [sequence via `;`](./variables.md#expression-blocks)
* overwriting values in variables in [assignments](./variables.md#assignments)
* overwriting values via references when [writing `*e1 = e2`](./references.md#reading-and-writing-through-references).

If a value has `drop`, all values contained inside of that value have `drop`.

### `store`

The `store` ability allows values of types with this ability to exist inside of a struct (resource) in global storage, *but* not necessarily as a top-level resource in global storage. This is the only ability that does not directly gate an operation. Instead it gates the existence in global storage when used in tandem with `key`.

If a value has `store`, all values contained inside of that value have `store`

### `key`

The `key` ability allows the type to serve as a key for [global storage operations](./global-storage-operators.md). It gates all global storage operations, so in order for a type to be used with `move_to`, `borrow_global`, `move_from`, etc., the type must have the `key` ability. Note that the operations still must be used in the module where the `key` type is defined (in a sense, the operations are private to the defining module).

If a value has `key`, all values contained inside of that value have `store`. This is the only ability with this sort of asymmetry.

## Builtin Types

Most primitive, builtin types have `copy`, `drop`, and `store` with the exception of `signer`, which just has `store`

* `bool`, `u8`, `u64`, `u128`, and `address` all have `copy`, `drop`, and `store`.
* `signer` has `drop`
    * Cannot be copied and cannot be put into global storage
* `vector<T>` may have `copy`, `drop`, and `store` depending on the abilities of `T`.
    * See [Conditional Abilities and Generic Types](#conditional-abilities-and-generic-types) for more details.
* Immutable references `&` and mutable references `&mut` both have `copy` and `drop`.
    * This refers to copying and dropping the reference itself, not what they refer to.
    * References cannot appear in global storage, hence they do not have `store`.

None of the primitive types have `key`, meaning none of them can be used directly with the [global storage operations](./global-storage-operators.md).

## Annotating Structs

To declare that a `struct` has an ability, it is declared with `has <ability>` after the struct name but before the fields. For example:

```move
struct Ignorable has drop { f: u64 }
struct Pair has copy, drop, store { x: u64, y: u64 }
```

In this case: `Ignorable` has the `drop` ability. `Pair` has `copy`, `drop`, and `store`.


All of these abilities have strong guarantees over these gated operations. The operation can be performed on the value only if it has that ability; even if the value is deeply nested inside of some other collection!

As such: when declaring a struct’s abilities, certain requirements are placed on the fields. All fields must satisfy these constraints. These rules are necessary so that structs satisfy the reachability rules for the abilities given above. If a struct is declared with the ability...

* `copy`, all fields must have `copy`.
* `drop`, all fields must have `drop`.
* `store`, all fields must have `store`.
* `key`, all fields must have `store`.
    * `key` is the only ability currently that doesn’t require itself.

For example:

```move
// A struct without any abilities
struct NoAbilities {}

struct WantsCopy has copy {
    f: NoAbilities, // ERROR 'NoAbilities' does not have 'copy'
}
```

and similarly:

```move
// A struct without any abilities
struct NoAbilities {}

struct MyResource has key {
    f: NoAbilities, // Error 'NoAbilities' does not have 'store'
}
```

## Conditional Abilities and Generic Types

When abilities are annotated on a generic type, not all instances of that type are guaranteed to have that ability. Consider this struct declaration:

```
struct Cup<T> has copy, drop, store, key { item: T }
```

It might be very helpful if `Cup` could hold any type, regardless of its abilities. The type system can *see* the type parameter, so it should be able to remove abilities from `Cup` if it *sees* a type parameter that would violate the guarantees for that ability.

This behavior might sound a bit confusing at first, but it might be more understandable if we think about collection types. We could consider the builtin type `vector` to have the following type declaration:

```
vector<T> has copy, drop, store;
```

We want `vector`s to work with any type. We don't want separate `vector` types for different abilities. So what are the rules we would want? Precisely the same that we would want with the field rules above.  So, it would be safe to copy a `vector` value only if the inner elements can be copied. It would be safe to ignore a `vector` value only if the inner elements can be ignored/dropped. And, it would be safe to put a `vector` in global storage only if the inner elements can be in global storage.

To have this extra expressiveness, a type might not have all the abilities it was declared with depending on the instantiation of that type; instead, the abilities a type will have depends on both its declaration **and** its type arguments. For any type, type parameters are pessimistically assumed to be used inside of the struct, so the abilities are only granted if the type parameters meet the requirements described above for fields. Taking `Cup` from above as an example:

* `Cup` has the ability `copy` only if `T` has `copy`.
* It has `drop` only if `T` has `drop`.
* It has `store` only if `T` has `store`.
* It has `key` only if `T` has `store`.

Here are examples for this conditional system for each ability:

### Example: conditional `copy`

```
struct NoAbilities {}
struct S has copy, drop { f: bool }
struct Cup<T> has copy, drop, store { item: T }

fun example(c_x: Cup<u64>, c_s: Cup<S>) {
    // Valid, 'Cup<u64>' has 'copy' because 'u64' has 'copy'
    let c_x2 = copy c_x;
    // Valid, 'Cup<S>' has 'drop' because 'S' has 'drop'
    let c_s2 = copy c_s;
}

fun invalid(c_account: Cup<signer>, c_n: Cup<NoAbilities>) {
    // Invalid, 'Cup<signer>' does not have 'copy'.
    // Even though 'Cup' was declared with copy, the instance does not have 'copy'
    // because 'signer' does not have 'copy'
    let c_account2 = copy c_account;
    // Invalid, 'Cup<NoAbilities>' does not have 'drop'
    // because 'NoAbilities' does not have 'drop'
    let c_n2 = copy c_n;
}
```

### Example: conditional `drop`

```
struct NoAbilities {}
struct S has copy, drop { f: bool }
struct Cup<T> has copy, drop, store { item: T }

fun unused() {
    Cup<bool> { item: true }; // Valid, 'Cup<bool>' has 'drop'
    Cup<S> { item: S { f: false }}; // Valid, 'Cup<S>' has 'drop'
}

fun left_in_local(c_account: Cup<signer>): u64 {
    let c_b = Cup<bool> { item: true };
    let c_s = Cup<S> { item: S { f: false }};
    // Valid return: 'c_account', 'c_b', and 'c_s' have values
    // but 'Cup<signer>', 'Cup<bool>', and 'Cup<S>' have 'drop'
    0
}

fun invalid_unused() {
    // Invalid, Cannot ignore 'Cup<NoAbilities>' because it does not have 'drop'.
    // Even though 'Cup' was declared with 'drop', the instance does not have 'drop'
    // because 'NoAbilities' does not have 'drop'
    Cup<NoAbilities> { item: NoAbilities {}};
}

fun invalid_left_in_local(): u64 {
    let n = Cup<NoAbilities> { item: NoAbilities {}};
    // Invalid return: 'c_n' has a value
    // and 'Cup<NoAbilities>' does not have 'drop'
    0
}
```

### Example: conditional `store`

```
struct Cup<T> has copy, drop, store { item: T }

// 'MyInnerResource' is declared with 'store' so all fields need 'store'
struct MyInnerResource has store {
    yes: Cup<u64>, // Valid, 'Cup<u64>' has 'store'
    // no: Cup<signer>, Invalid, 'Cup<signer>' does not have 'store'
}

// 'MyResource' is declared with 'key' so all fields need 'store'
struct MyResource has key {
    yes: Cup<u64>, // Valid, 'Cup<u64>' has 'store'
    inner: Cup<MyInnerResource>, // Valid, 'Cup<MyInnerResource>' has 'store'
    // no: Cup<signer>, Invalid, 'Cup<signer>' does not have 'store'
}
```

### Example: conditional `key`

```
struct NoAbilities {}
struct MyResource<T> has key { f: T }

fun valid(account: &signer) acquires MyResource {
    let addr = Signer::address_of(account);
     // Valid, 'MyResource<u64>' has 'key'
    let has_resource = exists<MyResource<u64>>(addr);
    if (!has_resource) {
         // Valid, 'MyResource<u64>' has 'key'
        move_to(account, MyResource<u64> { f: 0 })
    };
    // Valid, 'MyResource<u64>' has 'key'
    let r = borrow_global_mut<MyResource<u64>>(addr)
    r.f = r.f + 1;
}

fun invalid(account: &signer) {
   // Invalid, 'MyResource<NoAbilities>' does not have 'key'
   let has_it = exists<MyResource<NoAbilities>>(addr);
   // Invalid, 'MyResource<NoAbilities>' does not have 'key'
   let NoAbilities {} = move_from<NoAbilities>(addr);
   // Invalid, 'MyResource<NoAbilities>' does not have 'key'
   move_to(account, NoAbilities {});
   // Invalid, 'MyResource<NoAbilities>' does not have 'key'
   borrow_global<NoAbilities>(addr);
}
```
