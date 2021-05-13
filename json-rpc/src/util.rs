// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use once_cell::sync::Lazy;
use regex::Regex;
use std::{fmt, str::FromStr};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SdkLang {
    Rust,
    Java,
    Python,
    Typescript,
    Go,
    CSharp,
    Cpp,
    Unknown,
}

impl Default for SdkLang {
    fn default() -> Self {
        SdkLang::Unknown
    }
}

impl SdkLang {
    pub fn from_str(user_agent_part: &str) -> SdkLang {
        match str::trim(user_agent_part) {
            "diem-client-sdk-rust" => SdkLang::Rust,
            "diem-client-sdk-java" => SdkLang::Java,
            "diem-client-sdk-python" => SdkLang::Python,
            "diem-client-sdk-typescript" => SdkLang::Typescript,
            "diem-client-sdk-golang" => SdkLang::Go,
            "diem-client-sdk-csharp" => SdkLang::CSharp,
            "diem-client-sdk-cpp" => SdkLang::Cpp,
            _ => SdkLang::Unknown,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            SdkLang::Rust => "rust",
            SdkLang::Java => "java",
            SdkLang::Python => "python",
            SdkLang::Typescript => "typescript",
            SdkLang::Go => "golang",
            SdkLang::CSharp => "csharp",
            SdkLang::Cpp => "cpp",
            SdkLang::Unknown => "unknown",
        }
    }
}

static SDK_VERSION_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\b([0-3])\.(\d{1,2})\.(\d{1,2})\b").unwrap());

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SdkVersion {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
}

impl SdkVersion {
    pub fn from_str(user_agent_part: &str) -> SdkVersion {
        if let Some(captures) = SDK_VERSION_REGEX.captures(user_agent_part) {
            if captures.len() == 4 {
                if let (Ok(major), Ok(minor), Ok(patch)) = (
                    u16::from_str(&captures[1]),
                    u16::from_str(&captures[2]),
                    u16::from_str(&captures[3]),
                ) {
                    return SdkVersion {
                        major,
                        minor,
                        patch,
                    };
                }
            }
        }
        SdkVersion::default()
    }
}

impl fmt::Display for SdkVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

#[derive(Clone, Copy, Default, Debug, Eq, PartialEq)]
pub struct SdkInfo {
    pub language: SdkLang,
    pub version: SdkVersion,
}

impl SdkInfo {
    pub fn from_user_agent(user_agent: &str) -> SdkInfo {
        // parse our sdk user agent strings, i.e `diem-client-sdk-python / 0.1.12`
        let lowercase_user_agent = user_agent.to_lowercase();
        let user_agent_parts: Vec<&str> = lowercase_user_agent.split('/').collect();
        if user_agent_parts.len() == 2 {
            let language = SdkLang::from_str(&user_agent_parts[0]);
            let version = SdkVersion::from_str(&user_agent_parts[1]);
            if language != SdkLang::Unknown && version != SdkVersion::default() {
                return SdkInfo { language, version };
            }
        }
        SdkInfo::default()
    }
}

pub fn sdk_info_from_user_agent(user_agent: Option<&str>) -> SdkInfo {
    match user_agent {
        Some(user_agent) => SdkInfo::from_user_agent(user_agent),
        None => SdkInfo::default(),
    }
}
