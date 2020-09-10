// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use codespan::Span;
use serde::{Deserialize, Serialize};
use std::{
    cmp::Ordering,
    fmt,
    hash::{Hash, Hasher},
};

//**************************************************************************************************
// Loc
//**************************************************************************************************

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Loc {
    file: &'static str,
    span: Span,
}
impl Loc {
    pub fn new(file: &'static str, span: Span) -> Loc {
        Loc { file, span }
    }

    pub fn file(self) -> &'static str {
        self.file
    }

    pub fn span(self) -> Span {
        self.span
    }
}

impl PartialOrd for Loc {
    fn partial_cmp(&self, other: &Loc) -> Option<Ordering> {
        let file_ord = self.file.partial_cmp(other.file)?;
        if file_ord != Ordering::Equal {
            return Some(file_ord);
        }

        let start_ord = self.span.start().partial_cmp(&other.span.start())?;
        if start_ord != Ordering::Equal {
            return Some(start_ord);
        }

        self.span.end().partial_cmp(&other.span.end())
    }
}

impl Ord for Loc {
    fn cmp(&self, other: &Loc) -> Ordering {
        self.file.cmp(other.file).then_with(|| {
            self.span
                .start()
                .cmp(&other.span.start())
                .then_with(|| self.span.end().cmp(&other.span.end()))
        })
    }
}

//**************************************************************************************************
// Spanned
//**************************************************************************************************

#[derive(Copy, Clone)]
pub struct Spanned<T> {
    pub loc: Loc,
    pub value: T,
}

impl<T> Spanned<T> {
    pub fn new(loc: Loc, value: T) -> Spanned<T> {
        Spanned { loc, value }
    }

    const NO_LOC_FILE: &'static str = "";
    pub fn unsafe_no_loc(value: T) -> Spanned<T> {
        Spanned {
            value,
            loc: Loc::new(Self::NO_LOC_FILE, Span::default()),
        }
    }
}

impl<T: PartialEq> PartialEq for Spanned<T> {
    fn eq(&self, other: &Spanned<T>) -> bool {
        self.value == other.value
    }
}

impl<T: Eq> Eq for Spanned<T> {}

impl<T: Hash> Hash for Spanned<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}

impl<T: PartialOrd> PartialOrd for Spanned<T> {
    fn partial_cmp(&self, other: &Spanned<T>) -> Option<Ordering> {
        self.value.partial_cmp(&other.value)
    }
}

impl<T: Ord> Ord for Spanned<T> {
    fn cmp(&self, other: &Spanned<T>) -> Ordering {
        self.value.cmp(&other.value)
    }
}

impl<T: fmt::Display> fmt::Display for Spanned<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", &self.value)
    }
}

impl<T: fmt::Debug> fmt::Debug for Spanned<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", &self.value)
    }
}

/// Function used to have nearly tuple-like syntax for creating a Spanned
pub const fn sp<T>(loc: Loc, value: T) -> Spanned<T> {
    Spanned { loc, value }
}

/// Macro used to create a tuple-like pattern match for Spanned
#[macro_export]
macro_rules! sp {
    (_, $value:pat) => {
        $crate::location::Spanned { value: $value, .. }
    };
    ($loc:pat, _) => {
        $crate::location::Spanned { loc: $loc, .. }
    };
    ($loc:pat, $value:pat) => {
        $crate::location::Spanned {
            loc: $loc,
            value: $value,
        }
    };
}
