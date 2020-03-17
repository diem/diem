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

#[derive(Debug)]
pub struct Tracer {
    /// Whether to trace the human readable variant of the (De)Serialize traits.
    pub(crate) is_human_readable: bool,

    /// Formats of the named containers discovered so far, while tracing
    /// serialization and/or deserialization.
    pub(crate) registry: BTreeMap<&'static str, Format>,

    /// Value samples recorded during serialization.
    /// This will help passing user-defined checks during deserialization.
    values: BTreeMap<&'static str, Value>,

    /// Enums that have detected to be yet incomplete (i.e. missing variants)
    /// while tracing deserialization.
    pub(crate) incomplete_enums: BTreeSet<String>,
}

impl Tracer {
    /// Start tracing deserialization.
    pub fn new(is_human_readable: bool) -> Self {
        Self {
            is_human_readable,
            registry: BTreeMap::new(),
            values: BTreeMap::new(),
            incomplete_enums: BTreeSet::new(),
        }
    }

    /// Trace the serialization of a particular value.
    /// Nested containers will be added to the global registry, indexed by
    /// their (non-qualified) name.
    pub fn trace_value<T>(&mut self, value: &T) -> Result<(Format, Value)>
    where
        T: ?Sized + Serialize,
    {
        let serializer = Serializer::new(self);
        value.serialize(serializer)
    }

    /// Trace a single deserialization of a particular type.
    /// Nested containers will be added to the global registry, indexed by
    /// their (non-qualified) name.
    pub fn trace_type_once<'de, T>(&mut self) -> Result<Format>
    where
        T: Deserialize<'de>,
    {
        let mut format = Format::Unknown;
        let deserializer = Deserializer::new(&mut *self, &mut format);
        T::deserialize(deserializer)?;
        Ok(format)
    }

    /// Same as trace_type_once for seeded deserialization.
    pub fn trace_type_once_with_seed<'de, T>(&mut self, seed: T) -> Result<Format>
    where
        T: DeserializeSeed<'de>,
    {
        let mut format = Format::Unknown;
        let deserializer = Deserializer::new(&mut *self, &mut format);
        seed.deserialize(deserializer)?;
        Ok(format)
    }

    /// Same as trace_type_once but if `T` is an enum, we repeat the process
    /// until all variants are covered.
    pub fn trace_type<'de, T>(&mut self) -> Result<Format>
    where
        T: Deserialize<'de>,
    {
        loop {
            let format = self.trace_type_once::<T>()?;
            if let Format::TypeName(name) = &format {
                if self.incomplete_enums.contains(name) {
                    self.incomplete_enums.remove(name);
                    continue;
                }
            }
            return Ok(format);
        }
    }

    /// Same as `trace` for seeded deserialization.
    pub fn trace_type_with_seed<'de, T>(&'de mut self, seed: T) -> Result<Format>
    where
        T: DeserializeSeed<'de> + Clone,
    {
        loop {
            let format = self.trace_type_once_with_seed(seed.clone())?;
            if let Format::TypeName(name) = &format {
                if self.incomplete_enums.contains(name) {
                    self.incomplete_enums.remove(name);
                    continue;
                }
            }
            return Ok(format);
        }
    }

    /// Compute a default value for `T`.
    /// Return true if `T` is an enum and more values are needed to cover
    /// all variant cases.
    pub fn sample_type_once<'de, T>(&mut self) -> Result<(T, bool)>
    where
        T: Deserialize<'de>,
    {
        let mut format = Format::Unknown;
        let deserializer = Deserializer::new(&mut *self, &mut format);
        let value = T::deserialize(deserializer)?;
        let has_more = match &format {
            Format::TypeName(name) => {
                if self.incomplete_enums.contains(name) {
                    self.incomplete_enums.remove(name);
                    true
                } else {
                    false
                }
            }
            _ => false,
        };
        Ok((value, has_more))
    }

    /// Same as sample_type_once but if `T` is an enum, we repeat the process
    /// until all variants are covered.
    pub fn sample_type<'de, T>(&mut self) -> Result<Vec<T>>
    where
        T: Deserialize<'de>,
    {
        let mut values = Vec::new();
        loop {
            let (value, has_more) = self.sample_type_once::<T>()?;
            values.push(value);
            if !has_more {
                return Ok(values);
            }
        }
    }

    /// Finish tracing and recover a map of normalized formats.
    pub fn registry(self) -> Result<BTreeMap<&'static str, Format>> {
        let mut registry = self.registry;
        for format in registry.values_mut() {
            format.normalize()?;
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
    pub fn registry_unchecked(self) -> Result<BTreeMap<&'static str, Format>> {
        let mut registry = self.registry;
        for format in registry.values_mut() {
            format.normalize().unwrap_or(());
        }
        Ok(registry)
    }
}

impl Tracer {
    pub(crate) fn record(
        &mut self,
        name: &'static str,
        format: Format,
        value: Value,
    ) -> Result<(Format, Value)> {
        let entry = self.registry.entry(name).or_insert(Format::Unknown);
        entry.merge(format)?;
        self.values.insert(name, value.clone());
        Ok((Format::TypeName(name.into()), value))
    }

    pub(crate) fn record_variant(
        &mut self,
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
        let format = Format::Variant(variants);
        let value = Value::Variant(variant_index, Box::new(variant_value));
        self.record(name, format, value)
    }

    pub(crate) fn get_value(&mut self, name: &'static str) -> Option<&Value> {
        self.values.get(name)
    }
}
