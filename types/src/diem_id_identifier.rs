// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use serde::{de, Serialize};
use std::{
    fmt::{Display, Formatter},
    str::FromStr,
};

/// This file implements length and character set limited string types needed
/// for DiemID as defined by DIP-10: https://github.com/diem/dip/blob/main/dips/dip-10.md

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiemId {
    // A reusable identifier that represents either the source or destination
    // end-user of a payment transaction. It is unique to per user at VASP level
    user_identifier: DiemIdUserIdentifier,
    // A unique string that is mapped to a VASP
    vasp_domain_identifier: DiemIdVaspDomainIdentifier,
}

impl DiemId {
    pub fn new(
        user_identifier: DiemIdUserIdentifier,
        vasp_domain_identifier: DiemIdVaspDomainIdentifier,
    ) -> Self {
        Self {
            user_identifier,
            vasp_domain_identifier,
        }
    }

    fn new_from_raw(
        user_identifier: &str,
        vasp_domain_identifier: &str,
    ) -> Result<Self, DiemIdParseError> {
        Ok(DiemId::new(
            DiemIdUserIdentifier::new(user_identifier)?,
            DiemIdVaspDomainIdentifier::new(vasp_domain_identifier)?,
        ))
    }

    pub fn user_identifier(&self) -> &DiemIdUserIdentifier {
        &self.user_identifier
    }

    pub fn vasp_domain_identifier(&self) -> &DiemIdVaspDomainIdentifier {
        &self.vasp_domain_identifier
    }
}

impl Display for DiemId {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}@{}",
            self.user_identifier.as_str(),
            self.vasp_domain_identifier.as_str()
        )
    }
}

impl FromStr for DiemId {
    type Err = DiemIdParseError;

    fn from_str(s: &str) -> Result<Self, DiemIdParseError> {
        let index = s
            .find('@')
            .ok_or_else(|| DiemIdParseError::new("DiemId does not have @".to_string()))?;
        let split = s.split_at(index);
        DiemId::new_from_raw(split.0, (split.1).trim_start_matches('@'))
    }
}

impl std::error::Error for DiemIdParseError {}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct DiemIdUserIdentifier(Box<str>);

impl DiemIdUserIdentifier {
    pub fn new(s: &str) -> Result<Self, DiemIdParseError> {
        if s.len() > 64 || s.is_empty() {
            return Err(DiemIdParseError::new(
                "Invalid length for user identifier".to_string(),
            ));
        }

        let mut chars = s.chars();
        match chars.next() {
            Some('a'..='z') | Some('A'..='Z') | Some('0'..='9') => {}
            Some(_) => {
                return Err(DiemIdParseError::new(
                    "Invalid user identifier input".to_string(),
                ))
            }
            None => {
                return Err(DiemIdParseError::new(
                    "Invalid user identifier input".to_string(),
                ))
            }
        }
        for c in chars {
            match c {
                'a'..='z' | 'A'..='Z' | '0'..='9' | '.' => {}
                _ => return Err(DiemIdParseError::new(format!("Invalid character {}", c))),
            }
        }
        Ok(DiemIdUserIdentifier(s.into()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> de::Deserialize<'de> for DiemIdUserIdentifier {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        use serde::de::Error;

        let s = <String>::deserialize(deserializer)?;
        DiemIdUserIdentifier::new(&s).map_err(D::Error::custom)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct DiemIdVaspDomainIdentifier(Box<str>);

impl DiemIdVaspDomainIdentifier {
    pub fn new(s: &str) -> Result<Self, DiemIdParseError> {
        if s.len() > 63 || s.is_empty() {
            return Err(DiemIdParseError::new(
                "Invalid length for vasp domain identifier".to_string(),
            ));
        }

        let mut chars = s.chars();
        match chars.next() {
            Some('a'..='z') | Some('A'..='Z') | Some('0'..='9') => {}
            Some(_) => {
                return Err(DiemIdParseError::new(
                    "Invalid vasp domain input".to_string(),
                ))
            }
            None => {
                return Err(DiemIdParseError::new(
                    "Invalid vasp domain input".to_string(),
                ))
            }
        }
        for c in chars {
            match c {
                'a'..='z' | 'A'..='Z' | '0'..='9' | '.' => {}
                _ => return Err(DiemIdParseError::new(format!("Invalid character {}", c))),
            }
        }
        Ok(DiemIdVaspDomainIdentifier(s.into()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> de::Deserialize<'de> for DiemIdVaspDomainIdentifier {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        use serde::de::Error;

        let s = <String>::deserialize(deserializer)?;
        DiemIdVaspDomainIdentifier::new(&s).map_err(D::Error::custom)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiemIdParseError {
    message: String,
}

impl DiemIdParseError {
    fn new(message: String) -> DiemIdParseError {
        DiemIdParseError { message }
    }
}

impl Display for DiemIdParseError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "Unable to parse DiemId: {}", self.message)
    }
}

#[test]
fn test_invalid_user_identifier() {
    // Test valid domain
    let raw_identifier = "abcd1234";
    let identifier = DiemIdUserIdentifier::new(&raw_identifier);
    assert!(identifier.is_ok());

    // Test having 64 characters is valid
    let raw_identifier = "aaaaaaaaaabbbbbbbbbbccccccccccddddddddddeeeeeeeeeeffffffffff1234";
    let identifier = DiemIdUserIdentifier::new(&raw_identifier);
    assert!(identifier.is_ok());

    // Test using "-" character is invalid
    let raw_identifier = "abcd!!!1234";
    let identifier = DiemIdUserIdentifier::new(&raw_identifier);
    assert_eq!(
        identifier.unwrap_err(),
        DiemIdParseError {
            message: "Invalid character !".to_string()
        },
    );
    // Test having 64 characters is invalid
    let raw_identifier = "aaaaaaaaaabbbbbbbbbbccccccccccddddddddddeeeeeeeeeeffffffffff12345";
    let identifier = DiemIdUserIdentifier::new(&raw_identifier);
    assert_eq!(
        identifier.unwrap_err(),
        DiemIdParseError {
            message: "Invalid length for user identifier".to_string()
        },
    );
}

#[test]
fn test_invalid_vasp_domain_identifier() {
    // Test valid domain
    let raw_identifier = "diem";
    let identifier = DiemIdVaspDomainIdentifier::new(&raw_identifier);
    assert!(identifier.is_ok());

    // Test having 63 characters is valid
    let raw_identifier = "aaaaaaaaaabbbbbbbbbbccccccccccddddddddddeeeeeeeeeeffffffffff123";
    let identifier = DiemIdVaspDomainIdentifier::new(&raw_identifier);
    assert!(identifier.is_ok());

    // Test using "-" character is invalid
    let raw_identifier = "diem-domain";
    let identifier = DiemIdVaspDomainIdentifier::new(&raw_identifier);
    assert_eq!(
        identifier.unwrap_err(),
        DiemIdParseError {
            message: "Invalid character -".to_string()
        },
    );
    // Test having 64 characters is invalid
    let raw_identifier = "aaaaaaaaaabbbbbbbbbbccccccccccddddddddddeeeeeeeeeeffffffffffgggg";
    let identifier = DiemIdVaspDomainIdentifier::new(&raw_identifier);
    assert_eq!(
        identifier.unwrap_err(),
        DiemIdParseError {
            message: "Invalid length for vasp domain identifier".to_string()
        },
    );
}

#[test]
fn test_get_diem_id_string_from_components() {
    let user_identifier = DiemIdUserIdentifier::new(&"username").unwrap();
    let vasp_domain_identifier = DiemIdVaspDomainIdentifier::new(&"diem").unwrap();
    let diem_id = DiemId::new(user_identifier, vasp_domain_identifier);
    let full_id = diem_id.to_string();
    assert_eq!(full_id, "username@diem");
}

#[test]
fn test_get_diem_id_from_identifier_string() {
    let diem_id_str = "username@diem";
    let diem_id = DiemId::from_str(diem_id_str).unwrap();
    assert_eq!(diem_id.user_identifier().as_str(), "username");
    assert_eq!(diem_id.vasp_domain_identifier().as_str(), "diem");

    let diem_id_str = "username@diem.com";
    let diem_id = DiemId::from_str(diem_id_str).unwrap();
    assert_eq!(diem_id.user_identifier().as_str(), "username");
    assert_eq!(diem_id.vasp_domain_identifier().as_str(), "diem.com");

    let diem_id_str = "username@diem@com";
    let diem_id = DiemId::from_str(diem_id_str);
    assert_eq!(
        diem_id.unwrap_err(),
        DiemIdParseError {
            message: "Invalid character @".to_string()
        },
    );
}
