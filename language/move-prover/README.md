---
id: move-prover
title: Formal verification code
custom_edit_url: https://github.com/libra/libra/edit/master/language/move-prover/README.md
---



## Code under this subtree is experimental. It is out of scope for [the Libra Bug Bounty](https://hackerone.com/libra) until it is no longer marked experimental.

# The Move Prover

The Move Prover supports formal specification and verification of Move programs. It can automatically prove
logical properties of Move smart contracts, while providing a user experience similar to a type checker or linter.
It's purpose is to make contracts more *trustworthy*, specifically:

- Protect massive assets managed by the Libra blockchain from smart contract bugs
- Protect against well-resourced adversaries
- Anticipate justified regulator scrutiny and compliance requirements
- Allow domain experts with mathematical background, but not necessarily software engineering background, to
  understand what smart contracts do

For more information, refer to the documentation:

-  [Move Prover User Guide](./doc/user/prover-guide.md)
-  [To be done] Move Specification Language
-  [To be done] Move Prover Developer Guide
