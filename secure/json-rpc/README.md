---
id: secure_json_rpc
title: Secure JSON RPC
custom_edit_url: https://github.com/diem/diem/edit/master/secure/json-rpc/README.md
---
# Secure JSON RPC

The secure JSON RPC crate provides a lightweight and minimal JSON RPC client to talk to
the JSON RPC service offered by Diem nodes. This is useful for various security-critical
components (e.g., the key manager), as it allows interaction with the Diem blockchain in a
lightweight manner.

Note: while a JSON RPC client implementation already exists in the Diem codebase, this
aims to provide a simpler implementation with fewer dependencies.

## Overview

For an overview of the APIs offered by the JSON RPC client and server,
see the JSON RPC specification:
[TODO(joshlind): link to the json rpc spec when published!]

## Implementation Details

This crate defines a lightweight JSON RPC client, `JsonRpcClient`, that supports a
subset of the API calls offered by the JSON RPC server.

The crate also offers several simple unit test for the client, as well as fuzzing
support.

## How is this module organized?
```
    |- secure/json-rpc/             # Contains the json rpc client implementation and tests
```
