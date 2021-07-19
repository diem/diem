// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use compatibility_test::SimpleValidatorUpgrade;
use forge::{forge_main, ForgeConfig, InitialVersion, LocalFactory, Options, Result};
use std::num::NonZeroUsize;

fn main() -> Result<()> {
    ::diem_logger::Logger::init_for_testing();

    let tests = ForgeConfig::default()
        .with_initial_validator_count(NonZeroUsize::new(4).unwrap())
        .with_initial_version(InitialVersion::Oldest)
        .with_network_tests(&[&SimpleValidatorUpgrade]);

    let options = Options::from_args();
    forge_main(
        tests,
        LocalFactory::with_upstream_merge_base_and_workspace()?,
        &options,
    )
}
