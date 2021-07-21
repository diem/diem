---
title: "Global storage - structure"
slug: "move-global-storage-structure"
hidden: false
metadata: 
  title: "Global storage - structure"
  description: "Read about the structure of global storage in Move."
createdAt: "2021-02-04T01:03:45.239Z"
updatedAt: "2021-04-21T21:34:49.291Z"
---
The purpose of Move programs is to [read from and write to](doc:move-global-storage-operators) tree-shaped persistent global storage. Programs cannot access the filesystem, network, or any other data outside of this tree.

In pseudocode, the global storage looks something like

```rust
struct GlobalStorage {
  resources: Map<(address, ResourceType), ResourceValue>
  modules: Map<(address, ModuleName), ModuleBytecode>
}
```

Structurally, global storage is a [forest](https://en.wikipedia.org/wiki/Tree_(graph_theory)) consisting of trees rooted at an account [`address`](doc:move-primitives-address). Each address can store both [resource](doc:move-basics-structs-and-resources) data values and [module](doc:move-modules-and-scripts) code values. As the pseudocode above indicates, each `address` can store at most one resource value of a given type and at most one module with a given name.