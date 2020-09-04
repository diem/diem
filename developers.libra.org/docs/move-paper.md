---
id: move-paper
title: Move: A Language With Programmable Resources
---

<!-- hide the table of contents --><style>.toc-headings {display: none !important; visibility: hidden !important;}</style>

_**Note to readers:** This report was published before the Association released White Paper v2.0, which includes a number of key updates to the Libra payment system. Outdated links have been removed, but otherwise, this report has not been modified to incorporate the updates and should be read in that context._

## Abstract

We present _Move_, a safe and flexible programming language for the Libra Blockchain. Move is an executable bytecode language used to implement custom transactions and smart contracts. The key feature of Move is the ability to define custom _resource types_ with semantics inspired by linear logic: a resource can never be copied or implicitly discarded, only moved between program storage locations. These safety guarantees are enforced statically by Move’s type system. Despite these special protections, resources are ordinary program values — they can be stored in data structures, passed as arguments to procedures, and so on. First-class resources are a very general concept that programmers can use not only to implement safe digital assets but also to write correct business logic for wrapping assets and enforcing access control policies. The safety and expressivity of Move have enabled us to implement significant parts of the Libra protocol in Move, including Libra coin, transaction processing, and validator management.

### Downloads

[![Move: A Language With Programmable Resources PDF Download](assets/illustrations/move-language-pdf.png){: .download}](assets/papers/libra-move-a-language-with-programmable-resources/2020-05-26.pdf)

<a href="/papers">Previous versions</a>
