// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use ipnet::IpNet;
use std::{env, net::IpAddr};

/// Represents a possible matching entry for an IP address
#[derive(Clone, Debug)]
enum Ip {
    Address(IpAddr),
    Network(IpNet),
}

/// A wrapper around a list of IP cidr blocks or addresses with a [IpMatcher::contains] method for
/// checking if an IP address is contained within the matcher
#[derive(Clone, Debug, Default)]
struct IpMatcher(Vec<Ip>);

/// A wrapper around a list of domains with a [DomainMatcher::contains] method for checking if a
/// domain is contained within the matcher
#[derive(Clone, Debug, Default)]
struct DomainMatcher(Vec<String>);

/// A configuration for filtering out requests that shouldn't be proxied
#[derive(Clone, Debug, Default)]
struct NoProxy {
    ips: IpMatcher,
    domains: DomainMatcher,
}

pub struct Proxy {
    http_proxy: Option<String>,
    https_proxy: Option<String>,
    no_proxy: Option<NoProxy>,
}

impl Proxy {
    pub fn new() -> Self {
        let http_proxy = env::var("http_proxy")
            .or_else(|_| env::var("HTTP_PROXY"))
            .ok();
        let https_proxy = env::var("https_proxy")
            .or_else(|_| env::var("HTTPS_PROXY"))
            .ok();
        let no_proxy = NoProxy::new();

        Self {
            http_proxy,
            https_proxy,
            no_proxy,
        }
    }

    pub fn http(&self, host: &str) -> Option<&str> {
        if let Some(no_proxy) = &self.no_proxy {
            if no_proxy.contains(host) {
                return None;
            }
        }
        self.http_proxy.as_deref()
    }

    pub fn https(&self, host: &str) -> Option<&str> {
        if let Some(no_proxy) = &self.no_proxy {
            if no_proxy.contains(host) {
                return None;
            }
        }
        self.https_proxy.as_deref()
    }
}

impl NoProxy {
    /// Returns a new no proxy configration if the no_proxy/NO_PROXY environment variable is set.
    /// Returns None otherwise
    fn new() -> Option<Self> {
        let raw = env::var("no_proxy")
            .or_else(|_| env::var("NO_PROXY"))
            .unwrap_or_default();
        if raw.is_empty() {
            return None;
        }
        let mut ips = Vec::new();
        let mut domains = Vec::new();
        let parts = raw.split(',');
        for part in parts {
            match part.parse::<IpNet>() {
                // If we can parse an IP net or address, then use it, otherwise, assume it is a domain
                Ok(ip) => ips.push(Ip::Network(ip)),
                Err(_) => match part.parse::<IpAddr>() {
                    Ok(addr) => ips.push(Ip::Address(addr)),
                    Err(_) => domains.push(part.to_owned()),
                },
            }
        }
        Some(NoProxy {
            ips: IpMatcher(ips),
            domains: DomainMatcher(domains),
        })
    }

    fn contains(&self, host: &str) -> bool {
        // According to RFC3986, raw IPv6 hosts will be wrapped in []. So we need to strip those off
        // the end in order to parse correctly
        let host = if host.starts_with('[') {
            let x: &[_] = &['[', ']'];
            host.trim_matches(x)
        } else {
            host
        };
        match host.parse::<IpAddr>() {
            // If we can parse an IP addr, then use it, otherwise, assume it is a domain
            Ok(ip) => self.ips.contains(ip),
            Err(_) => self.domains.contains(host),
        }
    }
}

impl IpMatcher {
    fn contains(&self, addr: IpAddr) -> bool {
        for ip in self.0.iter() {
            match ip {
                Ip::Address(address) => {
                    if &addr == address {
                        return true;
                    }
                }
                Ip::Network(net) => {
                    if net.contains(&addr) {
                        return true;
                    }
                }
            }
        }
        false
    }
}

impl DomainMatcher {
    fn contains(&self, domain: &str) -> bool {
        for d in self.0.iter() {
            // First check for a "wildcard" domain match. A single "." will match anything.
            // Otherwise, check that the domains are equal
            if (d.starts_with('.') && domain.ends_with(d.get(1..).unwrap_or_default()))
                || d == domain
            {
                return true;
            }
        }
        false
    }
}
