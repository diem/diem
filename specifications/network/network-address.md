# NetworkAddress

`NetworkAddress` is a compact, efficient, self-describing and future-proof network address represented as a stack of
protocols, inspired by libp2p's [multiaddr](https://multiformats.io/multiaddr/) format. The primary differences include
using [BCS] to describe the binary format and reducing the set of supported protocols.

<!-- TODO(philiphayes): include `EncNetworkAddress` spec -->

In particular, a `NetworkAddress` is intended to be a fully self-contained description of _how_ to dial a
[DiemNet](README.md) peer, describing both the base transport protocol and all subsequent connection upgrade protocols.

## Data Structures

```rust
/// A `NetworkAddress` is a sequence of `Protocol`s describing how to dial a peer.
///
/// Note: `NetworkAddress` must not be empty.
pub struct NetworkAddress(Vec<Protocol>);

/// A single protocol in the [`NetworkAddress`] protocol stack.
pub enum Protocol {
    Ip4(Ipv4Addr), // effectively [u8; 4]
    Ip6(Ipv6Addr), // effectively [u8; 16]
    Dns(DnsName),
    Dns4(DnsName),
    Dns6(DnsName),
    Tcp(u16),
    Memory(u16),
    // human-readable x25519::PublicKey is lower-case hex encoded
    NoiseIK(x25519::PublicKey),
    Handshake(u8),
}

/// A minimally parsed DNS name. We don't really do any checking other than
/// enforcing:
///
/// 1\. it is not an empty string
/// 2\. it is not larger than 255 bytes
/// 3\. it does not contain any forward slash '/' characters
/// 4\. it is a valid unicode string
///
/// From the [DNS name syntax RFC](https://tools.ietf.org/html/rfc2181#page-13),
/// the standard rules are:
///
/// 1\. the total size <= 255 bytes
/// 2\. each label <= 63 bytes
/// 3\. any binary string is valid
///
/// So the restrictions we're adding are (3) no '/' characters and (4) the name
/// is a valid unicode string. We do this because '/' characters are already our
/// protocol delimiter and Rust's [`::std::net::ToSocketAddr`] API requires a
/// `&str`.
pub struct DnsName(String);
```

<!-- TODO(philiphayes): link to x25519 ser/de spec -->
<!-- TODO(philiphayes): hardening to limit NetworkAddress max size -->

## Human-readable Format

All `NetworkAddress` sent and received over-the-wire or stored on-chain are in their compact [`u8`] [`bcs`]-serialized
format. However, implementations may also wish to implement the optional human-readable format to aid reading
`NetworkAddress`es from configuration or printing `NetworkAddress` in logs.

Each `Protocol` segment is formatted of the form `/<Protocol>/<ProtocolConfig>`, and applies to the following rules:
1. `Protocol` and `ProtocolConfig` must not contain any forward slash `/`
2. Must be able to be UTF-8 encoded

Examples:
```
Ip4([127, 0, 0, 1]) => "/ip4/127.0.0.1",
Ip6([0x2601, 0, .., 0, 0xfebc]) => "/ip6/2601::febc",
Dns("example.com") => "/dns/example.com",
Dns4("example.com") => "/dns4/example.com",
Dns6("example.com") => "/dns6/example.com",
Tcp(6080) => "/tcp/6080",
Memory(1234) => "/memory/1234",
NoiseIK(b"080e287879c918794170e258bfaddd75acac5b3e350419044655e4983a487120") =>
    "/ln-noise-ik/080e287879c918794170e258bfaddd75acac5b3e350419044655e4983a487120",
Handshake(0) => "/ln-handshake/0",
```

A `NetworkAddress` is then just a concatenation of these individually formatted `Protocol` segments:

```
NetworkAddress([
    Ip4([127, 0, 0, 1]),
    Tcp(6080),
    NoiseIK(b"080e287879c918794170e258bfaddd75acac5b3e350419044655e4983a487120"),
    Handshake(0),
]) => "/ip4/127.0.0.1/tcp/6080/ln-noise-ik/080e287879c918794170e258bfaddd75acac5b3e350419044655e4983a487120/ln-handshake/0"
```

A `NetworkAddress` as a concatenation of `Protocol` segments must have the following characteristics:
1. The address must contain exactly one Layer3 protocol (e.g. `Ip4` or `Dns`) and
2. The address must contain exactly one Layer4 protocol (e.g. `Tcp`).
3. `Memory` is a special protocol that is both Layer3 and Layer4.
4. A protocol may be used at most once in an address.
5. The address must not end in a forward slash `/`

Example possible combinations:
* `/ip4/127.0.0.1/tcp/6080`
* `/memory/6080`
* `/dns/novi.com/tcp/80/ln-noise-ik/080e287879c918794170e258bfaddd75acac5b3e350419044655e4983a487120/ln-handshake/0`

## Network Protocol Upgradability

It is important for Diem to support upgrading its network protocols. In particular, providing a process for doing
backwards-incompatible upgrades allows Diem to evolve quickly and prevents protocol ossification.

Each node advertises the full set of protocols needed to speak with a it using
[onchain discovery](onchain-discovery.md). In other words, a node that sees
`"/dns6/example.com/tcp/1234/ln-noise-ik/<pubkey>/ln-handshake/0"` knows unambiguously the precise set of protocols it
must speak and in which order for the listening node to understand them.

By allowing several `NetworkAddress`es to be advertised at once, nodes can also facilitate backwards-incompatible
network protocol upgrades. For instance, suppose Diem wants to modify the Noise secure transport to a different
handshake pattern or crypto primitives (e.g. move from `"Noise_IK_25519_AESGCM_SHA256"` to
`"Noise_IX_25519_ChaChaPoly_BLAKE2s"`). A new binary version of Diem could be released that supports the new Noise
handshake. Nodes that update to the new binary version would then advertise both the old handshake protocol and the new
handshake protocol:

```
addrs := [
    // new protocol
    "/ip4/1.2.3.4/tcp/6181/ln-noise-ix/<pubkey>/ln-handshake/0",
    // old protocol
    "/ip4/1.2.3.4/tcp/6180/ln-noise-ik/<pubkey>/ln-handshake/0",
]
```

Nodes running the old binary (a binary that doesn't understand the new handshake protocol) will simply dial updated
nodes using the advertised address containing the old protocol. Meanwhile, new nodes can use the updated protocol to
speak with each other. By advertising old and new simultaneously, connectivity can be maintained during the upgrade
rollout.

Finally, when all nodes have updated, each node can rotate out the old protocol. A subsequent release can then fully
remove the old handshake logic from the codebase. The final set of advertised addresses for each node will then look
like:

```
// new protocol only
addrs := [
    "/ip4/1.2.3.4/tcp/6181/ln-noise-ix/<pubkey>/ln-handshake/0",
]
```
