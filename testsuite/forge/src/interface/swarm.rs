// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{AdminInfo, PublicInfo};

/// Trait used to represent a running network comprised of Validators and FullNodes
pub trait Swarm {
    fn admin_info(&mut self) -> AdminInfo<'_>;
    fn public_info(&mut self) -> PublicInfo<'_>;
}
