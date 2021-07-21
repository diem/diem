---
title: "Address"
slug: "move-primitives-address"
hidden: false
metadata: 
  title: "Address"
  description: "Read about the Move primitive type, address, which is used to represent a location in global storage."
createdAt: "2021-02-04T01:03:45.223Z"
updatedAt: "2021-03-31T23:49:54.112Z"
---
`address` is a built-in type in Move that is used to represent locations (sometimes called accounts) in global storage. An `address` value is a 128-bit (16 byte) identifier. At a given address, two things can be stored: [Modules](doc:move-modules-and-scripts) and [Resources](doc:move-basics-structs-and-resources). 

Although an `address` is a 128 bit integer under the hood, Move addresses are intentionally opaque---they cannot be created from integers, they do not support arithmetic operations, and they cannot be modified. Even though there might be interesting programs that would use such a feature (e.g., pointer arithmetic in C fills a similar niche), Move does not allow this dynamic behavior because it has been designed from the ground up to support static verification.

You can use runtime address values (values of type `address`) to access resources at that address. You *cannot* access modules at runtime via address values.

## Literals

`address` literals are 16-byte hex literals, i.e. `0x<hex encoded value>`. For convenience, leading `0`s are added for literals that are too short.

### Examples

```rust
let a1: address = 0x1; // shorthand for 0x00000000000000000000000000000001
let a2: address = 0x42; // shorthand for 0x00000000000000000000000000000042
let a3: address = 0xDEADBEEF; // // shorthand for 0x000000000000000000000000DEADBEEF
let a4: address = 0x0000000000000000000000000000000A;
```

## Global Storage Operations

The primary purpose of `address` values are to interact with the global storage operations.

`address` values are used with the `exists`, `borrow_global`, `borrow_global_mut`, and `move_from` [operations](doc:move-global-storage-operators).

The only global storage operation that *does not* use `address` is `move_to`, which uses [`signer`](doc:move-primitives-signer).

## Ownership

As with the other scalar values built-in to the language, `address` values are implicitly copyable, meaning they can be copied without an explicit instruction such as [`copy`](doc:move-basics-equality).