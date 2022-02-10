---
id: diem-framework
title: Diem Framework
custom_edit_url: https://github.com/diem/diem/edit/main/diem-move/diem-framework/README.md
---

## The Diem Framework

The Diem Framework defines the standard actions that can be performed on-chain
both by the Diem VM---through the various prologue/epilogue functions---and by
users of the blockchain---through the allowed set of transactions. This
directory contains different directories that hold the source Move
modules and transaction scripts, along with a framework for generation of
documentation, ABIs, and error information from the Move source
files. See the [Layout](#layout) section for a more detailed overview of the structure.

## Documentation

Each of the main components of the Diem Framework and contributing guidelines are documented separately. Particularly:
* Documentation for the set of allowed transaction scripts in DPN can be found in [script_documentation.md](DPN/releases/artifacts/current/build/DPNFramework/docs/script_documentation.md).
* The overview documentation for the Move modules can be found in [overview.md](DPN/releases/artifacts/current/build/DPNFramework/docs/overview.md).
* An overview of the approach to formal verification of the framework can be found in [spec_documentation.md](DPN/releases/artifacts/current/build/DPNFramework/docs/spec_documentation.md).
* Contributing guidelines and basic coding standards for the Diem Framework can be found in [CONTRIBUTING.md](CONTRIBUTING.md).

## Compilation and Generation

Recompilation of the Diem Framework and the regeneration of the documents,
ABIs, and error information can be performed by running `cargo run` from this
directory. There are a number of different options available and these are
explained in the help for this command by running `cargo run -- --help` in this
directory. Compilation and generation will be much faster if run in release
mode (`cargo run --release`).

## Layout
The overall structure of the Diem Framework is as follows:

```
├── DPN          # Diem Framework source modules, script modules, and generated documentation
├── core         # Minimal core configuration and Move files needed for a running Diem node
└── experimental # An example usage of the `core` modules and how they can be used to define a chain
```
