// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::path::Path;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum SourcePackageLayout {
    Sources,
    Specifications,
    Tests,
    Scripts,
    Examples,
    Manifest,
}

impl SourcePackageLayout {
    /// A Move source package is laid out on-disk as
    /// a_move_package
    /// ├── Move.toml      (required)
    /// ├── sources        (required)
    /// ├── examples       (optional, dev mode)
    /// ├── scripts        (optional)
    /// ├── specifications (optional)
    /// └── tests          (optional, test mode)
    pub fn path(&self) -> &Path {
        Path::new(self.location_str())
    }

    pub fn location_str(&self) -> &'static str {
        match self {
            Self::Sources => "sources",
            Self::Manifest => "Move.toml",
            Self::Tests => "tests",
            Self::Scripts => "scripts",
            Self::Examples => "examples",
            Self::Specifications => "specifications",
        }
    }

    pub fn is_optional(&self) -> bool {
        match self {
            Self::Sources | Self::Manifest => false,
            Self::Tests | Self::Scripts | Self::Examples | Self::Specifications => true,
        }
    }
}
