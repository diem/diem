// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{CryptoStorage, Error, GetResponse, KVStorage, PublicKeyResponse, Value};
use libra_crypto::ed25519::{Ed25519PrivateKey, Ed25519PublicKey, Ed25519Signature};
use libra_secure_time::{RealTimeService, TimeService};
use std::{collections::HashMap, sync::RwLock};

pub struct CachedStorage<S> {
    storage: Box<S>,
    cache: RwLock<HashMap<String, Value>>,
    time_service: RealTimeService,
}

impl<S: KVStorage> CachedStorage<S> {
    pub fn new(storage: S) -> CachedStorage<S> {
        CachedStorage {
            storage: Box::new(storage),
            cache: RwLock::new(HashMap::new()),
            time_service: RealTimeService::new(),
        }
    }
}

impl<S: KVStorage> KVStorage for CachedStorage<S> {
    fn available(&self) -> Result<(), Error> {
        self.storage.available()
    }

    fn get(&self, key: &str) -> Result<GetResponse, Error> {
        {
            // only acquires a read lock on the cache
            if let Some(value) = self.cache.read().unwrap().get(key) {
                return match &value {
                    Value::Ed25519PrivateKey(_) => Err(Error::InternalError(
                        "private keys should not be cached".into(),
                    )),
                    Value::Ed25519PublicKey(_) => Err(Error::InternalError(
                        "public keys should not be cached".into(),
                    )),
                    Value::HashValue(value) => Ok(GetResponse::new(
                        Value::HashValue(*value),
                        self.time_service.now(),
                    )),
                    Value::String(value) => Ok(GetResponse::new(
                        Value::String(value.clone()),
                        self.time_service.now(),
                    )),
                    Value::Transaction(value) => Ok(GetResponse::new(
                        Value::Transaction(value.clone()),
                        self.time_service.now(),
                    )),
                    Value::U64(value) => Ok(GetResponse::new(
                        Value::U64(*value),
                        self.time_service.now(),
                    )),
                    Value::Bytes(value) => Ok(GetResponse::new(
                        Value::Bytes(value.clone()),
                        self.time_service.now(),
                    )),
                };
            }
        }

        // will acquire a write lock on the cache
        match self.storage.get(key) {
            Err(err) => Err(err),
            Ok(GetResponse { last_update, value }) => match value {
                // private keys are not cached
                Value::Ed25519PrivateKey(value) => Ok(GetResponse::new(
                    Value::Ed25519PrivateKey(value),
                    last_update,
                )),
                // public keys are not cached
                Value::Ed25519PublicKey(value) => Ok(GetResponse::new(
                    Value::Ed25519PublicKey(value),
                    last_update,
                )),
                Value::HashValue(value) => {
                    self.cache
                        .write()
                        .unwrap()
                        .insert(key.to_string(), Value::HashValue(value));
                    Ok(GetResponse::new(Value::HashValue(value), last_update))
                }
                Value::String(value) => {
                    self.cache
                        .write()
                        .unwrap()
                        .insert(key.to_string(), Value::String(value.clone()));
                    Ok(GetResponse::new(Value::String(value), last_update))
                }
                Value::Transaction(value) => {
                    self.cache
                        .write()
                        .unwrap()
                        .insert(key.to_string(), Value::Transaction(value.clone()));
                    Ok(GetResponse::new(Value::Transaction(value), last_update))
                }
                Value::U64(value) => {
                    self.cache
                        .write()
                        .unwrap()
                        .insert(key.to_string(), Value::U64(value));
                    Ok(GetResponse::new(Value::U64(value), last_update))
                }
                Value::Bytes(value) => {
                    self.cache
                        .write()
                        .unwrap()
                        .insert(key.to_string(), Value::Bytes(value.clone()));
                    Ok(GetResponse::new(Value::Bytes(value), last_update))
                }
            },
        }
    }

    fn set(&mut self, key: &str, value: Value) -> Result<(), Error> {
        match value {
            // private keys are not cached
            Value::Ed25519PrivateKey(value) => {
                self.storage.set(key, Value::Ed25519PrivateKey(value))
            }
            // public keys are not cached
            Value::Ed25519PublicKey(value) => self.storage.set(key, Value::Ed25519PublicKey(value)),
            Value::HashValue(value) => {
                self.storage.set(key, Value::HashValue(value))?;
                self.cache
                    .write()
                    .unwrap()
                    .insert(key.to_string(), Value::HashValue(value));
                Ok(())
            }
            Value::String(value) => {
                self.storage.set(key, Value::String(value.clone()))?;
                self.cache
                    .write()
                    .unwrap()
                    .insert(key.to_string(), Value::String(value));
                Ok(())
            }
            Value::Transaction(value) => {
                self.storage.set(key, Value::Transaction(value.clone()))?;
                self.cache
                    .write()
                    .unwrap()
                    .insert(key.to_string(), Value::Transaction(value));
                Ok(())
            }
            Value::U64(value) => {
                self.storage.set(key, Value::U64(value))?;
                self.cache
                    .write()
                    .unwrap()
                    .insert(key.to_string(), Value::U64(value));
                Ok(())
            }
            Value::Bytes(value) => {
                self.storage.set(key, Value::Bytes(value.clone()))?;
                self.cache
                    .write()
                    .unwrap()
                    .insert(key.to_string(), Value::Bytes(value));
                Ok(())
            }
        }
    }

    #[cfg(any(test, feature = "testing"))]
    fn reset_and_clear(&mut self) -> Result<(), Error> {
        self.storage.reset_and_clear()?;
        self.cache.write().unwrap().clear();
        Ok(())
    }
}

impl<S: CryptoStorage> CryptoStorage for CachedStorage<S> {
    fn create_key(&mut self, name: &str) -> Result<Ed25519PublicKey, Error> {
        self.storage.create_key(name)
    }

    fn export_private_key(&self, name: &str) -> Result<Ed25519PrivateKey, Error> {
        self.storage.export_private_key(name)
    }

    fn import_private_key(&mut self, _name: &str, _key: Ed25519PrivateKey) -> Result<(), Error> {
        self.storage.import_private_key(_name, _key)
    }

    fn export_private_key_for_version(
        &self,
        name: &str,
        version: Ed25519PublicKey,
    ) -> Result<Ed25519PrivateKey, Error> {
        self.storage.export_private_key_for_version(name, version)
    }

    fn get_public_key(&self, name: &str) -> Result<PublicKeyResponse, Error> {
        self.storage.get_public_key(name)
    }

    fn get_public_key_previous_version(&self, name: &str) -> Result<Ed25519PublicKey, Error> {
        self.storage.get_public_key_previous_version(name)
    }

    fn rotate_key(&mut self, name: &str) -> Result<Ed25519PublicKey, Error> {
        self.storage.rotate_key(name)
    }

    fn sign<T: libra_crypto::hash::CryptoHash + serde::Serialize>(
        &mut self,
        name: &str,
        message: &T,
    ) -> Result<Ed25519Signature, Error> {
        self.storage.sign(name, message)
    }

    fn sign_using_version<T: libra_crypto::hash::CryptoHash + serde::Serialize>(
        &mut self,
        name: &str,
        version: Ed25519PublicKey,
        message: &T,
    ) -> Result<Ed25519Signature, Error> {
        self.storage.sign_using_version(name, version, message)
    }
}
