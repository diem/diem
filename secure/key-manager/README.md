---
id: key_manager
title: Key Manager
custom_edit_url: https://github.com/libra/libra/edit/master/secure/key-manager/README.md
---
# Key Manager

The Key Manager is the primary service responsible for managing and rotating cryptographic
keys used by validator nodes and validator full nodes in the Libra payment network.

## Overview

For a design overview of the key manager, including the component dependencies, modules, data
structures and error types, refer to the key manager specification:
[TODO(joshlind): publish the key manager spec!]

## Implementation Details

This crate defines the key manager implementation. Internally, the crate includes:
 - `KeyManager`: the key manager struct containing the logic for the key manager component.
 - `LibraInterface`: the interface the key manager uses to communicate with the Libra blockchain.
 - `JsonRpcLibraInterface`: the `LibraInterface` implementation using the JSON RPC endpoints.


## How is this module organized?
```
    |- secure/key-manager/             # Contains the key manager implementation and internals (i.e.,
                                             all components identified above).
    |- secure/key-manager/tests.rs     # The unit tests for the key manager.
```
