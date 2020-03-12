// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module implements a matcher that checks if an evaluation log matches the
//! patterns specified by a list of directives.
//!
//! The directives first get divided into groups, where each group consists of
//! 0 or more negative directives, followed by an optional positive directive.
//!
//!     // Directives:
//!     //     not: 1
//!     //     not: 2
//!     //     check: 3
//!     //     not: 4
//!     //     check: bar
//!     //     not : 6
//!
//!     // Groups:
//!     //     group 1:
//!     //         not: 1
//!     //         not: 2
//!     //         check: 3
//!     //     group 2:
//!     //         not: 4
//!     //         check: 5
//!     //     group 3:
//!     //         not: 6
//!
//! Then in order, we take one group at a time and match it against the evaluation log.
//! Recall that the (stringified) evaluation log is essentially a list of string entries
//! that look like this:
//!
//!     // [1] abc de
//!     // [2] foo 3
//!     // [3] 5 bar 6
//!     // [4] 7
//!
//! For each group, we find the earliest place in the current entry where any of the
//! directives matches.
//!     - If the matched directive is negative, abort and report error.
//!     - If the matched direcrive is positive, move on to the next group and start a
//!       new match right after the last matched location.
//!     - If no match is found, retry the current group with the next entry in the log.
//!
//! Example matches:
//!
//!     // [1] abc de
//!     // [2] foo 3
//!     //         ^
//!     //         check: 3
//!     // [3] 5 bar 6
//!     //       ^^^ ^
//!     //       |   not 6
//!     //       check: bar
//!     // [5] 7
//!
//! Note: the group matching procedure above requires searching for multiple string patterns
//! simultatenously. Right now this is implemented using the Aho-Corasick algorithm, achieving
//! an overall time complexity of O(n), where n is the length of the log + the total length of
//! the string patterns in the directives.
//!
//! In order for the match to succeed, it is required that:
//!     1) All positive directives are matched.
//!     2) No negative directives are matched.
//!     3) All error entries in the log are matched.
//!
//! The example above would fail with a negative match.

use crate::{
    checker::directives::Directive,
    evaluator::{EvaluationLog, EvaluationOutput},
};
use aho_corasick::{AhoCorasick, AhoCorasickBuilder};

/// A group consisting of 0 or more negative directives followed by an optional positive directive.
/// An Aho-Corasick automaton is used for efficient matching.
struct MatcherGroup<D> {
    directives: Vec<(usize, D)>,
    automaton: AhoCorasick,
}

/// A group match consisting of the type of the match (p/n), the id of the matched directive and
/// the start and end locations of the text matched (in bytes).
struct GroupMatch {
    is_positive: bool,
    directive_id: usize,
    start: usize,
    end: usize,
}

impl<D: AsRef<Directive>> MatcherGroup<D> {
    /// Find the earliest place where any directive in the group is matched.
    fn match_earliest(&self, s: &str) -> Option<GroupMatch> {
        self.automaton.earliest_find(s).map(|mat| {
            let pat_id = mat.pattern();
            let directive_id = self.directives[pat_id].0;
            let is_positive = self.directives[pat_id].1.as_ref().is_positive();

            GroupMatch {
                is_positive,
                directive_id,
                start: mat.start(),
                end: mat.end(),
            }
        })
    }
}

/// Divides the directives into matcher groups and builds an Aho-Corasick automaton for each group.
fn build_matcher_groups<I, D>(directives: I) -> Vec<MatcherGroup<D>>
where
    D: AsRef<Directive>,
    I: IntoIterator<Item = D>,
{
    let mut groups = vec![];
    let mut buffer = vec![];

    for (id, d) in directives.into_iter().enumerate() {
        if d.as_ref().is_positive() {
            buffer.push((id, d));
            groups.push(buffer);
            buffer = vec![];
        } else {
            buffer.push((id, d));
        }
    }
    if !buffer.is_empty() {
        groups.push(buffer);
    }

    groups
        .into_iter()
        .map(|directives| {
            let automaton = AhoCorasickBuilder::new()
                .dfa(true)
                .build(directives.iter().map(|(_, d)| d.as_ref().pattern_str()));
            MatcherGroup {
                directives,
                automaton,
            }
        })
        .collect()
}

/// An iterator that steps through all matches produced by the given matcher groups
/// against the (stringified) log.
struct MatchIterator<'a, D, S> {
    text: &'a [S],
    matcher_groups: &'a [MatcherGroup<D>],
    cur_entry_id: usize,
    cur_entry_offset: usize,
    cur_group_id: usize,
}

impl<'a, D, S> MatchIterator<'a, D, S>
where
    D: AsRef<Directive>,
    S: AsRef<str>,
{
    fn new(matcher_groups: &'a [MatcherGroup<D>], text: &'a [S]) -> Self {
        Self {
            text,
            matcher_groups,
            cur_entry_id: 0,
            cur_entry_offset: 0,
            cur_group_id: 0,
        }
    }
}

impl<'a, D, S> Iterator for MatchIterator<'a, D, S>
where
    D: AsRef<Directive>,
    S: AsRef<str>,
{
    type Item = (bool, Match);

    fn next(&mut self) -> Option<Self::Item> {
        if self.cur_entry_id >= self.text.len() || self.cur_group_id >= self.matcher_groups.len() {
            return None;
        }

        let cur_group = &self.matcher_groups[self.cur_group_id];
        while self.cur_entry_id < self.text.len() {
            let cur_entry = &self.text[self.cur_entry_id].as_ref();
            let cur_text_fragment = &cur_entry[self.cur_entry_offset..];

            match cur_group.match_earliest(cur_text_fragment) {
                Some(gm) => {
                    let m = Match {
                        pat_id: gm.directive_id,
                        entry_id: self.cur_entry_id,
                        start: gm.start + self.cur_entry_offset,
                        end: gm.end + self.cur_entry_offset,
                    };
                    self.cur_group_id += 1;
                    self.cur_entry_offset = m.end;
                    if self.cur_entry_offset >= cur_entry.len() {
                        self.cur_entry_id += 1;
                        self.cur_entry_offset = 0;
                    }
                    return Some((gm.is_positive, m));
                }
                None => {
                    self.cur_entry_id += 1;
                    self.cur_entry_offset = 0;
                }
            }
        }
        None
    }
}

/// A single match consisting of the index of the log entry, the start location and the end location (in bytes).
#[derive(Debug)]
pub struct Match {
    pub pat_id: usize,
    pub entry_id: usize,
    pub start: usize,
    pub end: usize,
}

/// A match error.
#[derive(Debug)]
pub enum MatchError {
    NegativeMatch(Match),
    UnmatchedDirectives(Vec<usize>),
    UnmatchedErrors(Vec<usize>),
}

/// The status of a match.
/// Can be either success or failure with errors.
#[derive(Debug)]
pub enum MatchStatus {
    Success,
    Failure(Vec<MatchError>),
}

impl MatchStatus {
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success)
    }

    pub fn is_failure(&self) -> bool {
        matches!(self, Self::Failure(_))
    }
}

/// The result of matching the directives against the evaluation log.
#[derive(Debug)]
pub struct MatchResult {
    pub status: MatchStatus,
    pub text: Vec<String>,
    pub matches: Vec<Match>,
}

impl MatchResult {
    pub fn is_success(&self) -> bool {
        self.status.is_success()
    }

    pub fn is_failure(&self) -> bool {
        self.status.is_failure()
    }
}

/// Matches the directives against the evaluation log.
pub fn match_output<I, D>(log: &EvaluationLog, directives: I) -> MatchResult
where
    D: AsRef<Directive>,
    I: IntoIterator<Item = D>,
{
    // Convert each entry of the evaluation log into a string, which will be later matched against.
    let text: Vec<_> = log
        .outputs
        .iter()
        .map(|output| match output {
            EvaluationOutput::Error(e) => format!("{:?}", e.root_cause()),
            _ => format!("{:?}", output),
        })
        .collect();

    // Split directives into groups and build an Aho-Corasick automaton for each group.
    let groups = build_matcher_groups(directives);

    // Compute the matches.
    let mut matches = vec![];
    let mut it = MatchIterator::new(&groups, &text);
    while let Some((is_positive, m)) = it.next() {
        if !is_positive {
            return MatchResult {
                status: MatchStatus::Failure(vec![MatchError::NegativeMatch(m)]),
                text,
                matches,
            };
        }
        matches.push(m);
    }

    // Check if all positive directives are matched.
    let mut errors = vec![];
    let mut unmatched_directives = groups[it.cur_group_id..]
        .iter()
        .flat_map(|group| {
            group
                .directives
                .iter()
                .filter(|(_, d)| d.as_ref().is_positive())
                .map(|(id, _)| *id)
        })
        .peekable();
    if unmatched_directives.peek().is_some() {
        errors.push(MatchError::UnmatchedDirectives(
            unmatched_directives.collect(),
        ));
    }

    // Check if all error entries are matched.
    let mut is_log_entry_matched: Vec<_> = text.iter().map(|_| false).collect();
    for m in &matches {
        is_log_entry_matched[m.entry_id] = true;
    }
    let mut unmatched_errors = is_log_entry_matched
        .iter()
        .enumerate()
        .filter(|(id, b)| !*b && log.outputs[*id].is_error())
        .map(|(id, _)| id)
        .peekable();
    if unmatched_errors.peek().is_some() {
        errors.push(MatchError::UnmatchedErrors(unmatched_errors.collect()));
    }

    // Return the result.
    MatchResult {
        status: if errors.is_empty() {
            MatchStatus::Success
        } else {
            MatchStatus::Failure(errors)
        },
        text,
        matches,
    }
}
