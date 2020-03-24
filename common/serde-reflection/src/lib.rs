// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

//! # Serde Reflection (experimental)
//!
//! This crate provides a way to extract IDL-like format descriptions for Rust containers
//! that implement the Serialize and/or Deserialize trait(s) of Serde.
//!
//! ```rust
//! # use libra_serde_reflection::*;
//! # use serde::{Deserialize, Serialize};
//! # #[derive(Serialize, PartialEq, Eq, Debug, Clone)]
//! struct Name(String);
//! #
//! # impl<'de> Deserialize<'de> for Name {
//! #     fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
//! #     where
//! #         D: ::serde::Deserializer<'de>,
//! #     {
//! #         // Make sure to wrap our value in a container with the same name
//! #         // as the original type.
//! #         # [derive(Deserialize)]
//! #         # [serde(rename = "Name")]
//! #         struct InternalValue(String);
//! #
//! #         let value = InternalValue::deserialize(deserializer)?.0;
//! #         // Enforce some custom invariant
//! #         if value.len() >= 2 && value.chars().all(char::is_alphabetic) {
//! #             Ok(Name(value))
//! #         } else {
//! #             Err(<D::Error as ::serde::de::Error>::custom(format!(
//! #                 "Invalid name {}",
//! #                 value
//! #             )))
//! #         }
//! #     }
//! # }
//!
//! # #[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
//! enum Person {
//!     NickName(Name),
//!     FullName { first: Name, last: Name },
//! }
//!
//! # fn main() -> Result<(), Error> {
//! // Start a session to trace formats.
//! let mut tracer = Tracer::new(/* is_human_readable */ false);
//!
//! // For every type (here `Name`), if a user-defined implementation of `Deserialize`
//! // exists that performs custom validation checks, start by tracing the serialization
//! // of a valid value of this type.
//! let bob = Name("Bob".into());
//! tracer.trace_value(&bob)?;
//!
//! // Then, we may trace deserialization for top-level types (here `Person`) and their
//! // dependencies as follows. This phase will use Rust values previously recorded
//! // during serialization in order to pass custom checks.
//! // TODO: Currently, recorded values are only used for `NewTypeStruct`s.
//! let format = tracer.trace_type::<Person>()?;
//! assert_eq!(format, Format::TypeName("Person".into()));
//!
//! // We may also request a sample value of type `Person`.
//! let (person, _) = tracer.sample_type_once::<Person>()?;
//! assert_eq!(person, Person::NickName(bob.clone()));
//!
//! // Stop the tracing session and obtain a final registry of containers (i.e. structs and enums).
//! let registry = tracer.registry()?;
//!
//! // We have successfully extracted a format description of all Serde containers under `Person`.
//! assert_eq!(
//!     registry.get("Name").unwrap(),
//!     &ContainerFormat::NewTypeStruct(Box::new(Format::Str)),
//! );
//! match registry.get("Person").unwrap() {
//!     ContainerFormat::Enum(_) => (),
//!      _ => panic!(),
//! };
//!
//! // Export the registry in YAML.
//! let data = serde_yaml::to_string(&registry).unwrap() + "\n";
//! assert_eq!(&data, r#"---
//! Name:
//!   NEWTYPESTRUCT: STR
//! Person:
//!   ENUM:
//!     0:
//!       NickName:
//!         NEWTYPE:
//!           TYPENAME: Name
//!     1:
//!       FullName:
//!         STRUCT:
//!           - first:
//!               TYPENAME: Name
//!           - last:
//!               TYPENAME: Name
//! "#);
//! # Ok(())
//! # }
//! ```

mod de;
mod error;
mod format;
mod ser;
mod trace;
mod value;

pub use error::{Error, Result};
pub use format::{ContainerFormat, Format, Named, VariantFormat};
pub use trace::{Registry, RegistryOwned, Tracer};
pub use value::Value;
