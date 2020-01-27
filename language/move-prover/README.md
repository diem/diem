---
id: move-prover
title: Formal verification code
custom_edit_url: https://github.com/libra/libra/edit/master/language/move-prover/README.md
---

# Code supporting formal verification for Move programs

## Code under this subtree is experimental. It is out of scope for [the Libra Bug Bounty](https://hackerone.com/libra) until it is no longer marked experimental.

## Debugging tips

During development, we sometimes want to edit the boogie file and run boogie on it explicitly.  You can see the boogie command line by cd'ing to the bytecode-to-boogie subdirectory and typing something like:

cargo run -- --verbose debug ../../stdlib/modules/vector.mvir test_mvir/verify_vector.mvir
