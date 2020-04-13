// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    de::Deserializer,
    error::{Error, Result},
    format::*,
    ser::Serializer,
    value::Value,
};
use serde::{de::DeserializeSeed, Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// A map of container formats.
pub type Registry = BTreeMap<&'static str, ContainerFormat>;

/// Similar to `Registry` but compatible with `serde::de::DeserializeOwned`.
pub type RegistryOwned = BTreeMap<String, ContainerFormat>;

/// Structure to drive the tracing of Serde serialization and deserialization.
/// This typically aims at computing a `Registry`.
#[derive(Debug)]
pub struct Tracer {
    /// Whether to trace the human readable variant of the (De)Serialize traits.
    pub(crate) is_human_readable: bool,

    /// Formats of the named containers discovered so far, while tracing
    /// serialization and/or deserialization.
    pub(crate) registry: Registry,

    /// Enums that have detected to be yet incomplete (i.e. missing variants)
    /// while tracing deserialization.
    pub(crate) incomplete_enums: BTreeSet<String>,
}

/// Value samples recorded during serialization.
/// This will help passing user-defined checks during deserialization.
#[derive(Debug, Default)]
pub struct SerializationRecords {
    pub(crate) values: BTreeMap<&'static str, Value>,
}

impl SerializationRecords {
    pub fn new() -> Self {
        Self {
            values: BTreeMap::new(),
        }
    }
}

impl Tracer {
    /// Start tracing deserialization.
    pub fn new(is_human_readable: bool) -> Self {
        Self {
            is_human_readable,
            registry: BTreeMap::new(),
            incomplete_enums: BTreeSet::new(),
        }
    }

    /// Trace the serialization of a particular value.
    /// Nested containers will be added to the tracing registry, indexed by
    /// their (non-qualified) name.
    pub fn trace_value<T>(
        &mut self,
        records: &mut SerializationRecords,
        value: &T,
    ) -> Result<(Format, Value)>
    where
        T: ?Sized + Serialize,
    {
        let serializer = Serializer::new(self, records);
        value.serialize(serializer)
    }

    /// Trace a single deserialization of a particular type.
    /// * Nested containers will be added to the tracing registry, indexed by
    /// their (non-qualified) name.
    /// * As a byproduct of deserialization, we also return a value of type `T`.
    /// * Tracing deserialization of a type may fail if this type or some dependencies
    /// have implemented a custom deserializer that validates data. In this case,
    /// `trace_value` must be called first on the relevant user types to provide valid
    /// examples.
    pub fn trace_type_once<'de, T>(
        &mut self,
        records: &'de SerializationRecords,
    ) -> Result<(Format, T)>
    where
        T: Deserialize<'de>,
    {
        let mut format = Format::Unknown;
        let deserializer = Deserializer::new(self, records, &mut format);
        let value = T::deserialize(deserializer)?;
        Ok((format, value))
    }

    /// Same as `trace_type_once` for seeded deserialization.
    pub fn trace_type_once_with_seed<'de, S>(
        &mut self,
        records: &'de SerializationRecords,
        seed: S,
    ) -> Result<(Format, S::Value)>
    where
        S: DeserializeSeed<'de>,
    {
        let mut format = Format::Unknown;
        let deserializer = Deserializer::new(self, records, &mut format);
        let value = seed.deserialize(deserializer)?;
        Ok((format, value))
    }

    /// Same as `trace_type_once` but if `T` is an enum, we repeat the process
    /// until all variants of `T` are covered.
    /// We accumulate and return all the sampled values at the end.
    pub fn trace_type<'de, T>(
        &mut self,
        records: &'de SerializationRecords,
    ) -> Result<(Format, Vec<T>)>
    where
        T: Deserialize<'de>,
    {
        let mut values = Vec::new();
        loop {
            let (format, value) = self.trace_type_once::<T>(records)?;
            values.push(value);
            if let Format::TypeName(name) = &format {
                if self.incomplete_enums.contains(name) {
                    // Restart the analysis to find more variants of T.
                    self.incomplete_enums.remove(name);
                    continue;
                }
            }
            return Ok((format, values));
        }
    }

    /// Same as `trace_type` for seeded deserialization.
    pub fn trace_type_with_seed<'de, S>(
        &mut self,
        records: &'de SerializationRecords,
        seed: S,
    ) -> Result<(Format, Vec<S::Value>)>
    where
        S: DeserializeSeed<'de> + Clone,
    {
        let mut values = Vec::new();
        loop {
            let (format, value) = self.trace_type_once_with_seed(records, seed.clone())?;
            values.push(value);
            if let Format::TypeName(name) = &format {
                if self.incomplete_enums.contains(name) {
                    // Restart the analysis to find more variants of T.
                    self.incomplete_enums.remove(name);
                    continue;
                }
            }
            return Ok((format, values));
        }
    }

    /// Finish tracing and recover a map of normalized formats.
    /// Returns an error if we detect incompletely traced types.
    /// This may happen in a few of cases:
    /// * We traced serialization of user-provided values but we are still missing the content
    ///   of an option type, the content of a sequence type, the key or the value of a dictionary type.
    /// * We traced deserialization of an enum type but we detect that some enum variants are still missing.
    pub fn registry(self) -> Result<Registry> {
        let mut registry = self.registry;
        for (name, format) in registry.iter_mut() {
            format
                .normalize()
                .map_err(|_| Error::UnknownFormatInContainer(name))?;
        }
        if self.incomplete_enums.is_empty() {
            Ok(registry)
        } else {
            Err(Error::MissingVariants(
                self.incomplete_enums.into_iter().collect(),
            ))
        }
    }

    /// Same as registry but always return a value, even if we detected issues.
    /// This should only be use for debugging.
    pub fn registry_unchecked(self) -> Registry {
        let mut registry = self.registry;
        for format in registry.values_mut() {
            format.normalize().unwrap_or(());
        }
        registry
    }

    pub(crate) fn record_container(
        &mut self,
        records: &mut SerializationRecords,
        name: &'static str,
        format: ContainerFormat,
        value: Value,
    ) -> Result<(Format, Value)> {
        self.registry.entry(name).unify(format)?;
        records.values.insert(name, value.clone());
        Ok((Format::TypeName(name.into()), value))
    }

    pub(crate) fn record_variant(
        &mut self,
        records: &mut SerializationRecords,
        name: &'static str,
        variant_index: u32,
        variant_name: &'static str,
        variant: VariantFormat,
        variant_value: Value,
    ) -> Result<(Format, Value)> {
        let mut variants = BTreeMap::new();
        variants.insert(
            variant_index,
            Named {
                name: variant_name.into(),
                value: variant,
            },
        );
        let format = ContainerFormat::Enum(variants);
        let value = Value::Variant(variant_index, Box::new(variant_value));
        self.record_container(records, name, format, value)
    }

    pub(crate) fn get_recorded_value<'de, 'a>(
        &'a self,
        records: &'de SerializationRecords,
        name: &'static str,
    ) -> Option<(&'a ContainerFormat, &'de Value)> {
        match records.values.get(name) {
            Some(value) => {
                let format = self
                    .registry
                    .get(name)
                    .expect("recorded containers should have a format already");
                Some((format, value))
            }
            None => None,
        }
    }
}
