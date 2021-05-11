// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This file implements length and character set limited string types needed
//! for DiemID as defined by DIP-10: https://github.com/diem/dip/blob/main/dips/dip-10.md

#[derive(Clone, Debug, Eq, PartialEq)]
struct DiemId {
    user_identifier: DiemIdUserIdentifier,
    vasp_domain_identifier: DiemIdVaspDomainIdentifier,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiemIdUserIdentifier(Box<str>);

impl DiemIdUserIdentifier {
    pub fn new(s: &str) -> Result<Self, DiemIdParseError> {
        if s.len() > 64 || s.is_empty() {
            return Err(DiemIdParseError);
        }
        let mut chars = s.chars();
        match chars.next() {
            Some('a'..='z') | Some('A'..='Z') | Some('0'..='9') => {}
            Some(_) => return Err(DiemIdParseError),
            None => return Err(DiemIdParseError),
        }
        for c in chars {
            match c {
                'a'..='z' | 'A'..='Z' | '0'..='9' | '.' => {}
                _ => return Err(DiemIdParseError),
            }
        }
        Ok(DiemIdUserIdentifier(s.into()))
    }

    pub fn as_str(&self) -> &str {
        &self.0

        // str::from_utf8(&self.0).expect(
        //     "This can never fail since &self.0 will only ever contain the \
        //      following characters: '0123456789abcdef', which are all valid \
        //      ASCII characters and therefore all valid UTF-8",
        // )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiemIdVaspDomainIdentifier(Box<str>);

impl DiemIdVaspDomainIdentifier {
    pub fn new(s: &str) -> Result<Self, DiemIdParseError> {
        if s.len() > 63 || s.is_empty() {
            return Err(DiemIdParseError);
        }
        let mut chars = s.chars();
        match chars.next() {
            Some('a'..='z') | Some('A'..='Z') | Some('0'..='9') => {}
            Some(_) => return Err(DiemIdParseError),
            None => return Err(DiemIdParseError),
        }
        for c in chars {
            match c {
                'a'..='z' | 'A'..='Z' | '0'..='9' | '.' => {}
                _ => return Err(DiemIdParseError),
            }
        }
        Ok(DiemIdVaspDomainIdentifier(s.into()))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiemIdParseError;
