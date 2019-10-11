---
id: secret-service
title: Secret Service
custom_edit_url: https://github.com/libra/libra/edit/master/crypto/secret_service/README.md
---
# Secret Service

The secret service is a separate process that will manage cryptographic secret keys.

## Overview

**Note**: The secret service is under development, the rest of the code does not use the secret service yet, but will use it in the upcoming version.

The secret service will hold the following secret keys for a validator node:
* account key giving the validator control over the three keys below,
* consensus signing key,
* network discovery signing key,
* network handshake Diffie-Hellman static key.

All of the signing operations will be happening on the side of the secret service, and no signing key will ever leave the secret service process boundary.
We also plan in the future to equip the secret service with the ability to do the network handshake, so that the Diffie-Hellman static key also stays within the boundaries of the process.

Right now the secret service exposes the following APIs:
* generate key: takes in a specification for key generation and returns the keyid which is handle to a newly generated key,
* get public key: returns the public key given the key id,
* sign: given a prehashed message and a keyid returns a signature.
These APIs will evolve possibly allowing for key-rotations, key-backup, key-provisioning, key-drop, etc.

Right now the keys are generated randomly: the seed is driven from OS randomness (EntropyRng), the seedable Rng (ChaChaRng) is instantiated with the seed and the keys are generated using this seedable rng. The procedure for key derivation will be changed to facilitate:
* forward security,
* post-compromise security,
* easy backup,
* strong entropy.

## How is this module organized?
    secret_service/src
    ├── secret_service_server.rs   # Struct SecretServiceServer that holds the map of the generated secret keys and implements API answering the requests
    ├── secret_service_client.rs   # ConsensusKeyManager that represents a client for the secret service, it submits the requests and wraps the responses
    ├── secret_service_node.rs     # Runnable SecretServiceNode that opens connections on the ports specified in the node_config
    ├── crypto_wrappers.rs         # Helper methods for new crypto API located in the crypto directory
    ├── main.rs                    # Runs the secret service in its own process
    ├── unit_tests                 # Tests
    ├── lib.rs
    └── proto/
        └── secret_service.proto    # Rpc definitions of callable functions, the format for request and response messages as well as the error codes
