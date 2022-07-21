---
title: "Bool"
id: move-primitives-bool
hidden: false
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

- [`if (bool) { ... } `](/docs/move/move-basic-concepts/move-basics-conditionals) 
- [`while(bool) { .. }`](/docs/move/move-basic-concepts/move-basics-loops) 
- [`assert(bool, u64)`](/docs/move/move-basic-concepts/move-basics-abort-assert)