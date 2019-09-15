# Libra Canonical Serialization (LCS)

## Overview

This document defines Libra Canonical Serialization (LCS). LCS defines a
deterministic means for translating a message or data structure into bytes
irrespective of platform, architecture, or programming language.

## Background

In Libra, participants pass around messages or data structures that often times need to be signed
by a prover and verified by one or more verifiers. Serialization in this context refers to the
process of converting a message into a byte array. Many serialization approaches support loose
standards such that two implementations can produce two different byte streams that would
represent the same, identical message. While for many applications, non-deterministic
serialization causes no issues, it does so for applications using serialization for cryptographic
purposes. For example, given a signature and a message, a verifier may not unable to produce the
same serialized byte array constructed by the prover when the prover signed the message resulting
in a non-verifiable message. In other words, to ensure message verifiability when using
non-deterministic serialization, participants must either retain the original serialized bytes or
risk losing the ability to verify messages. This creates a burden requiring participants to
maintain both a copy of the serialized bytes and the deserialized message often leading to
confusion about safety and correctness. While there exist a handful of existing deterministic
serialization formats, there is no obvious choice.  To address this, we propose Libra Canonical
Serialization that defines a deterministic means for translating a message into bytes.

## Specification

LCS supports the following primitive types:

* Booleans
* Signed 8-bit, 16-bit, 32-bit, and 64-bit integers
* Unsigned 8-bit, 16-bit, 32-bit, and 64-bit  integers
* Length prefixed byte array
* UTF-8 Encoded Strings
* Structures

## General structure

The serialized format of a message does not self-specify its encoding, in other words LCS employs
stream encoding enabling storage of arbitrary and variable length messages. LCS defines the
structure of primitives like integers, booleans, and byte arrays. It uses a composition of these
primitives to support advanced data structures such as enumerations maps, and objects.
Because of this typeless but expressive nature, each message requires both a serializer and
reciprocal deserializer.

Unless specified, all numbers are stored in little endian, two's complement format.

### Booleans and Integers

|Type                       |Original data          |Hex representation |Serialized format  |
|---                        |---                    |---                |---                |
|Boolean                    |True / False           |0x01 / 0x00        |[01] / [00]        |
|8-bit signed integer       |-1                     |0xFF               |[FF]               |
|8-bit unsigned integer     |1                      |0x01               |[01]               |
|16-bit signed integer      |-4660                  |0xEDCC             |[CCED]             |
|16-bit unsigned integer    |4660                   |0x1234             |[3412]             |
|32-bit signed integer      |-305419896             |0xEDCBA988         |[88A9CBED]         |
|32-bit unsigned integer    |305419896              |0x12345678         |[78563412]         |
|64-bit signed integer      |-1311768467750121216   |0xEDCBA98754321100 |[0011325487A9CBED] |
|64-bit unsigned integer    |1311768467750121216    |0x12345678ABCDEF00 |[00EFCDAB78563412] |

### Byte Arrays

Byte arrays are length prefixed with a 32-bit unsigned integer as defined in
the primitive section of this document. Byte arrays must be 2^31 bytes or less.

Example:

Given a byte array of length 231800522: [0x00 0x01 ... 0xFE 0xFF]

LCS representation: [CAFE D00D 00 01 ... FE FF]

Where 231800522 serialized into [CAFE D00D]

### Strings

Strings by default are stored in UTF-8 format with a 32-bit unsigned int prefix for the byte
array representation of the UTF-8 string.

Given the string of length :ሰማይ አይታረስ ንጉሥ አይከሰስ።

Length prefixed format: [36000000E188B0E1889BE18BAD20E18AA0E18BADE189B3E188A8E188B520E18A95E18C89E188A520E18AA0E18BADE18AA8E188B0E188B5E18DA2]

Note: the string consists of 20 characters but 54 UTF-8 encoded bytes

### Structures

Structures are fixed sized types consisting of fields with potentially different types:

```
struct MyStruct {
  boolean: bool,
  bytes: Vec<u8>,
  label: String,
}
```

Each field within a struct is serialized in the order specified by the canonical structure
definition.

LCS would serialize an instance of MyStruct into:
[boolean bytes label]

Furthermore, structs can exist within other structs. LCS recurses into each struct and serializes
them in order. Consider

```
struct Wrapper {
  inner: MyStruct,
  name: String,
}
```

LCS would serialize an instance of Wrapper into:
[inner name]
With one layer of abstraction removed:
[boolean bytes label name]

There are no labels, the struct ordering defines the organization within the serialization
stream.

## Advanced Datatypes

Leveraging the primitive specification more advanced types can be serialized via LCS.

Advanced types include:

* Tuples
* Variable length arrays
* Maps (Key / Value Stores)
* Enumerations
* Optional data

### Tuples

Tuples are typed composition of objects:
(Type0, Type1)

Tuples can be considered unnamed structs and leverage the same organization as an anonymous
structure within LCS. Like structures, each object should be serialized using its well-defined
consistent serialization method and then placed sequentially into the bitstream in the order
defined within the tuple.

In byte representation:
[tuple.0, tuple.1]

Note: Tuples do not need length as they are fixed in length like structures are fixed in fields.

### Variable Length Arrays

Variable length arrays consist of a common object. In LCS, they are represented first with a
length prefix on the number of elements in the array and then the object serialization for each
object in the order it is stored within the array.

Assuming an array of objects, [obj0, obj1, obj2, ...]:

LCS would serialize an instance of this tuple into:
[length of array | obj0 obj1 obj2 ...]

### Maps (Key / Value Stores)

Maps can be considered a variable length array of length two tuples where Key points to Value is
represented as (Key, Value). Hence they should be serialized first with the length of number of
entries followed by each entry in lexicographical order as defined by the byte representation of
the LCS serialized key.

Consider the following map:

```
{
  "A" => "B",
  "C" => "D",
  "E" => "F"
}
```

LCS would serialize this into:
[ 3 A B C D E F]

(Note the above are already in lexicographic order)

### Enumerations

An enumeration is typically represented as:

```
enum Option {
  option0(u32),
  option1(u64)
}
```

wherein the enum object can only representation one of these options.

In LCS, each option is mapped to a specific unsigned 32-bit integer followed by optionally
serialized data if the type has an associated value.

option0 would be encoded as 0

and option1 would be encoded as 1

Examples:

* option0(5) -> [0000 0000 0500 0000]
* option1(6) -> [0100 0000 0600 0000 0000 0000]

### Optional Data

Optional or nullable data either exists in its full representation or does not. For example,

```
optional_data: Option(uint8); // Rust
uint8 *optional_data; // C
```

LCS represents this as such:
If the data, say optional\_data is equal to 8, is present:
[True data] -> [01 08]
If the data is not present:
[False] -> [00]

## Backwards compatibility

Advanced objects are only loosely defined but are more dependent upon the specification in which
they are used. LCS does not provide direct provisions for versioning or backwards / forwards
compatibility. A change in an objects structure could prevent historical clients from
understanding new clients and vice-versa.

## RawTransaction Serialization

Note: See `types/src/unit_tests/canonical_serializer_examples.rs` for verification of these
types in the Rust implementation of LCS.

RawTransaction:

```
struct RawTransaction {
    sender: AccountAddress,
    sequence_number: u64,
    payload: TransactionPayload,
    max_gas_amount: u64,
    gas_unit_price: u64,
    expiration_time: Duration,
}
```

- AccountAddress is represented as a variable length byte area wherein the byte area is the address itself
- u64 is a 64-bit unsigned integer
- TransactionPayload is an enum for either Program or WriteSet
- Duration is the time in seconds as a 64-bit unsigned integer

Program:

```
struct Program {
  code: Vec<u8>, // Variable length byte array
  args: Vec<TransactionArgument>, // Variable length array of TransactionArguments
  modules: Vec<Vec<u8>>, // Variable length array of variable length byte arrays
}
```

TransactionArgument:

```
enum TransactionArgument {
    U64(u64), // unsigned 64-bit integer
    Address(AccountAddress), // Address represented as a variable length byte array
    ByteArray(ByteArray), // Variable length byte array
    String(String), // Variable length byte array of a string in UTF8 format
}
```

WriteSet:

```

struct WriteSet {
   // Variable length array of the tuple containing AccessPath and WriteOp
  write_set: Vec<(AccessPath, WriteOp)>,
}
```


AccessPath:

```
struct AccessPath {
  address: AccountAddress, // Address represented as a variable length byte array
  path: Vec<u8>, // Variable length byte array
}
```

WriteOp:

```
struct WriteOp {
  is_value: bool,
  value: Vec<u8>, // This is optional and not written if is_value is false
}
```

### Examples

**AccountAddress**

String representation:
```
ca820bf9305eb97d0d784f71b3955457fbf6911f5300ceaa5d7e8621529eae19
```

LCS representation:
[20000000CA820BF9305EB97D0D784F71B3955457FBF6911F5300CEAA5D7E8621529EAE19]

where 20000000 is the size of the address: 32 represented as a little endian 32-bit unsigned integer

**TransactionArgument u64**

String representation:

```
{U64: 9213671392124193148}
```

LCS representation:
[000000007CC9BDA45089DD7F]

**TransactionArgument AccountAddress**

String representation:

```
{ADDRESS: 2c25991785343b23ae073a50e5fd809a2cd867526b3c1db2b0bf5d1924c693ed}
```

LCS representation:
[01000000200000002C25991785343B23AE073A50E5FD809A2CD867526B3C1DB2B0BF5D1924C693ED]

**TransactionArgument String**

String representation:
```
{STRING: Hello, World!}
```

LCS representation:
[020000000D00000048656C6C6F2C20576F726C6421]

**TransactionArgument ByteAddress**

String representation:
```
{ByteArray: 0xb"cafed00d"}
```

LCS representation:
[0300000004000000CAFED00D]

**Program**

String representation:

```
{
  code: "move",
  args: [{STRING: CAFE D00D}, {STRING: cafe d00d}],
  modules: [[CA][FED0][0D]],
}
```

LCS representation:
[040000006D6F766502000000020000000900000043414645204430304402000000090000006361666520643030640300000001000000CA02000000FED0010000000D]

**AccessPath**

String representation:
```
{
  address: 9a1ad09742d1ffc62e659e9a7797808b206f956f131d07509449c01ad8220ad4,
  path: 01217da6c6b3e19f1825cfb2676daecce3bf3de03cf26647c78df00b371b25cc97
}
```

LCS representation:
[200000009A1AD09742D1FFC62E659E9A7797808B206F956F131D07509449C01AD8220AD42100000001217DA6C6B3E19F1825CFB2676DAECCE3BF3DE03CF26647C78DF00B371B25CC97]

**WriteOp Deletion**

LCS representation:
[00000000]

**WriteOp Value**

String representation:
cafed00d
[0100000004000000CAFED00D]

**WriteSet**

String representation:
```
[
  (
    AccessPath {
      address: a71d76faa2d2d5c3224ec3d41deb293973564a791e55c6782ba76c2bf0495f9a,
      path: 01217da6c6b3e19f1825cfb2676daecce3bf3de03cf26647c78df00b371b25cc97
    },
    Deletion
  ),
  (
    AccessPath {
      address: c4c63f80c74b11263e421ebf8486a4e398d0dbc09fa7d4f62ccdb309f3aea81f,
      path: 01217da6c6b3e19f18
    },
    cafed00d
  )
]
```

LCS representation:
[0200000020000000A71D76FAA2D2D5C3224EC3D41DEB293973564A791E55C6782BA76C2BF0495F9A2100000001217DA6C6B3E19F1825CFB2676DAECCE3BF3DE03CF26647C78DF00B371B25CC970000000020000000C4C63F80C74B11263E421EBF8486A4E398D0DBC09FA7D4F62CCDB309F3AEA81F0900000001217DA6C6B3E19F180100000004000000CAFED00D]

**TransactionPayload with a Program**

String representation:
```
{
  code: "move",
  args: [{STRING: CAFE D00D}, {STRING: cafe d00d}],
  modules: [[CA][FED0][0D]],
}
```

LCS representation:
[00000000040000006D6F766502000000020000000900000043414645204430304402000000090000006361666520643030640300000001000000CA02000000FED0010000000D]

**TransactionPayload with a WriteSet**

String representation:
```
[
  (
    AccessPath {
      address: a71d76faa2d2d5c3224ec3d41deb293973564a791e55c6782ba76c2bf0495f9a,
      path: 01217da6c6b3e19f1825cfb2676daecce3bf3de03cf26647c78df00b371b25cc97
    },
    Deletion
  ),
  (
    AccessPath {
      address: c4c63f80c74b11263e421ebf8486a4e398d0dbc09fa7d4f62ccdb309f3aea81f,
      path: 01217da6c6b3e19f18
    },
    cafed00d
  )
]
```

LCS representation:
[010000000200000020000000A71D76FAA2D2D5C3224EC3D41DEB293973564A791E55C6782BA76C2BF0495F9A2100000001217DA6C6B3E19F1825CFB2676DAECCE3BF3DE03CF26647C78DF00B371B25CC970000000020000000C4C63F80C74B11263E421EBF8486A4E398D0DBC09FA7D4F62CCDB309F3AEA81F0900000001217DA6C6B3E19F180100000004000000CAFED00D]

**RawTransaction with a Program**

String representation:

```
{
    sender: 3a24a61e05d129cace9e0efc8bc9e33831fec9a9be66f50fd352a2638a49b9ee,
    sequence_number: 32,
    payload: Program {
    code: "move",
    args: [{STRING: CAFE D00D}, {STRING: cafe d00d}],
    modules: [[CA][FED0][0D]],
  } ,
  max_gas_amount: 10000,
  gas_unit_price: 20000,
  expiration_time: 86400 seconds
}
```

LCS representation:
[200000003A24A61E05D129CACE9E0EFC8BC9E33831FEC9A9BE66F50FD352A2638A49B9EE200000000000000000000000040000006D6F766502000000020000000900000043414645204430304402000000090000006361666520643030640300000001000000CA02000000FED0010000000D1027000000000000204E0000000000008051010000000000]

**RawTransaction**

String representation:

```
{
  sender: c3398a599a6f3b9f30b635af29f2ba046d3a752c26e9d0647b9647d1f4c04ad4,
  sequence_number: 32,
  payload: WriteSet {
    write_set: [
      (
        AccessPath {
          address: a71d76faa2d2d5c3224ec3d41deb293973564a791e55c6782ba76c2bf0495f9a,
          path: 01217da6c6b3e19f1825cfb2676daecce3bf3de03cf26647c78df00b371b25cc97
        },
        Deletion
      ),
      (
        AccessPath {
          address: c4c63f80c74b11263e421ebf8486a4e398d0dbc09fa7d4f62ccdb309f3aea81f,
          path: 01217da6c6b3e19f18
        },
        cafed00d
      )
    ]
  },
  max_gas_amount: 0,
  gas_unit_price: 0,
  expiration_time: 18446744073709551615 seconds
}
```

LCS representation:
[20000000C3398A599A6F3B9F30B635AF29F2BA046D3A752C26E9D0647B9647D1F4C04AD42000000000000000010000000200000020000000A71D76FAA2D2D5C3224EC3D41DEB293973564A791E55C6782BA76C2BF0495F9A2100000001217DA6C6B3E19F1825CFB2676DAECCE3BF3DE03CF26647C78DF00B371B25CC970000000020000000C4C63F80C74B11263E421EBF8486A4E398D0DBC09FA7D4F62CCDB309F3AEA81F0900000001217DA6C6B3E19F180100000004000000CAFED00D00000000000000000000000000000000FFFFFFFFFFFFFFFF]
