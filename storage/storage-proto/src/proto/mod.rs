// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![allow(bare_trait_objects)]

use ::libra_types::proto::types;

pub mod storage {
    tonic::include_proto!("storage");
}
