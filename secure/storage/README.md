---
id: secure_storage
title: Secure Storage
custom_edit_url: https://github.com/diem/diem/edit/main/secure/storage/README.md
---
# Secure Storage

Secure storage provides a secure, persistent data store for sensitive data in the Diem
blockchain. Examples of sensitive data here include information required for safety and
identity within Diem, such as cryptographic keys and consensus safety rules, as well as
run-time configuration data.

## Overview

For a design overview of secure storage, including the APIs, error types and policies, refer
to the secure storage specification:
[TODO(joshlind): publish the secure storage spec!]

## Implementation Details

This crate defines the secure storage API, made up of two separate Rust traits (interfaces):
- `KVStorage`: The KVStorage trait offers a key-value storage abstraction (e.g., to get
and set key-value pairs).
- `CryptoStorage`: The CryptoStorage trait offers a cryptographic-key based storage
abstraction for Ed25519 keys (e.g., key creation, rotation and signing).

This crate provides four different secure storage implementations, each of which implements
both `KVStorage` and `CryptoStorage`:
- `Github`: The Github secure storage implementation provides a storage backend using a
Github repository.
- `Vault`: The Vault secure storage implementation uses the Vault Storage Engine (an engine
offered by HashiCorp: https://www.vaultproject.io/). The Vault secure storage implementation
is the one primarily used in production environments by nodes in the Diem blockchain.
- `InMemory`: The InMemory secure storage implementation provides a simple in-memory storage
engine. This engine should only be used for testing, as it does not offer any persistence, or
security (i.e., data is simply held in DRAM and may be lost on a crash, or restart).
- `OnDisk`: Similar to InMemory, the OnDisk secure storage implementation provides another
useful testing implementation: an on-disk storage engine, where the storage backend is
implemented using a single file written to local disk. In a similar fashion to the in-memory
storage, on-disk should not be used in production environments as it provides no security
guarantees (e.g., encryption before writing to disk). Moreover, OnDisk storage does not
currently support concurrent data accesses.

In addition, this crate also offers a `Namespaced` wrapper around secure storage
implementations. Using the Namespaced wrapper, different entities can share the
same secure storage instance, under different namespaces, providing an abstraction that
each entity has its own secure storage backend.

## How is this module organized?
```
    secure/storage/
    ├── github             # Contains the secure storage implementation based on Github.
    ├── src                # Contains the definitions for secure storage (e.g., API and error types),
                                as well as lightweight implementations for testing (e.g in-memory and on-disk).
    |── src/tests          # Contains the testsuite for all secure storage implementations.
    ├── vault              # Contains the secure storage implementation based on Vault, including the client
                                add fuzzing helper functions.
```
