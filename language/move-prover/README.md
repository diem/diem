---
id: move-prover
title: Formal verification code
custom_edit_url: https://github.com/diem/diem/edit/main/language/move-prover/README.md
---



## Code under this subtree is experimental. It is out of scope for [the Diem Bug Bounty](https://hackerone.com/diem) until it is no longer marked experimental.

# The Move Prover

The Move Prover supports formal specification and verification of Move code. It can automatically prove
logical properties of Move smart contracts, while providing a user experience similar to a type checker or linter.
It's purpose is to make contracts more *trustworthy*, specifically:

- Protect massive assets managed by the Diem blockchain from smart contract bugs
- Protect against well-resourced adversaries
- Anticipate justified regulator scrutiny and compliance requirements
- Allow domain experts with mathematical background, but not necessarily software engineering background, to
  understand what smart contracts do

For more information, refer to the documentation:

-  [To be done] Move Prover Whitepaper
-  [Move Prover User Guide](./doc/user/prover-guide.md)
-  [Move Specification Language](./doc/user/move-model.md)
-  [Installation](./doc/user/install.md)
-  [Testing](./tests/README.md)
-  [To be done] Move Prover Developer Guide
