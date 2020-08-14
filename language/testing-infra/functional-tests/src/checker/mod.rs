// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub mod directives;
pub mod matcher;

#[cfg(test)]
mod tests;

pub use crate::checker::{
    directives::Directive,
    matcher::{match_output, Match, MatchError, MatchResult, MatchStatus},
};
