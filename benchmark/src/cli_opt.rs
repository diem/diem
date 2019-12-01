// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{bail, Result};
use libra_config::config::{NodeConfig, PersistableConfig};
use libra_logger::prelude::*;
use std::{ffi::OsStr, fs, net::IpAddr, path::PathBuf, str::FromStr};
use structopt::{clap::arg_enum, StructOpt};
use walkdir::WalkDir;

arg_enum! {
    #[derive(Debug)]
    pub enum TransactionPattern {
        Ring,
        Pairwise,
    }
}

/// CLI options for creating Benchmarker, shared by binaries that use Benchmarker.
#[derive(Debug, StructOpt)]
pub struct BenchOpt {
    /// Validator address list separated by whitespace: `ip_address:port ip_address:port ...`.
    /// It is required unless (and hence conflict with) swarm-config-dir is present.
    #[structopt(
        short = "a",
        long,
        conflicts_with = "swarm-config-dir",
        required_unless = "swarm-config-dir"
    )]
    pub validator_addresses: Vec<String>,
    /// libra-swarm's config file directory, which holds libra-node's config .toml file(s).
    /// It is requried unless (and hence conflicts with) validator-addresses.
    #[structopt(
        short = "s",
        long,
        conflicts_with = "validator-addresses",
        required_unless = "validator-addresses"
    )]
    pub swarm_config_dir: Option<String>,
    /// Metrics server process's address.
    /// If this argument is not present, RuBen will not spawn metrics server.
    #[structopt(short = "m", long)]
    pub metrics_server_address: Option<String>,
    /// Valid faucet key file path.
    #[structopt(short = "f", long, required = true)]
    pub faucet_key_file_path: String,
    /// Number of AC clients.
    /// If not specified or equals 0, it will be set to #validators * #cpus. Why this value?
    /// We want to create/connect 1 AC client to each AC thread on each validator.
    #[structopt(short = "c", long, default_value = "0")]
    pub num_clients: usize,
    /// Randomly distribute the clients to start sending requests over the stagger_range_ms time.
    /// A value of 1 ms effectively means starting all clients at once.
    #[structopt(short = "g", long, default_value = "64")]
    pub stagger_range_ms: u16,
    /// Submit constant number of requests per second per client; otherwise flood requests.
    #[structopt(short = "k", long)]
    pub submit_rate: Option<u64>,
}

/// CLI options for RuBen.
#[derive(Debug, StructOpt)]
#[structopt(
    name = "ruben",
    about = "RuBen (Ru)ns The Libra (Ben)chmarker For You."
)]
pub struct RubenOpt {
    // Options for creating Benchmarker.
    #[structopt(flatten)]
    pub bench_opt: BenchOpt,
    /// Number of accounts to create in Libra.
    #[structopt(short = "n", long, default_value = "32")]
    pub num_accounts: u64,
    /// Number of repetition to attempt, in one epoch, to increase overall number of sent requests.
    #[structopt(short = "r", long, default_value = "1")]
    pub num_rounds: u64,
    /// Number of epochs to measure the TXN throughput, each time with newly created Benchmarker.
    #[structopt(short = "e", long, default_value = "10")]
    pub num_epochs: u64,
    /// Choices of how to generate TXNs.
    #[structopt(
        short = "t",
        long,
        possible_values(&TransactionPattern::variants()),
        case_insensitive = true,
        default_value = "Ring"
    )]
    pub txn_pattern: TransactionPattern,
}

/// CLI options for linear search max throughput.
#[derive(Debug, StructOpt)]
#[structopt(name = "max-throughput", about = "Linear Search Maximum Throughput.")]
pub struct SearchOpt {
    // Options for creating Benchmarker.
    #[structopt(flatten)]
    pub bench_opt: BenchOpt,
    /// Upper bound value of submission rate for each client.
    #[structopt(short = "u", long, default_value = "8000")]
    pub upper_bound: u64,
    /// Lower bound value of submission rate for each client.
    #[structopt(short = "l", long, default_value = "10")]
    pub lower_bound: u64,
    /// Increase step of submission rate for each client.
    #[structopt(short = "i", long, default_value = "10")]
    pub inc_step: u64,
    /// Number of epochs to send TXNs at a fixed rate.
    #[structopt(short = "e", long, default_value = "4")]
    pub num_epochs: u64,
    /// How many times to repeat the same linear search. Each time with new accounts/TXNs.
    #[structopt(short = "b", long, default_value = "10")]
    pub num_searches: u64,
}

/// Helper that checks if address is valid, and converts unspecified address to localhost.
/// If parsing as numeric network address fails, treat as valid domain name.
fn parse_socket_address(address: &str, port: u16) -> String {
    match IpAddr::from_str(address) {
        Ok(ip_address) => {
            if ip_address.is_unspecified() {
                format!("localhost:{}", port)
            } else {
                format!("{}:{}", address, port)
            }
        }
        Err(_) => format!("{}:{}", address, port),
    }
}

/// Scan *.node.config.toml files under config_dir_name, parse them as node config
/// and return libra-swarm's node addresses info as a vector.
pub fn parse_swarm_config_from_dir(config_dir_name: &str) -> Result<Vec<String>> {
    let mut validator_addresses: Vec<String> = Vec::new();
    let config_dir = PathBuf::from(config_dir_name);
    for entry in WalkDir::new(config_dir)
        .contents_first(true)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|dir_entry| {
            let path = dir_entry.path();
            warn!("checking entry: {:?}", path);
            path.is_file() && path.file_name() == Some(OsStr::new("node.config.toml"))
        })
    {
        let path = entry.path();
        let filename = path.file_name().unwrap();
        debug!("Parsing node config file {:?}.", filename);
        let config_string = fs::read_to_string(&path)
            .unwrap_or_else(|_| panic!("failed to load config file {:?}", filename));
        let config = NodeConfig::parse(&config_string)
            .unwrap_or_else(|_| panic!("failed to parse NodeConfig from {:?}", filename));
        let address = parse_socket_address(
            &config.admission_control.address,
            config.admission_control.admission_control_service_port,
        );
        validator_addresses.push(address);
    }
    if validator_addresses.is_empty() {
        bail!(
            "unable to parse validator_addresses from {}",
            config_dir_name
        )
    }
    Ok(validator_addresses)
}

impl BenchOpt {
    /// Override validator_addresses if swarm_config_dir is provided.
    pub fn try_parse_validator_addresses(&mut self) {
        if let Some(swarm_config_dir) = &self.swarm_config_dir {
            let validator_addresses =
                parse_swarm_config_from_dir(swarm_config_dir).expect("invalid arguments");
            self.validator_addresses = validator_addresses;
        }
    }

    /// Ensure at least one client for one validator.
    pub fn parse_num_clients(&mut self) {
        if self.num_clients == 0 {
            self.num_clients = self.validator_addresses.len() * num_cpus::get();
            info!(
                "Set number of clients in Benchmarker to {}",
                self.num_clients
            );
        }
    }

    /// Using either specified constant rate or flood.
    pub fn parse_submit_rate(&self) -> u64 {
        self.submit_rate.unwrap_or(std::u64::MAX)
    }
}

impl RubenOpt {
    pub fn new_from_args() -> Self {
        let mut args = RubenOpt::from_args();
        args.bench_opt.try_parse_validator_addresses();
        args.bench_opt.parse_num_clients();
        args
    }
}

impl SearchOpt {
    pub fn new_from_args() -> Self {
        let mut args = SearchOpt::from_args();
        args.bench_opt.try_parse_validator_addresses();
        args.bench_opt.parse_num_clients();
        assert!(args.lower_bound > 0);
        assert!(args.inc_step > 0);
        assert!(args.lower_bound < args.upper_bound);
        args
    }
}

#[cfg(test)]
mod tests {
    use crate::cli_opt::{parse_socket_address, parse_swarm_config_from_dir};

    #[test]
    fn test_parse_socket_address() {
        assert_eq!(
            parse_socket_address("216.10.234.56", 12345),
            "216.10.234.56:12345"
        );
        assert_eq!(parse_socket_address("0.0.0.0", 12345), "localhost:12345");
        assert_eq!(parse_socket_address("::", 12345), "localhost:12345");
        assert_eq!(
            parse_socket_address("face:booc::0", 12345),
            "face:booc::0:12345"
        );
        assert_eq!(
            parse_socket_address("2401:dbff:121f:a2f1:face:d:6c:0", 12345),
            "2401:dbff:121f:a2f1:face:d:6c:0:12345"
        );
        assert_eq!(parse_socket_address("localhost", 12345), "localhost:12345");
        assert_eq!(
            parse_socket_address("www.facebook.com", 12345),
            "www.facebook.com:12345"
        );
    }

    #[test]
    fn test_parse_swarm_config_from_invalid_dir() {
        // Directory doesn't exist at all.
        let non_exist_dir_name = String::from("NonExistDir");
        let mut result = parse_swarm_config_from_dir(&non_exist_dir_name);
        assert_eq!(result.is_err(), true);

        // Directory exists but config file does not.
        let path = std::env::current_dir().expect("unable to get current dir");
        if let Some(dir_without_config) = path.to_str() {
            result = parse_swarm_config_from_dir(&dir_without_config);
            assert_eq!(result.is_err(), true);
        }
    }
}
