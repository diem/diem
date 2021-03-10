---
id: key_manager
title: Key Manager
custom_edit_url: https://github.com/diem/diem/edit/main/secure/key-manager/README.md
---
# Key Manager

The Key Manager is the primary service responsible for managing and rotating cryptographic
keys used by validator nodes and validator full nodes in the Diem payment network.

## Overview

For a design overview of the key manager, including the component dependencies, modules, data
structures and error types, refer to the key manager specification:
[TODO(joshlind): publish the key manager spec!]

## Implementation Details

This crate defines the key manager implementation. Internally, the crate includes:
 - `KeyManager`: the key manager struct containing the logic for the key manager component.
 - `DiemInterface`: the interface the key manager uses to communicate with the Diem blockchain.
 - `JsonRpcDiemInterface`: the `DiemInterface` implementation using the JSON RPC endpoints.


## How is this module organized?
```
    |- secure/key-manager/             # Contains the key manager implementation and internals (i.e.,
                                             all components identified above).
    |- secure/key-manager/tests.rs     # The unit tests for the key manager.
```
