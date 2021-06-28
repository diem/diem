// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

// [Decoupled Execution]
//
//                 Execution
//  Consensus      Phase          Commit Phase
// ┌─────────┐    ┌─────────┐    ┌─────────────┐
// │ Ordered ├───►│ Execute ├───►│ Send Commit │
// │ Blocks  │    │         │    │ Vote    │
// └─────────┘    └─────────┘    └─────────────┘
//                                     ▼
//                               ┌─────────────┐    ┌──────────┐
//                               │ Signature   ├───►│ Commit   │
//                               │ Aggregation │    │ Blocks   │
//                               └─────────────┘    └──────────┘
//                                     ▼
//                               ┌─────────────┐
//                               │ Send Commit │
//                               │ Decision    │ (Asynchronously)
//                               └─────────────┘

pub mod ordering_state_computer;
pub mod execution_phase;
