// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use lcs;
use libra_crypto::{
    traits::{CryptoMaterialError, ValidKeyStringExt},
    x25519,
};
#[cfg(any(test, feature = "fuzzing"))]
use proptest::{collection::vec, prelude::*};
#[cfg(any(test, feature = "fuzzing"))]
use proptest_derive::Arbitrary;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::{
    convert::{Into, TryFrom},
    fmt,
    net::{self, Ipv4Addr, Ipv6Addr},
    num,
    str::FromStr,
    string::ToString,
};
use thiserror::Error;

const MAX_DNS_NAME_SIZE: usize = 255;

/// A `RawNetworkAddress` is the serialized, unverified, on-chain representation
/// of a [`NetworkAddress`]. Specifically, a `RawNetworkAddress` is usually an
/// [`lcs`]-serialized `NetworkAddress`.
///
/// This representation is useful because:
///
/// 1. Move does't understand (and doesn't really need to understand) what a
///    `NetworkAddress` is, so we can just store an opaque `vector<u8>` on-chain,
///    which we represent as `RawNetworkAddress` in Rust.
/// 2. We want to deserialize a `Vec<NetworkAddress>` but ignore ones that don't
///    properly deserialize. For example, a validator might advertise several
///    `NetworkAddress`. If that validator is running a newer version of the node
///    software, and old versions can't understand one of the `NetworkAddress`,
///    they would be able to ignore the one that doesn't deserialize for them.
///    We can easily do this by storing a `vector<vector<u8>>` on-chain, which
///    we deserialize as `Vec<RawNetworkAddress>` in Rust. Then we deserialize
///    each `RawNetworkAddress` into a `NetworkAddress` individually.
///
/// Note: deserializing a `RawNetworkAddress` does no validation, other than
/// deserializing the underlying `Vec<u8>`.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct RawNetworkAddress(#[serde(with = "serde_bytes")] Vec<u8>);

/// Libra `NetworkAddress` is a compact, efficient, self-describing and
/// future-proof network address represented as a stack of protocols. Essentially
/// libp2p's [multiaddr](https://multiformats.io/multiaddr/) but using [`lcs`] to
/// describe the binary format.
///
/// Most validators will advertise a network address like:
///
/// `/dns/example.com/tcp/6180/ln-noise-ik/<x25519-pubkey>/ln-handshake/1`
///
/// Unpacking, the above effectively means:
///
/// 1. Resolve the DNS name "example.com" to an ip address, `addr`.
/// 2. Open a TCP connection to `(addr, 6180)`.
/// 3. Perform a Noise IK handshake and assume the peer's static pubkey is
///    `<x25519-pubkey>`. After this step, we will have a secure, authenticated
///    connection with the peer.
/// 4. Perform a LibraNet version negotiation handshake (version 1).
///
/// One key concept behind `NetworkAddress` is that it is fully self-describing,
/// which allows us to easily "pre-negotiate" protocols while also allowing for
/// future upgrades. For example, it is generally unsafe to negotiate a secure
/// transport in-band. Instead, with `NetworkAddress` we can advertise (via
/// discovery) the specific secure transport protocol and public key that we
/// support (and even advertise multiple incompatible versions). When a peer
/// wishes to establish a connection with us, they already know which secure
/// transport protocol to use; in this sense, the secure transport protocol is
/// "pre-negotiated" by the dialier selecting which advertised protocol to use.
///
/// In addition, `NetworkAddress` is integrated with the LibraNet concept of a
/// [`Transport`], which takes a `NetworkAddress` when dialing and peels off
/// [`Protocols`] to establish a connection and perform initial handshakes.
/// Similarly, the `Transport` takes `NetworkAddress` to listen on, which tells
/// it what protocols to expect on the socket.
///
/// An example of a serialized `NetworkAddress` and `RawNetworkAddress`:
///
/// ```rust
/// // human-readable format:
/// //
/// //   "/ip4/10.0.0.16/tcp/80"
/// //
/// // serialized NetworkAddress:
/// //
/// //      [ 02 00 0a 00 00 10 05 80 00 ]
/// //       \  \  \           \  \
/// //        \  \  \           \  '-- u16 tcp port
/// //         \  \  \           '-- uvarint protocol id for /tcp
/// //          \  \  '-- u32 ipv4 address
/// //           \  '-- uvarint protocol id for /ip4
/// //            '-- uvarint number of protocols
/// //
/// // serialized RawNetworkAddress:
/// //
/// //   [ 09 02 00 0a 00 00 10 05 80 00 ]
/// //    \   \
/// //     \   '-- serialized NetworkAddress
/// //      '-- 9 byte uvarint length prefix
///
/// use libra_network_address::{NetworkAddress, RawNetworkAddress};
/// use lcs;
/// use std::{str::FromStr, convert::TryFrom};
///
/// let addr = NetworkAddress::from_str("/ip4/10.0.0.16/tcp/80").unwrap();
/// let raw_addr = RawNetworkAddress::try_from(&addr).unwrap();
/// let actual_ser_raw_addr = lcs::to_bytes(&raw_addr).unwrap();
/// let actual_raw_addr: Vec<u8> = raw_addr.into();
///
/// let expected_raw_addr: Vec<u8> = [2, 0, 10, 0, 0, 16, 5, 80, 0].to_vec();
/// let expected_ser_raw_addr: Vec<u8> = [09, 2, 0, 10, 0, 0, 16, 5, 80, 0].to_vec();
///
/// assert_eq!(expected_raw_addr, actual_raw_addr);
/// assert_eq!(expected_ser_raw_addr, actual_ser_raw_addr);
/// ```
#[derive(Clone, Eq, PartialEq)]
pub struct NetworkAddress(Vec<Protocol>);

/// A single protocol in the [`NetworkAddress`] protocol stack.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
pub enum Protocol {
    Ip4(Ipv4Addr),
    Ip6(Ipv6Addr),
    Dns(DnsName),
    Dns4(DnsName),
    Dns6(DnsName),
    Tcp(u16),
    Memory(u16),
    // human-readable x25519::PublicKey is lower-case hex encoded
    NoiseIk(x25519::PublicKey),
    // TODO(philiphayes): use actual handshake::MessagingProtocolVersion. we
    // probably need to move network wire into its own crate to avoid circular
    // dependency b/w network and types.
    Handshake(u8),
}

/// A minimally parsed DNS name. We don't really do any checking other than
/// enforcing:
///
/// 1. it is not an empty string
/// 2. it is not larger than 255 bytes
/// 3. it does not contain any forward slash '/' characters
///
/// From the [DNS name syntax RFC](https://tools.ietf.org/html/rfc2181#page-13),
/// the standard rules are:
///
/// 1. the total size <= 255 bytes
/// 2. each label <= 63 bytes
/// 3. any binary string is valid
///
/// So the restrictions we're adding are (1) no '/' characters and (2) the name
/// is a valid unicode string. We do this because '/' characters are already our
/// protocol delimiter and Rust's [`::std::net::ToSocketAddr`] API requires a
/// `&str`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DnsName(String);

/// Possible errors when parsing a human-readable [`NetworkAddress`].
#[derive(Error, Debug)]
pub enum ParseError {
    #[error("unknown protocol type: '{0}'")]
    UnknownProtocolType(String),

    #[error("unexpected end of protocol string")]
    UnexpectedEnd,

    #[error("error parsing ip4/ip6 address: {0}")]
    ParseAddrError(#[from] net::AddrParseError),

    #[error("error parsing integer: {0}")]
    ParseIntError(#[from] num::ParseIntError),

    #[error("error parsing x25519 public key: {0}")]
    ParseX25519PubkeyError(#[from] CryptoMaterialError),

    #[error("network address cannot be empty")]
    EmptyProtocolString,

    #[error("protocol string must start with '/'")]
    InvalidProtocolString,

    #[error("dns name cannot be empty")]
    EmptyDnsNameString,

    #[error("dns name cannot contain '/' characters")]
    InvalidDnsNameCharacter,

    #[error("dns name is too long: len: {0} bytes, max len: 255 bytes")]
    DnsNameTooLong(usize),
}

#[derive(Error, Debug)]
#[error("network address cannot be empty")]
pub struct EmptyError;

///////////////////////
// RawNetworkAddress //
///////////////////////

impl RawNetworkAddress {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

    pub fn empty() -> Self {
        Self(Vec::new())
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl Into<Vec<u8>> for RawNetworkAddress {
    fn into(self) -> Vec<u8> {
        self.0
    }
}

impl AsRef<[u8]> for RawNetworkAddress {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl TryFrom<&NetworkAddress> for RawNetworkAddress {
    type Error = lcs::Error;

    fn try_from(value: &NetworkAddress) -> Result<Self, lcs::Error> {
        let bytes = lcs::to_bytes(value)?;
        Ok(RawNetworkAddress::new(bytes))
    }
}

////////////////////
// NetworkAddress //
////////////////////

impl NetworkAddress {
    fn new(protocols: Vec<Protocol>) -> Self {
        Self(protocols)
    }
}

impl FromStr for NetworkAddress {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(ParseError::EmptyProtocolString);
        }

        let mut protocols = Vec::new();
        let mut parts = s.split('/');

        // the first character must be '/'
        if parts.next() != Some("") {
            return Err(ParseError::InvalidProtocolString);
        }

        while let Some(protocol_type) = parts.next() {
            let arg = parts.next().ok_or(ParseError::UnexpectedEnd)?;
            let protocol = match protocol_type {
                "ip4" => Protocol::Ip4(arg.parse()?),
                "ip6" => Protocol::Ip6(arg.parse()?),
                "dns" => Protocol::Dns(arg.parse()?),
                "dns4" => Protocol::Dns4(arg.parse()?),
                "dns6" => Protocol::Dns6(arg.parse()?),
                "tcp" => Protocol::Tcp(arg.parse()?),
                "memory" => Protocol::Memory(arg.parse()?),
                "ln-noise-ik" => Protocol::NoiseIk(x25519::PublicKey::from_encoded_string(arg)?),
                "ln-handshake" => Protocol::Handshake(arg.parse()?),
                unknown => return Err(ParseError::UnknownProtocolType(unknown.to_string())),
            };
            protocols.push(protocol);
        }

        Ok(NetworkAddress::new(protocols))
    }
}

impl TryFrom<Vec<Protocol>> for NetworkAddress {
    type Error = EmptyError;

    fn try_from(value: Vec<Protocol>) -> Result<Self, Self::Error> {
        if value.is_empty() {
            Err(EmptyError)
        } else {
            Ok(NetworkAddress::new(value))
        }
    }
}

impl TryFrom<&RawNetworkAddress> for NetworkAddress {
    type Error = lcs::Error;

    fn try_from(value: &RawNetworkAddress) -> Result<Self, lcs::Error> {
        let addr: NetworkAddress = lcs::from_bytes(value.as_ref())?;
        Ok(addr)
    }
}

impl fmt::Display for NetworkAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for protocol in self.0.iter() {
            protocol.fmt(f)?;
        }
        Ok(())
    }
}

impl fmt::Debug for NetworkAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl Serialize for NetworkAddress {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[derive(Serialize)]
        #[serde(rename = "NetworkAddress")]
        struct SerializeWrapper<'a>(&'a [Protocol]);

        if serializer.is_human_readable() {
            serializer.serialize_str(&self.to_string())
        } else {
            let val = SerializeWrapper(&self.0.as_ref());
            val.serialize(serializer)
        }
    }
}

impl<'de> Deserialize<'de> for NetworkAddress {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(rename = "NetworkAddress")]
        struct DeserializeWrapper(Vec<Protocol>);

        if deserializer.is_human_readable() {
            let s = <&str>::deserialize(deserializer)?;
            NetworkAddress::from_str(s).map_err(de::Error::custom)
        } else {
            let wrapper = DeserializeWrapper::deserialize(deserializer)?;
            let addr = NetworkAddress::try_from(wrapper.0).map_err(de::Error::custom)?;
            Ok(addr)
        }
    }
}

#[cfg(any(test, feature = "fuzzing"))]
impl Arbitrary for NetworkAddress {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        vec(any::<Protocol>(), 1..10)
            .prop_map(NetworkAddress::new)
            .boxed()
    }
}

//////////////
// Protocol //
//////////////

impl fmt::Display for Protocol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use self::Protocol::*;
        match self {
            Ip4(addr) => write!(f, "/ip4/{}", addr),
            Ip6(addr) => write!(f, "/ip6/{}", addr),
            Dns(domain) => write!(f, "/dns/{}", domain),
            Dns4(domain) => write!(f, "/dns4/{}", domain),
            Dns6(domain) => write!(f, "/dns6/{}", domain),
            Tcp(port) => write!(f, "/tcp/{}", port),
            Memory(port) => write!(f, "/memory/{}", port),
            NoiseIk(pubkey) => write!(
                f,
                "/ln-noise-ik/{}",
                pubkey
                    .to_encoded_string()
                    .expect("ValidKeyStringExt::to_encoded_string is infallible")
            ),
            Handshake(version) => write!(f, "/ln-handshake/{}", version),
        }
    }
}

/////////////
// DnsName //
/////////////

impl DnsName {
    pub fn is_valid(s: &str) -> Result<(), ParseError> {
        if s.is_empty() {
            Err(ParseError::EmptyDnsNameString)
        } else if s.as_bytes().len() > MAX_DNS_NAME_SIZE {
            Err(ParseError::DnsNameTooLong(s.as_bytes().len()))
        } else if s.contains('/') {
            Err(ParseError::InvalidDnsNameCharacter)
        } else {
            Ok(())
        }
    }
}

impl Into<String> for DnsName {
    fn into(self) -> String {
        self.0
    }
}

impl AsRef<str> for DnsName {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl TryFrom<String> for DnsName {
    type Error = ParseError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        DnsName::is_valid(s.as_str()).map(|_| DnsName(s))
    }
}

impl FromStr for DnsName {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        DnsName::is_valid(s).map(|_| DnsName(s.to_owned()))
    }
}

impl fmt::Display for DnsName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Serialize for DnsName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.0.as_str())
    }
}

impl<'de> Deserialize<'de> for DnsName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct StringVisitor;

        impl<'de> de::Visitor<'de> for StringVisitor {
            type Value = DnsName;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "DnsName")
            }
            fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
                DnsName::from_str(v).map_err(de::Error::custom)
            }
            fn visit_string<E: de::Error>(self, v: String) -> Result<Self::Value, E> {
                DnsName::try_from(v).map_err(de::Error::custom)
            }
        }

        deserializer.deserialize_string(StringVisitor)
    }
}

#[cfg(any(test, feature = "fuzzing"))]
impl Arbitrary for DnsName {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        // generate arbitrary unicode strings
        // + without '/'
        // + without control characters (so we can print them easily)
        // + between 1-255 bytes in length
        r"[^/\pC]{1,255}"
            // need this filter b/c the number of unicode characters does not
            // necessarily equal the number of bytes.
            .prop_filter_map("string too long", |s| {
                if s.as_bytes().len() > MAX_DNS_NAME_SIZE {
                    None
                } else {
                    Some(DnsName(s))
                }
            })
            .boxed()
    }
}

///////////
// Tests //
///////////

#[cfg(test)]
mod test {
    use super::*;
    use anyhow::format_err;
    use lcs::test_helpers::assert_canonical_encode_decode;

    #[test]
    fn test_network_address_display() {
        use super::Protocol::*;
        let addr = NetworkAddress::new(vec![Memory(1234), Handshake(0)]);
        assert_eq!("/memory/1234/ln-handshake/0", addr.to_string());
    }

    #[test]
    fn test_network_address_parse_success() {
        use super::Protocol::*;

        let pubkey_str = "080e287879c918794170e258bfaddd75acac5b3e350419044655e4983a487120";
        let pubkey = x25519::PublicKey::from_encoded_string(pubkey_str).unwrap();
        let noise_addr_str = format!(
            "/dns/example.com/tcp/1234/ln-noise-ik/{}/ln-handshake/5",
            pubkey_str
        );

        let test_cases = [
            (
                "/memory/1234/ln-handshake/0",
                vec![Memory(1234), Handshake(0)],
            ),
            (
                "/ip4/12.34.56.78/tcp/1234/ln-handshake/123",
                vec![
                    Ip4(Ipv4Addr::new(12, 34, 56, 78)),
                    Tcp(1234),
                    Handshake(123),
                ],
            ),
            (
                "/ip6/::1/tcp/0",
                vec![Ip6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1)), Tcp(0)],
            ),
            (
                "/ip6/dead:beef::c0de/tcp/8080",
                vec![
                    Ip6(Ipv6Addr::new(0xdead, 0xbeef, 0, 0, 0, 0, 0, 0xc0de)),
                    Tcp(8080),
                ],
            ),
            (
                "/dns/example.com/tcp/80",
                vec![Dns(DnsName("example.com".to_owned())), Tcp(80)],
            ),
            (
                &noise_addr_str,
                vec![
                    Dns(DnsName("example.com".to_owned())),
                    Tcp(1234),
                    NoiseIk(pubkey),
                    Handshake(5),
                ],
            ),
        ];

        for (addr_str, expected_address) in &test_cases {
            let actual_address = NetworkAddress::from_str(addr_str)
                .map_err(|err| format_err!("failed to parse: input: '{}', err: {}", addr_str, err))
                .unwrap();
            let expected_address = NetworkAddress::new(expected_address.clone());
            assert_eq!(actual_address, expected_address);
        }
    }

    #[test]
    fn test_network_address_parse_fail() {
        let test_cases = [
            "",
            "/",
            "/foobar",
            "/tcp",
            "tcp/1234",
            "/tcp/1234/",
            "/tcp/1234/foobar/5",
            "/tcp/99999",
            "/ip4/1.1.1",
            "/ip4/1.1.1.1.",
            "/ip4/1.1.1.1.1",
            "/ip4/1.1.1.999.1",
        ];

        for &addr_str in &test_cases {
            match NetworkAddress::from_str(addr_str) {
                Ok(addr) => panic!(
                    "parsing should fail: input: '{}', output: '{}'",
                    addr_str, addr
                ),
                Err(_) => {}
            }
        }
    }

    proptest! {
        #[test]
        fn test_network_address_canonical_serialization(addr in any::<NetworkAddress>()) {
            assert_canonical_encode_decode(addr);
        }

        #[test]
        fn test_network_address_display_roundtrip(addr in any::<NetworkAddress>()) {
            let addr_str = addr.to_string();
            let addr_parsed = NetworkAddress::from_str(&addr_str).unwrap();
            assert_eq!(addr, addr_parsed);
        }
    }
}
