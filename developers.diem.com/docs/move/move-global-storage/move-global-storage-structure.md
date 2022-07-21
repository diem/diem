---
title: "Global storage - structure"
id: move-global-storage-structure
hidden: false
---
The purpose of Move programs is to [read from and write to](/docs/move/move-global-storage/move-global-storage-operators) tree-shaped persistent global storage. Programs cannot access the filesystem, network, or any other data outside of this tree.

In pseudocode, the global storage looks something like

```rust
struct GlobalStorage {
  resources: Map<(address, ResourceType), ResourceValue>
  modules: Map<(address, ModuleName), ModuleBytecode>
}
```

Structurally, global storage is a [forest](https://en.wikipedia.org/wiki/Tree_(graph_theory)) consisting of trees rooted at an account [`address`](/docs/move/move-primitive-types/move-primitives-address). Each address can store both [resource](/docs/move/move-basic-concepts/move-basics-structs-and-resources) data values and [module](/docs/move/move-start-here/move-modules-and-scripts) code values. As the pseudocode above indicates, each `address` can store at most one resource value of a given type and at most one module with a given name.