# Secure Storage

## Overview

Secure Storage encapsulates a design paradigm for the secure storage of sensitive data for the Diem blockchain. The type of data includes information required for safety and identity within the system, such as cryptographic keys and consensus safety rules.

## Properties

* Persistent - data within storage remains intact across reboots and crashes
* Tamper resistant - data within storage cannot be subvertly modified through, for example, attempting to change a value or rollback to a previous value
* Integrity - data within storage remain consistent for their entire life-cycle
* Authenticated - clients can identify the service and operations occur over a trusted link
* Access control - the service must allow for distinct personas to access only a subset of the values, such that, two clients cannot access each other's sensitive information
* Key management - the service supports the ability to generate keys and export either the public or private key and support signing operations without exporting the private key

## Design / ecosystem goals

* Work across cloud providers and on-premise
* Simple interface,ideally a library or REST / JSON-RPC
* Minimal third-party dependencies
* Well-reputed, such that, it is auditiable, open-source, widely trusted

## Interface

Secure Storage supports two interfaces:

* Key/value store (KVStorage) for storing arbitrary data
* Crypto store (CryptoStorage) for secure generation and handling of cryptographic keys

```rust
pub trait KVStorage {
    /// Returns an error if the backend service is not online and available.
    fn available(&self) -> Result<(), Error>;

    /// Retrieves a value from storage and fails if the backend is unavailable or the process has
    /// invalid permissions.
    fn get(&self, key: &str) -> Result<GetResponse, Error>;

    /// Sets a value in storage and fails if the backend is unavailable or the process has
    /// invalid permissions.
    fn set(&mut self, key: &str, value: Value) -> Result<(), Error>;
}

/// A container for a get response that contains relevant metadata and the value stored at the
/// given key.
pub struct GetResponse {
    /// Time since Unix Epoch in seconds.
    pub last_update: u64,
    /// Value stored at the provided key
    pub value: Value,
}

pub enum Value {
    Ed25519PrivateKey(Ed25519PrivateKey),
    Ed25519PublicKey(Ed25519PublicKey),
    HashValue(HashValue),
    String(String),
    Transaction(Transaction),
    U64(u64),
}

pub enum Error {
    EntropyError(String),
    InternalError(String),
    KeyAlreadyExists(String),
    KeyNotSet(String),
    PermissionDenied,
    SerializationError(String),
    UnexpectedValueType,
    KeyVersionNotFound(String),
}
```

```rust
/// CryptoStorage provides an abstraction for secure generation and handling of cryptographic keys.
pub trait CryptoStorage: Send + Sync {
    /// Securely generates a new named Ed25519 private key. The behavior for calling this interface
    /// multiple times with the same name is implementation specific.
    fn create_key(&mut self, name: &str) -> Result<Ed25519PublicKey, Error>;

    /// Returns the Ed25519 private key stored at 'name'.
    fn export_private_key(&self, name: &str) -> Result<Ed25519PrivateKey, Error>;

    /// Returns the Ed25519 private key stored at 'name' and identified by 'version', which is the
    /// corresponding public key. This may fail even if the 'named' key exists but the version is
    /// not present.
    fn export_private_key_for_version(
        &self,
        name: &str,
        version: Ed25519PublicKey,
    ) -> Result<Ed25519PrivateKey, Error>;

    /// Returns the Ed25519 public key stored at 'name'.
    fn get_public_key(&self, name: &str) -> Result<PublicKeyResponse, Error>;

    /// Rotates an Ed25519 private key. Future calls without version to this 'named' key will
    /// return the rotated key instance. The previous key is retained and can be accessed via
    /// the version. At most two versions are expected to be retained.
    fn rotate_key(&mut self, name: &str) -> Result<Ed25519PublicKey, Error>;

    /// Signs the provided message using the 'named' private key.
    fn sign_message(&mut self, name: &str, message: &HashValue) -> Result<Ed25519Signature, Error>;

    /// Signs the provided message using the 'named' and 'versioned' private key. This may fail
    /// even if the 'named' key exists but the version is not present.
    fn sign_message_using_version(
        &mut self,
        name: &str,
        version: Ed25519PublicKey,
        message: &HashValue,
    ) -> Result<Ed25519Signature, Error>;
}

pub struct PublicKeyResponse {
    /// Time since Unix Epoch in seconds.
    pub last_update: u64,
    /// Ed25519PublicKey stored at the provided key
    pub public_key: Ed25519PublicKey,
}
```

## What this is used for

### Values

Name                               | Type    | Internal name
---------------------------------- | ------- | ---------------------
Validator owner account address    | String  | owner_account
Validator operator account key     | Ed25519 | operator
Validator operator account address | String  | operator_account
Consensus key                      | Ed25519 | consensus
Execution key                      | Ed25519 | execution
Validator network key              | Ed25519 | validator_network
Fullnode network key               | Ed25519 | fullnode_network
Validator network encryption key   | String  | validator_network_key
Epoch                              | u64     | epoch
Last voted round                   | u64     | last_voted_round
Preferred round                    | u64     | preferred_round
Waypoint                           | String  | waypoint

### Accessors (Services)

Name          | Description
------------- | ----------------------------------------------------------------------------------------
Initializer   | Initializes the secure storage, both the data and policies
Verifier      | Reads all (non-sensitive) data, displays it and compares to genesis if available
Management    | Reads all (non-sensitive) data, and writes some data to support genesis-building process
KeyManager    | Automatically rotates consensus and networking keys
SafetyRules   | Ensures consensus safety and has the role of signing all consensus messages
ValidatorNode | Runs the Diem Validator services
FullNode      | Runs the Diem Fullnode services

### Accessor capabilites

#### Initializer

Value                              | Capability
---------------------------------- | ----------
Validator owner account address    | Create
Validator operator account key     | Create
Validator operator account address | Create
Consensus key                      | Create
Execution key                      | Create
Validator network key              | Create
Fullnode network key               | Create
Validator network encryption key   | Create
Epoch                              | Create
Last voted round                   | Create
Preferred round                    | Create
Waypoint                           | Create

#### Verifier

Value                              | Capability
---------------------------------- | ----------
Validator owner account address    | Read
Validator operator account key     | Public key
Validator operator account address | Read
Consensus key                      | Public key
Execution key                      | Public key
Validator network key              | Public key
Fullnode network key               | Public key
Validator network encryption key   | Read
Epoch                              | Read
Last voted round                   | Read
Preferred round                    | Read
Waypoint                           | Read

#### Management

Value                              | Capability
---------------------------------- | ----------------
Validator owner account address    | Read, Write
Validator operator account key     | Public key, Sign
Validator operator account address | Read, Write
Consensus key                      | Public key
Validator network key              | Public key
Fullnode network key               | Public key
Validator network encryption key   | Read, Write
Waypoint                           | Read, Write

#### KeyManager

Value                            | Capability
-------------------------------- | ------------------
Validator owner account address  | Read
Validator operator account key   | Sign
Consensus key                    | Public key, Rotate
Validator network key            | Public key, Rotate
Fullnode network key             | Public key, Rotate
Validator network encryption key | Read, Write

#### SafetyRules

Value                           | Capability
------------------------------- | -----------
Validator owner account address | Read
Consensus key                   | Private Key
Execution key                   | Public Key
Epoch                           | Read, Write
Last voted round                | Read, Write
Preferred round                 | Read, Write
Waypoint                        | Read, Write

#### ValidatorNode

Value                            | Capability
-------------------------------- | -----------
Validator owner account address  | Read
Execution key                    | Private key
Validator network key            | Private key
Fullnode network key             | Private key
Validator network encryption key | Read
Waypoint                         | Read

#### FullNode

Value                           | Capability
------------------------------- | -----------
Validator owner account address | Read
Fullnode network key            | Private key
Waypoint                        | Read

**Notes:** If access is not mentioned, then it is not needed.
