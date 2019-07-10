use clap::arg_enum;
use config::config::NodeConfig;
use failure::prelude::*;
use logger::prelude::*;
use regex::Regex;
use std::{fs, net::IpAddr, path::PathBuf, str::FromStr};
use structopt::StructOpt;

arg_enum! {
    #[derive(Debug)]
    pub enum Executable {
        TestLiveness,
        MeasureThroughput,
    }
}

/// CLI options for RuBen.
#[derive(Debug, StructOpt)]
#[structopt(
    name = "RuBen",
    author = "Libra",
    about = "RuBen (Ru)ns The Libra (Ben)chmarker For You."
)]
pub struct Opt {
    /// Validator address list seperated by whitespace: `ip_address:port ip_address:port ...`.
    /// It is requried unless (and hence conflict with) swarm_config_dir is present.
    #[structopt(
        short = "a",
        long = "validator_addresses",
        conflicts_with = "swarm_config_dir",
        requires = "debug_address",
        required_unless = "swarm_config_dir"
    )]
    pub validator_addresses: Vec<String>,
    /// Debug interface address in the form of ip_address:port.
    /// It is requried unless (and hence conflict with) swarm_config_dir is present.
    #[structopt(
        short = "d",
        long = "debug_address",
        conflicts_with = "swarm_config_dir",
        requires = "validator_addresses",
        required_unless = "swarm_config_dir"
    )]
    pub debug_address: Option<String>,
    /// libra_swarm's config file directory, which holds libra_node's config .toml file(s).
    /// It is requried unless (and hence conflict with)
    /// validator_addresses and debug_address are both present.
    #[structopt(
        short = "s",
        long = "swarm_config_dir",
        raw(conflicts_with_all = r#"&["validator_addresses", "debug_address"]"#),
        raw(required_unless_all = r#"&["validator_addresses", "debug_address"]"#)
    )]
    pub swarm_config_dir: Option<String>,
    /// Valid faucet key file path.
    #[structopt(short = "f", long = "faucet_key_file_path", required = true)]
    pub faucet_key_file_path: String,
    /// Number of accounts to create in Libra.
    #[structopt(short = "n", long = "num_accounts", default_value = "32")]
    pub num_accounts: u64,
    /// Free lunch amount to accounts.
    #[structopt(short = "l", long = "free_lunch", default_value = "1000000")]
    pub free_lunch: u64,
    /// Number of AC clients.
    /// If not specified or equals 0, it will be set to validator_addresses.len().
    #[structopt(short = "c", long = "num_clients", default_value = "0")]
    pub num_clients: usize,
    /// Number of repetition to attempt, in one epoch, to increase overal number of sent TXNs.
    #[structopt(short = "r", long = "num_rounds", default_value = "1")]
    pub num_rounds: u64,
    /// Number of epochs to measure the TXN throughput, each time with newly created Benchmarker.
    #[structopt(short = "e", long = "num_epochs", default_value = "10")]
    pub num_epochs: u64,
    /// Supported application of Benchmarker: `TestLiveness` or `MeasureThroughput`.
    #[structopt(
        short = "x",
        long = "executable",
        raw(possible_values = "&Executable::variants()"),
        case_insensitive = true,
        default_value = "MeasureThroughput"
    )]
    pub executable: Executable,
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
/// and return libra_swarm's configuration info as a tuple:
/// (addresses for all nodes, debug_address)
fn parse_swarm_config_from_dir(config_dir_name: &str) -> Result<(Vec<String>, String)> {
    let mut validator_addresses: Vec<String> = Vec::new();
    let mut debug_address = None;
    let re = Regex::new(r"[[:alnum:]]{64}\.node\.config\.toml").expect("failed to build regex");
    let config_dir = PathBuf::from(config_dir_name);
    if config_dir.is_dir() {
        for entry in fs::read_dir(config_dir).expect("invalid config directory") {
            let path = entry.expect("invalid path under config directory").path();
            if path.is_file() {
                let filename = path
                    .file_name()
                    .expect("failed to convert filename to string")
                    .to_str()
                    .expect("failed to convert filename to string");
                if re.is_match(filename) {
                    debug!("Parsing node config file {:?}.", filename);
                    let config_string = fs::read_to_string(&path)
                        .unwrap_or_else(|_| panic!("failed to load config file {:?}", filename));
                    let config = NodeConfig::parse(&config_string).unwrap_or_else(|_| {
                        panic!("failed to parse NodeConfig from {:?}", filename)
                    });
                    if debug_address.is_none() {
                        debug_address = Some(parse_socket_address(
                            &config.debug_interface.address,
                            config.debug_interface.admission_control_node_debug_port,
                        ));
                    }
                    let address = parse_socket_address(
                        &config.admission_control.address,
                        config.admission_control.admission_control_service_port,
                    );
                    validator_addresses.push(address);
                }
            }
        }
    }
    if validator_addresses.is_empty() {
        bail!(
            "unable to parse validator_addresses from {}",
            config_dir_name
        )
    }
    Ok((
        validator_addresses,
        debug_address
            .ok_or_else(|| format_err!("unable to parse debug_address from {}", config_dir_name))?,
    ))
}

impl Opt {
    pub fn new_from_args() -> Self {
        let mut args = Opt::from_args();
        // Override validator_addresses and debug_address if swarm_config_dir is provided.
        if let Some(swarm_config_dir) = args.swarm_config_dir.as_ref() {
            let (validator_addresses, debug_address) =
                parse_swarm_config_from_dir(swarm_config_dir).expect("invalid arguments");
            args.validator_addresses = validator_addresses;
            args.debug_address = Some(debug_address);
        }
        if args.num_clients == 0 {
            args.num_clients = args.validator_addresses.len();
        }
        args
    }
}

#[cfg(test)]
mod tests {
    use crate::ruben_opt::{parse_socket_address, parse_swarm_config_from_dir};

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
