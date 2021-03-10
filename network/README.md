---
id: network
title: Network
custom_edit_url: https://github.com/diem/diem/edit/main/network/README.md
---

## Overview

For more detailed info, see the [DiemNet Specification](../specifications/network/README.md).

DiemNet is the primary protocol for communication between any two nodes in the
Diem ecosystem. It is specifically designed to facilitate the consensus, shared
mempool, and state sync protocols. DiemNet tries to maintain at-most one connection
with each remote peer; the application protocols to that remote peer are then
multiplexed over the single peer connection.

Currently, it provides application protocols with two primary interfaces:

* DirectSend: for fire-and-forget style message delivery.
* RPC: for unary Remote Procedure Calls.

The network component uses:

* TCP for reliable transport.
* [NoiseIK] for authentication and full end-to-end encryption.
* On-chain [`NetworkAddress`](./network-address/src/lib.rs) set for discovery, with
  optional seed peers in the [`NetworkConfig`](../config/src/config/network_config.rs)
  as a fallback.

Validators will only allow connections from other validators. Their identity and
public key information is provided by the [`simple-onchain-discovery`] protocol,
which updates the eligible member information on each consensus reconfiguration.
Each member of the validator network maintains a full membership view and connects
directly to all other validators in order to maintain a full-mesh network.

In contrast, Validator Full Node (VFNs) servers will only prioritize connections
from more trusted peers in the on-chain discovery set; they will still service
any public clients. Public Full Nodes (PFNs) connecting to VFNs will always
authenticate the VFN server using the available discovery information.

Validator health information, determined using periodic liveness probes, is not
shared between validators; instead, each validator directly monitors its peers
for liveness using the [`HealthChecker`] protocol.

This approach should scale up to a few hundred validators before requiring
partial membership views, sophisticated failure detectors, or network overlays.

## Implementation Details

### System Architecture

```
                      +-----------+---------+------------+--------+
 Application Modules  | Consensus | Mempool | State Sync | Health |
                      +-----------+---------+------------+--------+
                            ^          ^          ^           ^
   Network Interface        |          |          |           |
                            v          v          v           v
                      +----------------+--------------------------+   +---------------------+
      Network Module  |                 PeerManager               |<->| ConnectivityManager |
                      +----------------------+--------------------+   +---------------------+
                      |        Peer(s)       |                    |
                      +----------------------+                    |
                      |                DiemTransport              |
                      +-------------------------------------------+
```

The network component is implemented in the
[Actor](https://en.wikipedia.org/wiki/Actor_model) model &mdash; it uses
message-passing to communicate between different subcomponents running as
independent "tasks." The [tokio](https://tokio.rs/) framework is used as the task
runtime. The primary subcomponents in the network module are:

* [`Network Interface`] &mdash; The interface provided to application modules
using DiemNet.

* [`PeerManager`] &mdash; Listens for incoming connections, and dials outbound
connections to other peers. Demultiplexes and forwards inbound messages from
[`Peer`]s to appropriate application handlers. Additionally, notifies upstream
components of new or closed connections. Optionally can be connected to
[`ConnectivityManager`] for a network with Discovery.

* [`Peer`] &mdash; Manages a single connection to another peer. It reads and
writes [`NetworkMessage`]es from/to the wire. Currently, it implements the two
protocols: DirectSend and Rpc.

+ [`DiemTransport`] &mdash; A secure, reliable transport. It uses [NoiseIK] over
TCP to negotiate an encrypted and authenticated connection between peers.
The DiemNet version and any Diem-specific application protocols are negotiated
afterward using the [DiemNet Handshake Protocol].

* [`ConnectivityManager`] &mdash; Establishes connections to known peers found
via Discovery. Notifies [`PeerManager`] to make outbound dials, or disconnects based
on updates to known peers via Discovery updates.

* [`simple-onchain-discovery`] &mdash; Discovers the set of peers to connect to
via on-chain configuration. These are the `validator_network_addresses` and
`fullnode_network_addresses` of each [`ValidatorConfig`] in the
[`DiemSystem::validators`] set. Notifies the [`ConnectivityManager`] of updates
to the known peer set.

* [`HealthChecker`] &mdash; Performs periodic liveness probes to ensure the
health of a peer/connection. It resets the connection with the peer if a
configurable number of probes fail in succession. Probes currently fail on a
configurable static timeout.

## How is this module organized?

    network
    ├── benches                    # Network benchmarks
    ├── builder                    # Builds a network from a NetworkConfig
    ├── memsocket                  # In-memory socket interface for tests
    ├── netcore
    │   └── src
    │       ├── transport          # Composable transport API
    │       └── framing            # Read/write length prefixes to sockets
    ├── network-address            # Network addresses and encryption
    ├── simple-onchain-discovery   # Protocol for on-chain peer discovery
    └── src
        ├── peer_manager           # Manage peer connections and messages to/from peers
        ├── peer                   # Handles a single peer connection's state
        ├── connectivity_manager   # Monitor connections and ensure connectivity
        ├── protocols
        │   ├── network            # Application layer interface to network module
        │   ├── direct_send        # Protocol for fire-and-forget style message delivery
        │   ├── health_checker     # Protocol for health probing
        │   ├── rpc                # Protocol for remote procedure calls
        │   └── wire               # Protocol for DiemNet handshakes and messaging
        ├── transport              # The base transport layer for dialing/listening
        └── noise                  # Noise handshaking and wire integration

[`ConnectivityManager`]: ./src/connectivity_manager/mod.rs
[DiemNet Handshake Protocol]: ../specifications/network/handshake-v1.md
[`DiemSystem::validators`]: ../language/diem-framework/modules/doc/DiemSystem.md#struct-diemsystem
[`DiemTransport`]: ./src/transport/mod.rs
[`HealthChecker`]: ./src/protocols/health_checker/mod.rs
[`Network Interface`]: ./src/protocols/network/mod.rs
[`NetworkMessage`]: ./src/protocols/wire/messaging/v1/mod.rs
[NoiseIK]: ../specifications/network/noise.md
[`PeerManager`]: ./src/peer_manager/mod.rs
[`Peer`]: ./src/peer/mod.rs
[`ValidatorConfig`]: ../language/diem-framework/modules/doc/ValidatorConfig.md#struct-config
[`simple-onchain-discovery`]: ./simple-onchain-discovery/src/lib.rs
