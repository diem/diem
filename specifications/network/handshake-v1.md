# Handshake Protocol (v1)

The handshake protocol is used between two connecting peers during the initial connection establishment and upgrade process to negotiate the greatest common supported [DiemNet messaging protocol](messaging-v1.md) version and application protocol set.

## NetworkAddress Protocol

The handshake protocol version itself is "pre-negotiated" in peers' advertised or configured [`NetworkAddress`](network-address.md)es. Canonical DiemNet addresses will include the following `Protocol` after the `"/ln-noise-ik/<public-key>"` `Protocol`:

```
human-readable format: "/ln-handshake/<version>"
```

where supported `<version>` is currently only `0`.

(TODO(philiphayes): `NetworkAddress` handshake version and docs version should be consistent, i.e., `"/ln-handshake/0"` vs `handshake-v1`)

## Data structures

```rust
/// The HandshakeMsg contains a mapping from MessagingProtocolVersion suppported by the node
/// to a bit-vector specifying application-level protocols supported over that version.
pub struct HandshakeMsg {
    pub supported_protocols: BTreeMap<MessagingProtocolVersion, SupportedProtocols>,
}

/// Supported application protocols represented as a bit-vector.
pub struct SupportedProtocols(BitVec);

/// Position _i_ in the bit-vector is set if and only if the _i_th ProtocolId variant
/// is supported by the node.
pub struct BitVec {
    inner: Vec<u8>,
}

/// Enum representing different versions of the Diem network protocol. These should be
/// listed from old to new, old having the smallest value.
/// We derive `PartialOrd` since nodes need to find highest intersecting protocol version.
pub enum MessagingProtocolVersion {
    V1 = 0,
}
```

## Handshake (version = 0)

The `Handshake` protocol is currently symmetric, i.e., we follow the same upgrade procedure regardless of whether we are the dialer or listener in this connection upgrade. Note that serialized `HandshakeMsg`s are big-endian `u16` length-prefixed before sending and receiving over the Noise-wrapped socket.

* **`upgrade()`**

  * Construct a `HandshakeMsg` according to the set of supported DiemNet messaging protocol versions and corresponding application protocols for each version.

    * Note: the `HandshakeMsg` is a _sorted_ map from `MessageProtocolVersion` to `SupportedProtocols` , where `SupportedProtocols` is a bit-vector with a position set if the corresponding `ProtocolId` (represented as a `u8`) is supported over the given DiemNet version.

  * Serialize the `HandshakeMsg` into bytes and prepend the `u16` length-prefix.
  * Send the `u16` length-prefixed, serialized `HandshakeMsg` over the Noise-wrapped socket.
  * Receive the remote peer's `HandshakeMsg` from the Noise-wrapped socket.
  * After receiving the `HandshakeMsg`, both peers MUST pick the highest intersecting `MessagingProtocolVersion` to use for all subsequent communication.
  * Peers MUST only use a `ProtocolId` that is supported by the receiver. The receiver MAY respond with an error message of type `ErrorCode::NotSupported` if it receives a message with a `ProtocolId` it did not advertise or does not support.

(TODO(philiphayes): handshake protocol needs changes to better support client use-case) (TODO(philiphayes): describe and implement hardening: enforce maximum number of entries in supported_protocols map, maximum length of BitVec, no duplicates)
