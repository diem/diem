// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use diem_genesis_tool::validator_builder::ValidatorConfig;
use diem_types::{account_address::AccountAddress, on_chain_config::VMPublishingOption};
use move_cli::{
    package::cli as pkgcli,
    sandbox,
    sandbox::utils::{on_disk_state_view::OnDiskStateView, Mode, ModeType},
};
use move_lang::shared::AddressBytes;
use move_package::source_package::layout::SourcePackageLayout;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeMap},
    fs,
    path::{Path, PathBuf},
};

/// Default blockchain configuration
pub const DEFAULT_BLOCKCHAIN: &str = "goodday";

/// Name and directory of starter package for all new shuffle projects.
const HELLOBLOCKCHAIN: &str = "helloblockchain";

pub fn handle(blockchain: String, pathbuf: PathBuf) -> Result<()> {
    let project_path = pathbuf.as_path();
    println!("Creating shuffle project in {}", project_path.display());
    fs::create_dir_all(project_path)?;

    // Shuffle projects aspire to be on a Diem Core Framework that
    // does not include the DPN. ModeType::Diem is not quite there but is a step
    // towards that goal. Bare and Stdlib do not work as expected but this is a WIP
    let mode = Mode::new(ModeType::Diem);
    let build_path = project_path.join(HELLOBLOCKCHAIN).join("build");
    let storage_path = project_path.join(HELLOBLOCKCHAIN).join("storage");
    let state = mode.prepare_state(build_path.as_path(), storage_path.as_path())?;

    let config = Config {
        blockchain,
        named_addresses: fetch_named_addresses(&state)?,
    };
    write_project_config(project_path, &config)?;
    write_move_starter_modules(project_path)?;
    build_move_starter_modules(project_path, &state)?;
    generate_validator_config(project_path)?;
    Ok(())
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    pub(crate) blockchain: String,
    named_addresses: BTreeMap<String, AccountAddress>,
}

// Fetches the named addresses for a particular project or genesis.
// Uses a BTreeMap over BTreeMap because upstream AddressBytes does as well,
// probably because order matters.
fn fetch_named_addresses(state: &OnDiskStateView) -> Result<BTreeMap<String, AccountAddress>> {
    let address_bytes_map = state.get_named_addresses(BTreeMap::new())?;
    Ok(map_address_bytes_to_account_address(address_bytes_map))
}

// map_address_bytes_to_account_address converts BTreeMap<String,AddressBytes>
// to BTreeMap<String, AccountAddress> because AddressBytes is not serializable.
fn map_address_bytes_to_account_address(
    original: BTreeMap<String, AddressBytes>,
) -> BTreeMap<String, AccountAddress> {
    original
        .iter()
        .map(|(name, addr)| (name.to_string(), AccountAddress::new(addr.into_bytes())))
        .collect()
}

fn write_project_config(path: &Path, config: &Config) -> Result<()> {
    let toml_path = PathBuf::from(path).join("Shuffle").with_extension("toml");
    let toml_string = toml::to_string(config)?;
    fs::write(toml_path, toml_string)?;
    Ok(())
}

// Embeds bytes into the binary, keyed off of their file path relative to the
// crate sibling path. ie: shuffle/cli/../../$key
macro_rules! include_files(
    ($($key:expr),+) => {{
        let mut m = BTreeMap::new();
        $(
            m.insert($key, include_bytes!(concat!("../../", $key)).as_ref());
        )+
        m
    }};
);

/// Embeds .move files used to generate the starter template into the binary
/// at compilation time, for reference during project generation.
static EMBEDDED_MOVE_STARTER_FILES: Lazy<BTreeMap<&str, &[u8]>> = Lazy::new(|| {
    include_files! {
        "move/src/modules/SampleModule.move"
    }
});

// Writes all the move modules for a new project, including genesis and
// starter template.
fn write_move_starter_modules(root_path: &Path) -> Result<()> {
    let pkg_dir = root_path.join(HELLOBLOCKCHAIN);
    pkgcli::create_move_package(HELLOBLOCKCHAIN, pkg_dir.as_path())?;
    let sources_path = pkg_dir.join(SourcePackageLayout::Sources.path());
    for key in EMBEDDED_MOVE_STARTER_FILES.keys() {
        let dst = sources_path.join(Path::new(key).file_name().unwrap());
        fs::write(dst.as_path(), EMBEDDED_MOVE_STARTER_FILES[key])?;
    }
    Ok(())
}

// Inspired by https://github.com/diem/diem/blob/e0379458c85d58224798b79194a2871be9a7e655/shuffle/genesis/src/lib.rs#L72
// Reuse publish command from move cli
fn build_move_starter_modules(project_path: &Path, state: &OnDiskStateView) -> Result<()> {
    let src_dir = project_path
        .join(HELLOBLOCKCHAIN)
        .join(SourcePackageLayout::Sources.path());
    let natives =
        move_stdlib::natives::all_natives(AccountAddress::from_hex_literal("0x1").unwrap());
    sandbox::commands::publish(
        natives,
        state,
        &[src_dir.to_string_lossy().to_string()],
        true,
        true,
        None,
        state.get_named_addresses(BTreeMap::new())?,
        true,
    )
}

fn generate_validator_config(project_path: &Path) -> Result<ValidatorConfig> {
    let publishing_option = VMPublishingOption::open();
    // TODO (dimroc): place genesis module deployment/generate validator config
    // in shuffle node command
    shuffle_custom_node::generate_validator_config(
        project_path.join("nodeconfig").as_path(),
        diem_framework_releases::current_module_blobs().to_vec(),
        publishing_option,
    )
}

#[cfg(test)]
mod test {
    use super::*;
    use diem_config::config::NodeConfig;
    use tempfile::tempdir;

    #[test]
    fn test_write_project_config() {
        let dir = tempdir().unwrap();
        let config = Config {
            blockchain: String::from(DEFAULT_BLOCKCHAIN),
            named_addresses: map_address_bytes_to_account_address(
                diem_framework::diem_framework_named_addresses(),
            ),
        };

        write_project_config(dir.path(), &config).unwrap();

        let config_string =
            fs::read_to_string(dir.path().join("Shuffle").with_extension("toml")).unwrap();
        let read_config: Config = toml::from_str(config_string.as_str()).unwrap();
        assert_eq!(config, read_config);
        let actual_std_address = read_config.named_addresses["Std"].short_str_lossless();
        assert_eq!(actual_std_address, "1");
    }

    #[test]
    fn test_handle_e2e() {
        let dir = tempdir().unwrap();
        handle(String::from(DEFAULT_BLOCKCHAIN), PathBuf::from(dir.path())).unwrap();

        // spot check move starter files
        let expected_starter_content =
            String::from_utf8_lossy(include_bytes!("../../move/src/modules/SampleModule.move"));
        let actual_starter_content =
            fs::read_to_string(dir.path().join("helloblockchain/sources/SampleModule.move"))
                .unwrap();
        assert_eq!(expected_starter_content, actual_starter_content);

        // check if we can load generated node.yaml config file
        let _node_config =
            NodeConfig::load(dir.path().join("nodeconfig/0/node.yaml").as_path()).unwrap();
    }
}
