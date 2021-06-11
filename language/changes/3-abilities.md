# Abilities

* Status: Implemented in Move 1.2

## Introduction

Abilities are a new feature in Move version 1.2 to give more control over what actions are permissible for values of a given type. Previously in Move’s type system, there were copyable values and `resource` values. In this old system, copyable values were unrestricted, but resource values could not be copied and had to be used. With abilities, Move’s type system grants more fine grained control, allowing types to specifically allow certain operations for their values that were previously implicitly allowed/denied depending on the “kind” (copyable or resource).

## Motivations

Move’s kind system (copyable vs. resource structs) has been great, but it is not expressive enough for certain applications. Resource types are very powerful, but the system bundles up a lot of behavior into one kind, which does not always meet application needs. In the Diem Framework, there has been a need for a “hot potato” type for capabilities—that is some type that cannot be copied or dropped (like a resource), but cannot be stored in global storage. It is then a “hot potato” since you must keep passing it around and it must be consumed before the transaction completes. A type with these restrictions would allow the Move prover to verify the usage of these specific capabilities.

Extending the kind system just to handle this “hot potato” example is likely not the best future-proof solution. A more granular system can give programmers more control to implement not only “hot potato” types but also serve yet unknown needs. As such, we want the new system to be flexible enough to be extended in the future without requiring another large refactoring of the type system.

## Description

The kinds copyable and resource are replaced by four new *abilities*. These abilities gate access to various bytecode instructions. In order for a value to be used with the bytecode instruction, it must have the ability required (if one is required at all—not every instruction is gated by an ability).

### The Abilities

The four added abilities are `copy`, `drop`, `store`, and `key`. Broken down in detail:

* `copy`
    * Allows values of types with this ability to be copied
    * Gates: `CopyLoc` and `ReadRef`
    * If a value has `copy`, all values reachable inside of that value have `copy`.
* `drop`
    * Allows values of types with this ability to be popped/dropped.
        * Ownership does not *have* to be transferred
    * Gates: `Pop`, `WriteRef`, `StLoc`, `Eq` and `Neq`
    * Values left in local variables must have `drop` when the function returns with `Ret`
    * If a value has `drop`, all values reachable inside of that value have `drop`.
* `store`
    * Allows values of types with this ability to exist inside a struct in global storage
        * But not necessarily as a top-level value in global storage
    * This is the only ability currently that is not directly checked by an operation. But it is indirectly checked when `key` is checked
    * If a value has `store`, all values reachable inside of that value have `store`.
* `key`
    * Allows the type to serve as a key for global storage operations
        * Letting values of types with this ability exist at the top-level of global storage
    * Gates: `MoveTo`, `MoveFrom`, `BorrowGlobal`, `BorrowGlobalMut`, and `Exists`
    * If a value has `key`, all values reachable inside of that value have `store`.

### Primitive Types

Most primitive types all have `copy`, `drop`, and `store` with the exception of `signer`.

* `bool`, `u8`, `u64`, `u128`, and `address` all have `copy`, `drop`, and `store`.
* `signer` has `drop`
    * Cannot be copied and cannot be put into global storage
* `vector<T>` may have `copy`, `drop`, and `store` depending on the abilities of `T`.
    * See [Conditional Abilities with Generic Types](#Conditional-Abilities-with-Generic-Types) for more details.
* Immutable references `&` and mutable references `&mut` both have `copy` and `drop`.
    * This refers to copying and dropping the reference itself, not what they refer to.
    * References cannot appear in global storage, hence they do not have `store`.

### Annotating Structs

To declare that a `struct` has an ability, it is declared with `has <ability>` after the struct name but before the fields. For example:

```
struct Ignorable has drop { f: u64 }
struct Pair has copy, drop, store { x: u64, y: u64 }
```

In this case `Ignorable` has the `drop` ability, and `Pair` has `copy`, `drop`, and `store`.

When declaring a struct’s abilities, certain requirements are placed on the fields. All fields must satisfy these constraints. These rules are necessary so that structs satisfy the reachability rules for abilities given above. If a struct is declared with the ability...

* `copy`, all fields must have `copy`
* `drop`, all fields must have `drop`
* `store`, all fields must have `store`
* `key`, all fields must have `store`
    * `key` is the only ability currently that doesn’t require itself.

For example:

```
// A struct without any abilities
struct NoAbilities {}

struct WantsCopy has copy {
    f: NoAbilities, // ERROR 'NoAbilities' does not have 'copy'
}
```

and similarly:

```
// A struct without any abilities
struct NoAbilities {}

struct MyResource has key {
    f: NoAbilities, // Error 'NoAbilities' does not have 'store'
}
```

### Basic Examples

**Copy**

```
struct NoAbilities {}
struct S has copy, drop { f: bool }

fun example(x: u64, s: S) {
    let x2 = copy x; // Valid, 'u64' has 'copy'
    let s2 = copy s; // Valid, 'S' has 'copy'
}

fun invalid(account: signer, n: NoAbilities) {
    let a2 = copy account; // Invalid, 'signer' does not have 'copy'
    let n2 = copy n; // Invalid, 'NoAbilities' does not have 'drop'
}
```

**Drop**

```
struct NoAbilities {}
struct S has copy, drop { f: bool }

fun unused() {
    true; // Valid, 'bool' has 'drop'
    S { f: false }; // Valid, 'S' has 'drop'
}

fun left_in_local(account: signer): u64 {
    let b = true;
    let s = S { f: false };
    // Valid return: 'account', 'b', and 's' have values
    // but 'signer', 'bool', and 'S' have 'drop'
    0
}

fun invalid_unused() {
    NoAbilities {}; // Invalid, Cannot ignore 'NoAbilities' without 'drop'
}

fun invalid_left_in_local(): u64 {
    let n = NoAbilities{};
    // Invalid return: 'n' has a value and 'NoAbilities' does not have 'drop'
    0

}
```

**Store**

```
// 'MyInnerResource' is declared with 'store' so all fields need 'store'
struct MyInnerResource has store {
    yes: u64, // Valid, 'u64' has 'store'
    // no: signer, Invalid, 'signer' does not have 'store'
}

// 'MyResource' is declared with 'key' so all fields need 'store'
struct MyResource has key {
    yes: u64, // Valid, 'u64' has 'store'
    inner: MyInnerResource, // Valid, 'MyInnerResource' has 'store'
    // no: signer, Invalid, 'signer' does not have 'store'
}
```

**Key**

```
struct NoAbilities {}
struct MyResource has key { f: u64 }

fun valid(account: &signer) acquires MyResource {
    let addr = Signer::address_of(account);
    let has_resource = exists<MyResource>(addr); // Valid, 'MyResource' has 'key'
    if (!has_resource) {
        move_to(account, MyResource { f: 0 }) // Valid, 'MyResource' has 'key'
    };
    let r = borrow_global_mut<MyResource>(addr) // Valid, 'MyResource' has 'key'
    r.f = r.f + 1;
}

fun invalid(account: &signer) {
   let has_it = exists<NoAbilities>(addr); // Invalid, 'NoAbilities' does not have 'key'
   let NoAbilities {} = move_from<NoAbilities>(addr); // Invalid, does not have 'key'
   move_to(account, NoAbilities {}); // Invalid, 'NoAbilities' does not have 'key'
   borrow_global<NoAbilities>(addr); // Invalid, 'NoAbilities' does not have 'key'
}
```

### Constraining Generics

Abilities can used to constrain generics, meaning that only types with that ability can instantiate that type parameter. This can be used on both function and struct type parameters:

```
fun foo<T: copy>(x: T): (T, T) { (copy x, x) }
struct CopyCup<T: copy> has copy { item: T }
```

Type parameters can have more than one constraint, signified with `+`

```
fun bar<T: copy + drop>(x: T): T { copy x }
struct AllCup<T: copy + drop + store + key> has copy, drop, store, key { item: T }
```

### Conditional Abilities with Generic Types

When abilities are annotated on a generic type, not all instances of that type are guaranteed to have that ability. Consider this struct declaration:

```
struct Cup<T> has copy, drop, store, key { item: T }
```

The type parameter `T` is assumed to be used inside of the struct, so the abilities are only granted if the type parameters meet the requirements described above for fields. That means:

* `Cup` has the ability `copy` only if `T` has `copy`.
* It has `drop` only if `T` has `drop`.
* It has `store` only if `T` has `store`.
* It has `key` only if `T` has `store`.

This behavior might be a bit confusing at first, but it is extremely useful for collection-like types. Consider `vector`: we could consider it to have the following type declaration:

```
vector<T> has copy, drop, store;
```

With this, you can copy a `vector` value only if the inner elements can be copied. You can ignore a `vector` value only if the inner elements can be ignored/dropped. And, a `vector` can be in global storage only if the inner elements can be in global storage.

### More Examples

**Conditional Copy**

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

**Drop**

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

**Store**

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

**Key**

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

### Backwards Compatibility

The new ability system is backwards compatible with the kind system in nearly all cases. At the bytecode level, old modules and scripts with kinds can be loaded as if they were written with abilities.

For any struct:

* If it was declared as a “copyable”, non-`resource` struct, the struct will be given the abilities `copy`, `drop`, and `store`.
* If it was declared as a `resource`, the struct will be given the abilities `store` and `key`.
* For type parameters:
    * `copyable` becomes `copy + drop`
    * `resource` becomes `key`
    * `store` is not given as it is not needed in the constraint position. Any usage will still work.

For any function:

* For type parameters:
    * `copyable` becomes `copy + drop + store`
    * `resource` becomes `key + drop`
    * `store` is needed as it is not simple to determine if the type parameter will be used with global storage operations.

Putting these rules together, the old code

```
struct S<T: copyable> {}
resource struct R<T1: resource, T2> {}

fun foo<Tc: copyable, Tr: resource, T>() {}
```

will be loaded as if it was written as:

```
struct S<T: copy + drop> has copy, drop, store {}
struct R<T1: key, T2> {}

fun foo<Tc: copy + drop + store, Tr: key + store, T: store>() {}
```

This leads to one spot where there is a breaking change, namely any function instantiated with `signer` as a type parameter will not now load because the type parameter will  have the `store` constraint—all old function type parameters are given the `store` constraint—but `signer` does not have `store`. Given the restricted usage of `signer`, this is likely an extreme edge case, and we do not foresee it being an issue in practice for any project.

## Alternatives

### Extending the Kind System

For the main motivating example for this change, it was considered to add a “hot potato” kind to the system.
In the kind system there was:

* `Copyable` which corresponds to `copy + drop + store`,
* `Resource` which corresponds to `key + store`
* `All` which sometimes corresponds to `store` and sometimes to no-ability

Often this was viewed with a sub-kinding system where `Copyable <: All`  and `Resource <: All`. Adding a `HotPotato` kind to this system would be bit tricky, possibly giving a hierarchy of `Copyable <: All` and `Resource <: HotPotato <: All`. But, this could become a mess if:

* There needed to be an `AllWithStore` kind, giving `Copyable <: AllWithStore <: All` and `Resource <: AllWithStore <: All` and `Resource <: HotPotato <: All`.
* If any other kind was added, the complexity could quickly explode.

The complexity around this sub-kinding approach led to the more granular approach described above with abilities. We were particularly worried about needing another kind in a year or two and having the whole thing collapse. With abilities, we can easily add new ones over time if needed.

### Explicit Conditional Abilities

The current rules around generics being conditional for generic types might be potentially confusing, especially given the keyword `has`. For instance:

```
struct Cup<T> has copy, drop, store { item: T }
```

Despite effectively saying “has copy” and “has drop” and “has store”, `Cup<T>` may or may not have the ability depending on what `T` is. This might be rather confusing. It was considered then that for generic types you would write:

```
struct Cup<T> has ?copy, ?drop, ?store { item: T }
```

This would mean exactly what it means today, where it may or may not have the ability depending on `T` and then using `has` without the question mark `?`

```
struct Ex<T> has copy {}
```

would be equivalent to:

```
struct Ex<T: copy> has ?copy {}
```

The potential problem then comes in that there are a lot of combinations that are meaningless. So, in many cases the compiler would yell at you that there is really just one valid choice.

* `struct Ex<T: copy> has copy` is sort of redundant and could just be `struct Ex<T> has copy`.
* For a non-generic type, `struct Ex has ?copy` is meaningless in some way, as every instance has the ability, and it is the same as `struct Ex has copy`.

In short, having the option to have the question mark `?` caused there to be more cases and possibly more confusion. Furthermore this system was not more expressive, as a programmer could always annotate the generic `struct Cup<T: copy> has copy`, this would force every instance to have `copy` in a more explicit manner. In short, just having one option and one rule that might be a bit more confusing in the way it reads at first, but reduces the amount of complexity to consider when declaring a new struct.

### Alternative Names

Many different names were considered for all aspects of the abilities system.

* For the name “ability” itself, “kinds”, “traits”, “type classes”, and “interfaces” were all considered. But these items used in other programming languages are usually used to describe programmer-defined items. There is no way for programmers to define their own abilities. Additionally, abilities do not give anything that looks like dynamic dispatch. So, while those other names might be more familiar, we worried they would be too misleading.
* `copyable` and `dropable` and `storable` were all considered, but they felt too wordy. Shorter names felt more appropriate.
* `mustuse` or `mustmove` were considered for `drop`, but again, the more concise name felt better even if the others were more informative.
* `resource` was considered instead of `key`. The `key` ability is very similar to the `resource` keyword in many cases, but it felt very weird that something like a `Coin` which might have been `resource struct Coin` in the old system, would not be `struct Coin has store` and would not be a “resource”. Thus we are saving the word “resource” to be used in documentation for any time that does not have `copy` or `drop`.
