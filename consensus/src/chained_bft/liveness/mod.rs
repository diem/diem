// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub(crate) mod local_pacemaker;
pub(crate) mod pacemaker_timeout_manager;
pub(crate) mod proposal_generator;

#[cfg(test)]
mod local_pacemaker_test;
#[cfg(test)]
mod rotating_proposer_test;

#[cfg(fuzzing)]
pub mod pacemaker;
#[cfg(fuzzing)]
pub mod proposer_election;
#[cfg(fuzzing)]
pub mod rotating_proposer_election;

#[cfg(not(fuzzing))]
pub(crate) mod pacemaker;
#[cfg(not(fuzzing))]
pub(crate) mod proposer_election;
#[cfg(not(fuzzing))]
pub(crate) mod rotating_proposer_election;
