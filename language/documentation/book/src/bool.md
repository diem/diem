# Bool

`bool` is Move's primitive type for boolean `true` and `false` values.

## Literals

Literals for `bool` are either `true` or `false`.

## Operations

### Logical

`bool` supports three logical operations:


| Syntax   | Description | Equivalent Expression |
| -------- | ----------- | --------------------- |
| `&&` | short-circuiting logical and | `p && q` is equivalent to `if (p) q else false` |
| <code>&vert;&vert;</code> | short-circuiting logical or | <code>p &vert;&vert; q</code> is equivalent to `if (p) true else q` |
| `!`  | logical negation | `!p` is equivalent to `if (p) false else true` |

### Control Flow

`bool` values are used in several of Move's control-flow constructs:

- [`if (bool) { ... }`](./conditionals.md)
- [`while (bool) { .. }`](./loops.md)
- [`assert(bool, u64)`](./abort-and-assert.md)

## Ownership

As with the other scalar values built-in to the language, boolean values are implicitly copyable, meaning they can be copied without an explicit instruction such as [`copy`](./variables.md#move-and-copy).
