---
title: "Jellyfish Merkle Tree"
slug: "jellyfish-merkle-tree-paper"
hidden: false
metadata: 
  title: "Jellyfish Merkle Tree"
  description: "Read the technical paper to learn about the Jellyfish Merkle Tree (JMT)."
createdAt: "2021-02-04T01:06:43.882Z"
updatedAt: "2021-04-01T17:04:39.929Z"
---
## Abstract

This paper presents Jellyfish Merkle Tree (JMT), a space-and-computation-efficient sparse Merkle tree optimized for Log-Structured Merge-tree (LSM-tree) based key-value storage, which is designed specially for the Diem Blockchain. JMT was inspired by Patricia Merkle Tree (PMT), a sparse Merkle tree structure that powers the widely known Ethereum network. JMT further makes quite a few optimizations in node key, node types and proof format to find the ideal balance between the complexity of data structure, storage, I/O overhead and computation efficiency, to cater to the needs of the Diem Blockchain. JMT has been implemented in Rust, but it is language-independent such that it could be implemented in other programming languages. Also, the JMT structure presented is of great flexibility in implementation details for fitting various practical use cases.

### Downloads
[block:html]
{
  "html": "<d-publication-link \n  image=\"https://diem-developers-components.netlify.app/images/jellyfish-merkle-tree-pdf.png\"\n  doc-link=\"https://diem-developers-components.netlify.app/papers/jellyfish-merkle-tree/2021-01-14.pdf\"\n  title=\"Jellyfish Merkle Tree Paper\"\n></d-publication-link>"
}
[/block]