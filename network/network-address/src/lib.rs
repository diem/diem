// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_crypto::{
    traits::{CryptoMaterialError, ValidCryptoMaterialStringExt},
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
    iter::IntoIterator,
    net::{self, IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
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
/// //        \  \  \           \  \
/// //         \  \  \           \  '-- u16 tcp port
/// //          \  \  \           '-- uvarint protocol id for /tcp
/// //           \  \  '-- u32 ipv4 address
/// //            \  '-- uvarint protocol id for /ip4
/// //             '-- uvarint number of protocols
/// //
/// // serialized RawNetworkAddress:
/// //
/// //   [ 09 02 00 0a 00 00 10 05 80 00 ]
/// //     \  \
/// //      \  '-- serialized NetworkAddress
/// //       '-- uvarint length prefix
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
/// let expected_ser_raw_addr: Vec<u8> = [9, 2, 0, 10, 0, 0, 16, 5, 80, 0].to_vec();
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
    NoiseIK(x25519::PublicKey),
    // TODO(philiphayes): use actual handshake::MessagingProtocolVersion. we
    // probably need to move network wire into its own crate to avoid circular
    // dependency b/w network and types.
    Handshake(u8),
    // TODO(philiphayes): gate behind cfg(test), should not even parse in prod.
    // testing only, unauthenticated peer id exchange
    PeerIdExchange,
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
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
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

#[cfg(any(test, feature = "fuzzing"))]
impl Arbitrary for RawNetworkAddress {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        any::<NetworkAddress>()
            .prop_map(|addr| RawNetworkAddress::try_from(&addr).unwrap())
            .boxed()
    }
}

////////////////////
// NetworkAddress //
////////////////////

impl NetworkAddress {
    fn new(protocols: Vec<Protocol>) -> Self {
        Self(protocols)
    }

    // TODO(philiphayes): could return NonZeroUsize?
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn as_slice(&self) -> &[Protocol] {
        self.0.as_slice()
    }

    pub fn push(mut self, proto: Protocol) -> Self {
        self.0.push(proto);
        self
    }

    pub fn extend_from_slice(mut self, protos: &[Protocol]) -> Self {
        self.0.extend_from_slice(protos);
        self
    }

    /// Given a base `NetworkAddress`, append production protocols and
    /// return the modified `NetworkAddress`.
    ///
    /// ### Example
    ///
    /// ```rust
    /// use libra_crypto::{traits::ValidCryptoMaterialStringExt, x25519};
    /// use libra_network_address::NetworkAddress;
    /// use std::str::FromStr;
    ///
    /// let pubkey_str = "080e287879c918794170e258bfaddd75acac5b3e350419044655e4983a487120";
    /// let pubkey = x25519::PublicKey::from_encoded_string(pubkey_str).unwrap();
    /// let addr = NetworkAddress::from_str("/dns/example.com/tcp/6180").unwrap();
    /// let addr = addr.append_prod_protos(pubkey, 0);
    /// assert_eq!(
    ///     addr.to_string(),
    ///     "/dns/example.com/tcp/6180/ln-noise-ik/080e287879c918794170e258bfaddd75acac5b3e350419044655e4983a487120/ln-handshake/0",
    /// );
    /// ```
    // TODO(philiphayes): use handshake version enum
    pub fn append_prod_protos(
        self,
        network_pubkey: x25519::PublicKey,
        handshake_version: u8,
    ) -> Self {
        self.push(Protocol::NoiseIK(network_pubkey))
            .push(Protocol::Handshake(handshake_version))
    }

    /// Given a base `NetworkAddress`, append production protocols and
    /// return the modified `NetworkAddress`.
    ///
    /// ### Example
    ///
    /// ```rust
    /// use libra_network_address::NetworkAddress;
    /// use std::str::FromStr;
    ///
    /// let addr = NetworkAddress::from_str("/memory/123").unwrap();
    /// let addr = addr.append_test_protos(0);
    /// assert_eq!(addr.to_string(), "/memory/123/ln-peerid-ex/ln-handshake/0");
    /// ```
    // TODO(philiphayes): gate with cfg(test)
    // TODO(philiphayes): use handshake version enum
    pub fn append_test_protos(self, handshake_version: u8) -> Self {
        self.push(Protocol::PeerIdExchange)
            .push(Protocol::Handshake(handshake_version))
    }

    /// Check that a `NetworkAddress` looks like a typical LibraNet address with
    /// associated protocols.
    ///
    /// "typical" LibraNet addresses begin with a transport protocol:
    ///
    /// `"/ip4/<addr>/tcp/<port>"` or
    /// `"/ip6/<addr>/tcp/<port>"` or
    /// `"/dns4/<domain>/tcp/<port>"` or
    /// `"/dns6/<domain>/tcp/<port>"` or
    /// `"/dns/<domain>/tcp/<port>"` or
    /// cfg!(test) `"/memory/<port>"`
    ///
    /// followed by transport upgrade handshake protocols:
    ///
    /// `"/ln-noise-ik/<pubkey>/ln-handshake/<version>"` or
    /// cfg!(test) `"/ln-peerid-ex/ln-handshake/<version>"`
    ///
    /// ### Example
    ///
    /// ```rust
    /// use libra_network_address::NetworkAddress;
    /// use std::str::FromStr;
    ///
    /// let addr_str = "/ip4/1.2.3.4/tcp/6180/ln-noise-ik/080e287879c918794170e258bfaddd75acac5b3e350419044655e4983a487120/ln-handshake/0";
    /// let addr = NetworkAddress::from_str(addr_str).unwrap();
    /// assert!(addr.is_libranet_addr());
    /// ```
    pub fn is_libranet_addr(&self) -> bool {
        parse_libranet_protos(self.as_slice()).is_some()
    }

    /// A temporary, hacky function to parse out the first `/ln-noise-ik/<pubkey>` from
    /// a `NetworkAddress`. We can remove this soon, when we move to the interim
    /// "monolithic" transport model.
    pub fn find_noise_proto(&self) -> Option<&x25519::PublicKey> {
        self.0.iter().find_map(|proto| match proto {
            Protocol::NoiseIK(pubkey) => Some(pubkey),
            _ => None,
        })
    }

    #[cfg(any(test, feature = "fuzzing"))]
    pub fn mock() -> Self {
        NetworkAddress::new(vec![Protocol::Memory(1234)])
    }
}

impl IntoIterator for NetworkAddress {
    type Item = Protocol;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl FromStr for NetworkAddress {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(ParseError::EmptyProtocolString);
        }

        let mut protocols = Vec::new();
        let mut parts_iter = s.split('/');

        // the first character must be '/'
        if parts_iter.next() != Some("") {
            return Err(ParseError::InvalidProtocolString);
        }

        // parse all `Protocol`s
        while let Some(protocol_type) = parts_iter.next() {
            protocols.push(Protocol::parse(protocol_type, &mut parts_iter)?);
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

impl From<Protocol> for NetworkAddress {
    fn from(proto: Protocol) -> NetworkAddress {
        NetworkAddress::new(vec![proto])
    }
}

impl TryFrom<&RawNetworkAddress> for NetworkAddress {
    type Error = lcs::Error;

    fn try_from(value: &RawNetworkAddress) -> Result<Self, lcs::Error> {
        let addr: NetworkAddress = lcs::from_bytes(value.as_ref())?;
        Ok(addr)
    }
}

impl From<SocketAddr> for NetworkAddress {
    fn from(sockaddr: SocketAddr) -> NetworkAddress {
        let ip_proto = Protocol::from(sockaddr.ip());
        let tcp_proto = Protocol::Tcp(sockaddr.port());
        NetworkAddress::new(vec![ip_proto, tcp_proto])
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
            let s = <String>::deserialize(deserializer)?;
            NetworkAddress::from_str(s.as_str()).map_err(de::Error::custom)
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
            NoiseIK(pubkey) => write!(
                f,
                "/ln-noise-ik/{}",
                pubkey
                    .to_encoded_string()
                    .expect("ValidCryptoMaterialStringExt::to_encoded_string is infallible")
            ),
            Handshake(version) => write!(f, "/ln-handshake/{}", version),
            PeerIdExchange => write!(f, "/ln-peerid-ex"),
        }
    }
}

fn parse_one<'a, T>(args: &mut impl Iterator<Item = &'a str>) -> Result<T, ParseError>
where
    T: FromStr,
    T::Err: Into<ParseError>,
{
    let next_arg = args.next().ok_or(ParseError::UnexpectedEnd)?;
    next_arg.parse().map_err(Into::into)
}

impl Protocol {
    fn parse<'a>(
        protocol_type: &str,
        args: &mut impl Iterator<Item = &'a str>,
    ) -> Result<Protocol, ParseError> {
        let protocol = match protocol_type {
            "ip4" => Protocol::Ip4(parse_one(args)?),
            "ip6" => Protocol::Ip6(parse_one(args)?),
            "dns" => Protocol::Dns(parse_one(args)?),
            "dns4" => Protocol::Dns4(parse_one(args)?),
            "dns6" => Protocol::Dns6(parse_one(args)?),
            "tcp" => Protocol::Tcp(parse_one(args)?),
            "memory" => Protocol::Memory(parse_one(args)?),
            "ln-noise-ik" => Protocol::NoiseIK(x25519::PublicKey::from_encoded_string(
                args.next().ok_or(ParseError::UnexpectedEnd)?,
            )?),
            "ln-handshake" => Protocol::Handshake(parse_one(args)?),
            "ln-peerid-ex" => Protocol::PeerIdExchange,
            unknown => return Err(ParseError::UnknownProtocolType(unknown.to_string())),
        };
        Ok(protocol)
    }
}

impl From<IpAddr> for Protocol {
    fn from(addr: IpAddr) -> Protocol {
        match addr {
            IpAddr::V4(addr) => Protocol::Ip4(addr),
            IpAddr::V6(addr) => Protocol::Ip6(addr),
        }
    }
}

/////////////
// DnsName //
/////////////

impl DnsName {
    fn validate(s: &str) -> Result<(), ParseError> {
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

    pub fn as_str(&self) -> &str {
        self.0.as_str()
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
        DnsName::validate(s.as_str()).map(|_| DnsName(s))
    }
}

impl FromStr for DnsName {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        DnsName::validate(s).map(|_| DnsName(s.to_owned()))
    }
}

impl fmt::Display for DnsName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<'de> Deserialize<'de> for DnsName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(rename = "DnsName")]
        struct DeserializeWrapper(String);

        let wrapper = DeserializeWrapper::deserialize(deserializer)?;
        let name = DnsName::try_from(wrapper.0).map_err(de::Error::custom)?;
        Ok(name)
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

/////////////
// Parsing //
/////////////

/// parse the `&[Protocol]` into the `"/memory/<port>"` prefix and unparsed
/// `&[Protocol]` suffix.
pub fn parse_memory(protos: &[Protocol]) -> Option<(u16, &[Protocol])> {
    protos
        .split_first()
        .and_then(|(first, suffix)| match first {
            Protocol::Memory(port) => Some((*port, suffix)),
            _ => None,
        })
}

/// parse the `&[Protocol]` into the `"/ip4/<addr>/tcp/<port>"` or
/// `"/ip6/<addr>/tcp/<port>"` prefix and unparsed `&[Protocol]` suffix.
pub fn parse_ip_tcp(protos: &[Protocol]) -> Option<((IpAddr, u16), &[Protocol])> {
    use Protocol::*;

    if protos.len() < 2 {
        return None;
    }

    let (prefix, suffix) = protos.split_at(2);
    match prefix {
        [Ip4(ip), Tcp(port)] => Some(((IpAddr::V4(*ip), *port), suffix)),
        [Ip6(ip), Tcp(port)] => Some(((IpAddr::V6(*ip), *port), suffix)),
        _ => None,
    }
}

/// parse the `&[Protocol]` into the `"/dns/<domain>/tcp/<port>"`,
/// `"/dns4/<domain>/tcp/<port>"`, or `"/dns6/<domain>/tcp/<port>"` prefix and
/// unparsed `&[Protocol]` suffix.
pub fn parse_dns_tcp(protos: &[Protocol]) -> Option<((&DnsName, u16), &[Protocol])> {
    use Protocol::*;

    if protos.len() < 2 {
        return None;
    }

    let (prefix, suffix) = protos.split_at(2);
    match prefix {
        [Dns(name), Tcp(port)] => Some(((name, *port), suffix)),
        [Dns4(name), Tcp(port)] => Some(((name, *port), suffix)),
        [Dns6(name), Tcp(port)] => Some(((name, *port), suffix)),
        _ => None,
    }
}

/// parse the `&[Protocol]` into the `"/ln-noise-ik/<pubkey>"` prefix and
/// unparsed `&[Protocol]` suffix.
pub fn parse_noise_ik(protos: &[Protocol]) -> Option<(&x25519::PublicKey, &[Protocol])> {
    match protos.split_first() {
        Some((Protocol::NoiseIK(pubkey), suffix)) => Some((pubkey, suffix)),
        _ => None,
    }
}

/// parse the `&[Protocol]` into the `"/ln-peerid-ex"` prefix and unparsed
/// `&[Protocol]` suffix.
pub fn parse_peerid_ex(protos: &[Protocol]) -> Option<&[Protocol]> {
    match protos.split_first() {
        Some((Protocol::PeerIdExchange, suffix)) => Some(suffix),
        _ => None,
    }
}

/// parse the `&[Protocol]` into the `"/ln-handshake/<version>"` prefix and
/// unparsed `&[Protocol]` suffix.
pub fn parse_handshake(protos: &[Protocol]) -> Option<(u8, &[Protocol])> {
    match protos.split_first() {
        Some((Protocol::Handshake(version), suffix)) => Some((*version, suffix)),
        _ => None,
    }
}

/// Extract the second item of a 2-tuple.
fn snd<T, U>(arg: (T, U)) -> U {
    arg.1
}

/// parse canonical libranet protocols
///
/// See: `NetworkAddress::is_libranet_addr(&self)`
fn parse_libranet_protos(protos: &[Protocol]) -> Option<&[Protocol]> {
    // parse_ip_tcp
    // <or> parse_dns_tcp
    // <or> cfg(test) parse_memory

    let opt_transport_suffix = parse_ip_tcp(protos).map(snd);
    let opt_transport_suffix = opt_transport_suffix.or_else(|| parse_dns_tcp(protos).map(snd));
    #[cfg(test)]
    let opt_transport_suffix = opt_transport_suffix.or_else(|| parse_memory(protos).map(snd));
    let transport_suffix = opt_transport_suffix?;

    // and then: parse_noise_ik
    //           <or> cfg(test) parse_peerid_ex

    let opt_auth_suffix = parse_noise_ik(transport_suffix).map(snd);
    #[cfg(test)]
    let opt_auth_suffix = opt_auth_suffix.or_else(|| parse_peerid_ex(protos));
    let auth_suffix = opt_auth_suffix?;

    // and then: parse_handshake

    let handshake_suffix = parse_handshake(auth_suffix).map(snd)?;

    // and then: ensure no trailing protos after handshake

    if handshake_suffix.is_empty() {
        Some(protos)
    } else {
        None
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
                    NoiseIK(pubkey),
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
            if let Ok(addr) = NetworkAddress::from_str(addr_str) {
                panic!(
                    "parsing should fail: input: '{}', output: '{}'",
                    addr_str, addr
                );
            }
        }
    }

    #[test]
    fn test_parse_memory() {
        let addr = NetworkAddress::from_str("/memory/123").unwrap();
        let expected_suffix: &[Protocol] = &[];
        assert_eq!(
            parse_memory(addr.as_slice()).unwrap(),
            (123, expected_suffix)
        );

        let addr = NetworkAddress::from_str("/memory/123/tcp/999").unwrap();
        let expected_suffix: &[Protocol] = &[Protocol::Tcp(999)];
        assert_eq!(
            parse_memory(addr.as_slice()).unwrap(),
            (123, expected_suffix)
        );

        let addr = NetworkAddress::from_str("/tcp/999/memory/123").unwrap();
        assert_eq!(None, parse_memory(addr.as_slice()));
    }

    #[test]
    fn test_parse_ip_tcp() {
        let addr = NetworkAddress::from_str("/ip4/1.2.3.4/tcp/123").unwrap();
        let expected_suffix: &[Protocol] = &[];
        assert_eq!(
            parse_ip_tcp(addr.as_slice()).unwrap(),
            ((IpAddr::from_str("1.2.3.4").unwrap(), 123), expected_suffix)
        );

        let addr = NetworkAddress::from_str("/ip6/::1/tcp/123").unwrap();
        let expected_suffix: &[Protocol] = &[];
        assert_eq!(
            parse_ip_tcp(addr.as_slice()).unwrap(),
            ((IpAddr::from_str("::1").unwrap(), 123), expected_suffix)
        );

        let addr = NetworkAddress::from_str("/ip6/::1/tcp/123/memory/999").unwrap();
        let expected_suffix: &[Protocol] = &[Protocol::Memory(999)];
        assert_eq!(
            parse_ip_tcp(addr.as_slice()).unwrap(),
            ((IpAddr::from_str("::1").unwrap(), 123), expected_suffix)
        );

        let addr = NetworkAddress::from_str("/tcp/999/memory/123").unwrap();
        assert_eq!(None, parse_ip_tcp(addr.as_slice()));
    }

    #[test]
    fn test_parse_dns_tcp() {
        let dns_name = DnsName::from_str("example.com").unwrap();
        let addr = NetworkAddress::from_str("/dns/example.com/tcp/123").unwrap();
        let expected_suffix: &[Protocol] = &[];
        assert_eq!(
            parse_dns_tcp(addr.as_slice()).unwrap(),
            ((&dns_name, 123), expected_suffix)
        );

        let addr = NetworkAddress::from_str("/dns4/example.com/tcp/123").unwrap();
        let expected_suffix: &[Protocol] = &[];
        assert_eq!(
            parse_dns_tcp(addr.as_slice()).unwrap(),
            ((&dns_name, 123), expected_suffix)
        );

        let addr = NetworkAddress::from_str("/dns6/example.com/tcp/123").unwrap();
        let expected_suffix: &[Protocol] = &[];
        assert_eq!(
            parse_dns_tcp(addr.as_slice()).unwrap(),
            ((&dns_name, 123), expected_suffix)
        );

        let addr = NetworkAddress::from_str("/dns/example.com/tcp/123/ln-peerid-ex").unwrap();
        let expected_suffix: &[Protocol] = &[Protocol::PeerIdExchange];
        assert_eq!(
            parse_dns_tcp(addr.as_slice()).unwrap(),
            ((&dns_name, 123), expected_suffix)
        );

        let addr = NetworkAddress::from_str("/tcp/999/memory/123").unwrap();
        assert_eq!(None, parse_dns_tcp(addr.as_slice()));
    }

    #[test]
    fn test_parse_noise_ik() {
        let pubkey_str = "080e287879c918794170e258bfaddd75acac5b3e350419044655e4983a487120";
        let pubkey = x25519::PublicKey::from_encoded_string(pubkey_str).unwrap();
        let addr = NetworkAddress::from_str(&format!("/ln-noise-ik/{}", pubkey_str)).unwrap();
        let expected_suffix: &[Protocol] = &[];
        assert_eq!(
            parse_noise_ik(addr.as_slice()).unwrap(),
            (&pubkey, expected_suffix)
        );

        let addr =
            NetworkAddress::from_str(&format!("/ln-noise-ik/{}/tcp/999", pubkey_str)).unwrap();
        let expected_suffix: &[Protocol] = &[Protocol::Tcp(999)];
        assert_eq!(
            parse_noise_ik(addr.as_slice()).unwrap(),
            (&pubkey, expected_suffix)
        );

        let addr = NetworkAddress::from_str("/tcp/999/memory/123").unwrap();
        assert_eq!(None, parse_noise_ik(addr.as_slice()));
    }

    #[test]
    fn test_parse_peerid_ex() {
        let addr = NetworkAddress::from_str("/ln-peerid-ex").unwrap();
        let expected_suffix: &[Protocol] = &[];
        assert_eq!(parse_peerid_ex(addr.as_slice()).unwrap(), expected_suffix);

        let addr = NetworkAddress::from_str("/ln-peerid-ex/tcp/999").unwrap();
        let expected_suffix: &[Protocol] = &[Protocol::Tcp(999)];
        assert_eq!(parse_peerid_ex(addr.as_slice()).unwrap(), expected_suffix);

        let addr = NetworkAddress::from_str("/tcp/999/memory/123").unwrap();
        assert_eq!(None, parse_peerid_ex(addr.as_slice()));
    }

    #[test]
    fn test_parse_handshake() {
        let addr = NetworkAddress::from_str("/ln-handshake/0").unwrap();
        let expected_suffix: &[Protocol] = &[];
        assert_eq!(
            parse_handshake(addr.as_slice()).unwrap(),
            (0, expected_suffix),
        );

        let addr = NetworkAddress::from_str("/ln-handshake/0/tcp/999").unwrap();
        let expected_suffix: &[Protocol] = &[Protocol::Tcp(999)];
        assert_eq!(
            parse_handshake(addr.as_slice()).unwrap(),
            (0, expected_suffix),
        );

        let addr = NetworkAddress::from_str("/tcp/999/memory/123").unwrap();
        assert_eq!(None, parse_handshake(addr.as_slice()));
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
