// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![allow(bare_trait_objects)]

use ::proto_types::types;

pub mod admission_control {
    tonic::include_proto!("admission_control");
}
