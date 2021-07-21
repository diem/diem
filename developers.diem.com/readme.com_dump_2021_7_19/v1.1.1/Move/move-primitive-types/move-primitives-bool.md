---
title: "Bool"
slug: "move-primitives-bool"
hidden: false
metadata: 
  title: "Bool"
  description: "Learn more about the primitive type bool in Move."
createdAt: "2021-02-04T01:03:45.286Z"
updatedAt: "2021-06-11T17:25:47.379Z"
---
`bool` is Move's primitive type for boolean `true` and `false` values.

## Literals

Literals for `bool` are either `true` or `false`.

## Operations

### Logical

`bool` supports three logical operations:


| Syntax   | Description | Equivalent Expression |
| -------- | ----------- | --------------------- |
| `&&` | short-circuiting logical and | `p && q` is equivalent to `if (p) q else false` |
| `||` | short-circuiting logical or |`p || q` is equivalent to `if (p) true else q` |
| `!`  | logical negation | `!p` is equivalent to `if (p) false else true` |

### Control Flow

`bool` values are used in several of Move's control-flow constructs:

- [`if (bool) { ... } `](doc:move-basics-conditionals) 
- [`while(bool) { .. }`](doc:move-basics-loops) 
- [`assert(bool, u64)`](doc:move-basics-abort-assert)